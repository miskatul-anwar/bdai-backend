use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
    Json,
};
use sqlx::PgPool;
use std::collections::HashMap;

use crate::auth::{create_auth_response, verify_password, verify_token};
use crate::config::AppConfig;
use crate::error::AppError;
use crate::middleware::CurrentUser;
use crate::models::{
    AuthResponse, GoogleAuthRequest, GoogleAuthUrlResponse, GoogleTokenInfo, LoginRequest, User,
    UserResponse, VerifyTokenRequest, VerifyTokenResponse,
};

pub async fn login(
    State(pool): State<PgPool>,
    State(config): State<AppConfig>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let identifier = payload
        .username
        .as_deref()
        .or(payload.email.as_deref())
        .map(|s| s.trim().to_lowercase())
        .unwrap_or_default();

    if identifier.is_empty() {
        return Err(AppError::BadRequest("Username or email is required".to_string()));
    }

    // Dummy bcrypt hash (cost 12) used to equalize response latency
    // and completely eliminate username enumeration through timing analysis.
    const DUMMY_HASH: &str = "$2b$12$e8uqgXhWd6FqIqD8iI2V9e2g3Hk7FzJ7P7L1X9p8r7t6y5w4u3v2q";

    let user_opt = sqlx::query_as::<_, User>(
        "SELECT id, name, username, email, password_hash, role, avatar, department, status, created_at, updated_at
         FROM public.users
         WHERE LOWER(username) = $1 OR LOWER(email) = $1",
    )
    .bind(&identifier)
    .fetch_optional(&pool)
    .await?;

    let (user, is_valid_password) = match user_opt {
        Some(u) => {
            let valid = verify_password(&payload.password, &u.password_hash);
            (Some(u), valid)
        }
        None => {
            let _ = verify_password(&payload.password, DUMMY_HASH);
            (None, false)
        }
    };

    let user = match user {
        Some(u) if is_valid_password => u,
        _ => return Err(AppError::Unauthorized("Invalid username/email or password".to_string())),
    };

    if user.status != "active" {
        return Err(AppError::Forbidden("Account is inactive".to_string()));
    }

    let auth_resp = create_auth_response(user, &config.jwt_secret, config.jwt_expiration_hours)?;

    // Log login activity
    let _ = sqlx::query(
        "INSERT INTO public.activity_logs (action, entity, target_name, user_name)
         VALUES ($1, $2, $3, $4)",
    )
    .bind("Signed In (Credentials)")
    .bind("Session")
    .bind(auth_resp.user.username.as_deref().unwrap_or(&auth_resp.user.email))
    .bind(&auth_resp.user.name)
    .execute(&pool)
    .await;

    Ok(Json(auth_resp))
}

/// POST /api/auth/refresh - Refresh JWT Access Token for an active user
pub async fn refresh(
    State(pool): State<PgPool>,
    State(config): State<AppConfig>,
    CurrentUser(claims): CurrentUser,
) -> Result<Json<AuthResponse>, AppError> {
    let user_id = claims.user_uuid()?;

    let user = sqlx::query_as::<_, User>(
        "SELECT id, name, username, email, password_hash, role, avatar, department, status, created_at, updated_at
         FROM public.users
         WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    if user.status != "active" {
        return Err(AppError::Forbidden("Account is inactive".to_string()));
    }

    let auth_resp = create_auth_response(user, &config.jwt_secret, config.jwt_expiration_hours)?;
    Ok(Json(auth_resp))
}

/// POST /api/auth/verify - Verify a JWT access token and return token validity + decoded claims
pub async fn verify(
    State(config): State<AppConfig>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<VerifyTokenRequest>,
) -> Result<Json<VerifyTokenResponse>, AppError> {
    let token = payload.token.or_else(|| {
        headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|val| val.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer ").map(|s| s.to_string()))
    });

    let token = match token {
        Some(t) if !t.trim().is_empty() => t.trim().to_string(),
        _ => {
            return Ok(Json(VerifyTokenResponse {
                valid: false,
                claims: None,
                message: Some("No token provided in request body or Authorization header".to_string()),
            }));
        }
    };

    match verify_token(&token, &config.jwt_secret) {
        Ok(claims) => Ok(Json(VerifyTokenResponse {
            valid: true,
            claims: Some(claims),
            message: Some("Token is valid".to_string()),
        })),
        Err(e) => Ok(Json(VerifyTokenResponse {
            valid: false,
            claims: None,
            message: Some(e.to_string()),
        })),
    }
}

pub async fn me(
    State(pool): State<PgPool>,
    CurrentUser(claims): CurrentUser,
) -> Result<Json<UserResponse>, AppError> {
    let user_id = claims.user_uuid()?;

    let user = sqlx::query_as::<_, User>(
        "SELECT id, name, username, email, password_hash, role, avatar, department, status, created_at, updated_at
         FROM public.users
         WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(UserResponse::from(user)))
}

/// Helper: RFC 3986 percent-encoding for URLs
fn urlencoding_encode(s: &str) -> String {
    let mut encoded = String::new();
    for b in s.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}

/// Helper: Verify Google ID token via Google's tokeninfo service
async fn verify_google_id_token(
    id_token: &str,
    expected_client_id: Option<&str>,
) -> Result<GoogleTokenInfo, AppError> {
    let url = format!("https://oauth2.googleapis.com/tokeninfo?id_token={}", id_token);
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to reach Google token validation service: {e}")))?;

    if !resp.status().is_success() {
        return Err(AppError::Unauthorized(
            "Invalid, expired, or untrusted Google ID token".to_string(),
        ));
    }

    let token_info: GoogleTokenInfo = resp
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse Google tokeninfo response: {e}")))?;

    if let Some(err_desc) = &token_info.error_description {
        return Err(AppError::Unauthorized(format!("Google OAuth error: {err_desc}")));
    }

    // Verify email is verified by Google
    let verified = match &token_info.email_verified {
        Some(serde_json::Value::Bool(b)) => *b,
        Some(serde_json::Value::String(s)) => s == "true",
        _ => false,
    };

    if !verified {
        return Err(AppError::Unauthorized(
            "Google account email is not verified".to_string(),
        ));
    }

    // Verify audience if configured
    if let Some(client_id) = expected_client_id {
        if let Some(aud) = &token_info.aud {
            if aud != client_id {
                return Err(AppError::Unauthorized(
                    "Google token audience (client ID) mismatch".to_string(),
                ));
            }
        }
    }

    Ok(token_info)
}

/// Helper: Exchange Google authorization code for ID/access tokens
async fn exchange_google_code(
    code: &str,
    redirect_uri: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<GoogleTokenInfo, AppError> {
    let client = reqwest::Client::new();
    let params = [
        ("code", code),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("redirect_uri", redirect_uri),
        ("grant_type", "authorization_code"),
    ];

    let resp = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to exchange Google OAuth code: {e}")))?;

    if !resp.status().is_success() {
        let err_body = resp.text().await.unwrap_or_default();
        return Err(AppError::Unauthorized(format!(
            "Failed to exchange Google OAuth code: {err_body}"
        )));
    }

    #[derive(serde::Deserialize)]
    struct TokenResponse {
        id_token: Option<String>,
        access_token: Option<String>,
    }

    let token_resp: TokenResponse = resp
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse Google token exchange response: {e}")))?;

    if let Some(id_token) = token_resp.id_token {
        verify_google_id_token(&id_token, Some(client_id)).await
    } else if let Some(access_token) = token_resp.access_token {
        // Fetch userinfo using access token
        let userinfo_resp = client
            .get("https://www.googleapis.com/oauth2/v3/userinfo")
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch Google userinfo: {e}")))?;

        #[derive(serde::Deserialize)]
        struct UserInfo {
            email: String,
            email_verified: Option<bool>,
            name: Option<String>,
            picture: Option<String>,
        }

        let userinfo: UserInfo = userinfo_resp
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse Google userinfo: {e}")))?;

        Ok(GoogleTokenInfo {
            iss: Some("https://accounts.google.com".to_string()),
            sub: None,
            aud: Some(client_id.to_string()),
            email: userinfo.email,
            email_verified: Some(serde_json::Value::Bool(userinfo.email_verified.unwrap_or(true))),
            name: userinfo.name,
            picture: userinfo.picture,
            error_description: None,
        })
    } else {
        Err(AppError::Unauthorized(
            "No id_token or access_token returned by Google".to_string(),
        ))
    }
}

/// Helper: Process authorized Google user and enforce strict whitelist RBAC
async fn process_google_user(
    token_info: GoogleTokenInfo,
    pool: &PgPool,
    config: &AppConfig,
) -> Result<AuthResponse, AppError> {
    let email = token_info.email.trim().to_lowercase();

    // STRICT RBAC CHECK:
    // Only users whose email has previously been created/authorized by an Admin can sign in.
    let user = sqlx::query_as::<_, User>(
        "SELECT id, name, username, email, password_hash, role, avatar, department, status, created_at, updated_at
         FROM public.users
         WHERE LOWER(email) = $1",
    )
    .bind(&email)
    .fetch_optional(pool)
    .await?;

    let mut user = match user {
        Some(u) => u,
        None => {
            return Err(AppError::Forbidden(format!(
                "Access Denied: The Google account '{email}' is not authorized. An Administrator must first add your email before you can sign in."
            )));
        }
    };

    if user.status != "active" {
        return Err(AppError::Forbidden(
            "Account is inactive. Please contact an Administrator.".to_string(),
        ));
    }

    // Sync avatar from Google profile if user has default avatar or none
    if let Some(picture) = &token_info.picture {
        if user.avatar.as_deref() == Some("/team/miskat.jpg") || user.avatar.is_none() {
            let _ = sqlx::query("UPDATE public.users SET avatar = $1, updated_at = now() WHERE id = $2")
                .bind(picture)
                .bind(user.id)
                .execute(pool)
                .await;
            user.avatar = Some(picture.clone());
        }
    }

    let auth_resp = create_auth_response(user, &config.jwt_secret, config.jwt_expiration_hours)?;

    // Audit log
    let _ = sqlx::query(
        "INSERT INTO public.activity_logs (action, entity, target_name, user_name)
         VALUES ($1, $2, $3, $4)",
    )
    .bind("Signed In (Google OAuth)")
    .bind("Session")
    .bind(&auth_resp.user.email)
    .bind(&auth_resp.user.name)
    .execute(pool)
    .await;

    Ok(auth_resp)
}

/// POST /api/auth/google
/// Accepts Google ID Token (`credential`) or OAuth authorization code (`code`)
pub async fn google_auth(
    State(pool): State<PgPool>,
    State(config): State<AppConfig>,
    Json(payload): Json<GoogleAuthRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let token_info = if let Some(cred) = payload.credential {
        verify_google_id_token(&cred, config.google_client_id.as_deref()).await?
    } else if let Some(code) = payload.code {
        let client_id = config.google_client_id.as_ref().ok_or_else(|| {
            AppError::BadRequest("GOOGLE_CLIENT_ID is not configured on the server".to_string())
        })?;
        let client_secret = config.google_client_secret.as_ref().ok_or_else(|| {
            AppError::BadRequest("GOOGLE_CLIENT_SECRET is not configured on the server".to_string())
        })?;
        let redirect_uri = payload
            .redirect_uri
            .or_else(|| config.google_redirect_uri.clone())
            .unwrap_or_else(|| format!("{}/api/auth/google/callback", config.frontend_url));

        exchange_google_code(&code, &redirect_uri, client_id, client_secret).await?
    } else {
        return Err(AppError::BadRequest(
            "Either 'credential' (Google ID token) or 'code' (Auth code) must be provided in request body"
                .to_string(),
        ));
    };

    let auth_resp = process_google_user(token_info, &pool, &config).await?;
    Ok(Json(auth_resp))
}

/// GET /api/auth/google/url
/// Returns Google OAuth consent URL for redirect-based login
pub async fn google_auth_url(
    State(config): State<AppConfig>,
) -> Result<Json<GoogleAuthUrlResponse>, AppError> {
    let client_id = config.google_client_id.clone().unwrap_or_default();
    let redirect_uri = config
        .google_redirect_uri
        .clone()
        .unwrap_or_else(|| format!("{}/api/auth/google/callback", config.frontend_url));

    let encoded_redirect = urlencoding_encode(&redirect_uri);
    let url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope=openid%20email%20profile&access_type=offline&prompt=select_account",
        client_id, encoded_redirect
    );

    Ok(Json(GoogleAuthUrlResponse {
        url,
        client_id: config.google_client_id,
    }))
}

/// GET /api/auth/google/callback
/// Redirect target for Google OAuth consent screen
pub async fn google_callback(
    State(pool): State<PgPool>,
    State(config): State<AppConfig>,
    Query(params): Query<HashMap<String, String>>,
) -> axum::response::Response {
    let frontend_url = &config.frontend_url;

    if let Some(err) = params.get("error") {
        let redirect_to = format!("{}/login?error={}", frontend_url, urlencoding_encode(err));
        return Redirect::to(&redirect_to).into_response();
    }

    let code = match params.get("code") {
        Some(c) => c,
        None => {
            let redirect_to = format!("{}/login?error=missing_auth_code", frontend_url);
            return Redirect::to(&redirect_to).into_response();
        }
    };

    let client_id = match &config.google_client_id {
        Some(id) => id,
        None => {
            let redirect_to = format!("{}/login?error=google_oauth_unconfigured", frontend_url);
            return Redirect::to(&redirect_to).into_response();
        }
    };

    let client_secret = match &config.google_client_secret {
        Some(sec) => sec,
        None => {
            let redirect_to = format!("{}/login?error=google_oauth_secret_missing", frontend_url);
            return Redirect::to(&redirect_to).into_response();
        }
    };

    let redirect_uri = config
        .google_redirect_uri
        .clone()
        .unwrap_or_else(|| format!("{}/api/auth/google/callback", config.frontend_url));

    let token_info = match exchange_google_code(code, &redirect_uri, client_id, client_secret).await {
        Ok(ti) => ti,
        Err(e) => {
            let redirect_to = format!("{}/login?error={}", frontend_url, urlencoding_encode(&e.to_string()));
            return Redirect::to(&redirect_to).into_response();
        }
    };

    match process_google_user(token_info, &pool, &config).await {
        Ok(auth_resp) => {
            let redirect_to = format!("{}/login?token={}", frontend_url, auth_resp.token);
            Redirect::to(&redirect_to).into_response()
        }
        Err(e) => {
            let redirect_to = format!("{}/login?error={}", frontend_url, urlencoding_encode(&e.to_string()));
            Redirect::to(&redirect_to).into_response()
        }
    }
}
