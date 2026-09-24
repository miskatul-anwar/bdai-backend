use axum::{extract::State, Json};
use serde_json::json;
use sqlx::PgPool;

use crate::error::AppError;

/// GET /api/health - Health check and DB ping
pub async fn health_check(
    State(pool): State<PgPool>,
) -> Result<Json<serde_json::Value>, AppError> {
    let db_ok = sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .is_ok();

    Ok(Json(json!({
        "status": if db_ok { "healthy" } else { "degraded" },
        "service": "bdai_backend",
        "version": env!("CARGO_PKG_VERSION"),
        "database": if db_ok { "connected" } else { "disconnected" },
        "timestamp": chrono::Utc::now().to_rfc3339(),
    })))
}
