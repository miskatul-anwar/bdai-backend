mod auth;
mod cloudinary;
mod config;
mod db;
mod error;
mod middleware;
mod models;
mod routes;
mod state;

use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer, key_extractor::SmartIpKeyExtractor};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::AppConfig;
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize structured logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,bdai_backend=debug,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting BDAI Rust Backend Server...");

    // 2. Load configuration
    let config = AppConfig::from_env();

    // 3. Connect to Supabase / PostgreSQL database
    let db_pool = match db::create_pool(&config.database_url).await {
        Ok(pool) => {
            if let Err(e) = db::run_migrations_if_needed(&pool).await {
                error!("Database migration warning: {e}");
            }
            pool
        }
        Err(e) => {
            error!(
                "Failed to connect to database at '{}': {}",
                config.database_url, e
            );
            error!("TIP: Set DATABASE_URL in bdai_backend/.env to your Supabase connection string");
            error!("     or run 'supabase start' to start local Supabase containers.");
            return Err(e.into());
        }
    };

    let state = AppState {
        db: db_pool,
        config: config.clone(),
    };

    // 4. Configure Hardened CORS with Credentials & Cookie Support
    let cors = if config.cors_origins.is_empty() || config.cors_origins.contains(&"*".to_string()) {
        CorsLayer::new()
            .allow_origin(tower_http::cors::AllowOrigin::mirror_request())
            .allow_methods([
                axum::http::Method::GET,
                axum::http::Method::POST,
                axum::http::Method::PUT,
                axum::http::Method::DELETE,
                axum::http::Method::OPTIONS,
            ])
            .allow_headers([
                axum::http::header::AUTHORIZATION,
                axum::http::header::CONTENT_TYPE,
                axum::http::header::ACCEPT,
                axum::http::header::COOKIE,
            ])
            .expose_headers([
                axum::http::header::SET_COOKIE,
            ])
            .allow_credentials(true)
    } else {
        let allowed_origins: Vec<axum::http::HeaderValue> = config
            .cors_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();

        CorsLayer::new()
            .allow_origin(allowed_origins)
            .allow_methods([
                axum::http::Method::GET,
                axum::http::Method::POST,
                axum::http::Method::PUT,
                axum::http::Method::DELETE,
                axum::http::Method::OPTIONS,
            ])
            .allow_headers([
                axum::http::header::AUTHORIZATION,
                axum::http::header::CONTENT_TYPE,
                axum::http::header::ACCEPT,
                axum::http::header::COOKIE,
            ])
            .expose_headers([
                axum::http::header::SET_COOKIE,
            ])
            .allow_credentials(true)
    };

    // 5. Configure Rate Limiting (Token Bucket per Client IP via SmartIpKeyExtractor)
    let governor_conf = std::sync::Arc::new(
        GovernorConfigBuilder::default()
            .per_second(config.rate_limit_per_second)
            .burst_size(config.rate_limit_burst)
            .use_headers()
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .unwrap(),
    );

    let governor_limiter = governor_conf.limiter().clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            governor_limiter.retain_recent();
        }
    });

    // 6. Build Axum router with Comprehensive Security Hardening
    // - Health router (bypasses rate limiting for Render/Uptime health monitors)
    let health_router = routes::create_health_router();

    // - API router (protected by token-bucket rate limiting)
    let api_router = routes::create_api_router()
        .layer(GovernorLayer::new(governor_conf));

    let app = health_router
        .merge(api_router)
        .layer(axum::middleware::from_fn(security_headers_middleware))
        .layer(axum::extract::DefaultBodyLimit::max(15 * 1024 * 1024))
        .layer(tower_http::timeout::TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            std::time::Duration::from_secs(30),
        ))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("============================================================");
    info!("          BDAI Platform Rust Backend Active                 ");
    info!("  Database:   Connected to Supabase PostgreSQL             ");
    info!("  Listening:  http://0.0.0.0:{}", config.port);
    info!("  Health API: http://localhost:{}/api/health", config.port);
    info!("  Rate Limit: {} req/s (burst: {}) per client IP", config.rate_limit_per_second, config.rate_limit_burst);
    info!("  Admin RBAC: Only Admin can Add/Remove Users & Employees   ");
    info!("============================================================");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("BDAI Backend Server stopped gracefully.");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("Received Ctrl+C, initiating shutdown..."),
        _ = terminate => info!("Received SIGTERM, initiating shutdown..."),
    }
}

/// Injects industry-standard OWASP HTTP security headers into every API response
async fn security_headers_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    // 1. Prevent MIME-sniffing
    headers.insert(
        axum::http::header::X_CONTENT_TYPE_OPTIONS,
        axum::http::HeaderValue::from_static("nosniff"),
    );

    // 2. Prevent Clickjacking
    headers.insert(
        axum::http::header::X_FRAME_OPTIONS,
        axum::http::HeaderValue::from_static("DENY"),
    );

    // 3. XSS Filter (legacy defense)
    headers.insert(
        axum::http::HeaderName::from_static("x-xss-protection"),
        axum::http::HeaderValue::from_static("1; mode=block"),
    );

    // 4. Referrer Policy
    headers.insert(
        axum::http::header::REFERRER_POLICY,
        axum::http::HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // 5. Permissions Policy
    headers.insert(
        axum::http::HeaderName::from_static("permissions-policy"),
        axum::http::HeaderValue::from_static(
            "camera=(), microphone=(), geolocation=(), payment=(), usb=()",
        ),
    );

    // 6. Content Security Policy for API
    headers.insert(
        axum::http::HeaderName::from_static("content-security-policy"),
        axum::http::HeaderValue::from_static("default-src 'none'; frame-ancestors 'none'; sandbox"),
    );

    // 7. Strict-Transport-Security (HSTS) - 1 year
    headers.insert(
        axum::http::header::STRICT_TRANSPORT_SECURITY,
        axum::http::HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
    );

    // 8. Remove Server header to prevent fingerprinting
    headers.remove(axum::http::header::SERVER);

    response
}
