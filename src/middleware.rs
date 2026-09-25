use axum::{
    extract::{FromRef, FromRequestParts},
    http::{header, request::Parts},
};

use crate::auth::{verify_token, Claims};
use crate::config::AppConfig;
use crate::error::AppError;

/// Extracts access token from standard Cookie header string (supporting "access_token" or "bdai_access_token")
pub fn extract_token_from_cookies(cookie_header: &str) -> Option<&str> {
    for cookie in cookie_header.split(';') {
        let trimmed = cookie.trim();
        if let Some(token) = trimmed.strip_prefix("access_token=") {
            let val = token.trim();
            if !val.is_empty() {
                return Some(val);
            }
        }
        if let Some(token) = trimmed.strip_prefix("bdai_access_token=") {
            let val = token.trim();
            if !val.is_empty() {
                return Some(val);
            }
        }
    }
    None
}

pub struct CurrentUser(pub Claims);

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
    AppConfig: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 1. Try Authorization header: "Bearer <token>"
        let token_from_auth = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|val| val.to_str().ok())
            .and_then(|auth| auth.strip_prefix("Bearer ").map(|t| t.trim()));

        // 2. Try Cookie header: "access_token=<token>" or "bdai_access_token=<token>"
        let token_from_cookie = parts
            .headers
            .get(header::COOKIE)
            .and_then(|val| val.to_str().ok())
            .and_then(extract_token_from_cookies);

        let token = match (token_from_auth, token_from_cookie) {
            (Some(t), _) if !t.is_empty() => t,
            (_, Some(t)) if !t.is_empty() => t,
            _ => {
                return Err(AppError::Unauthorized(
                    "Missing authentication token in Authorization header or Cookie".to_string(),
                ));
            }
        };

        let config = AppConfig::from_ref(state);
        let claims = verify_token(token, &config.jwt_secret)?;

        Ok(CurrentUser(claims))
    }
}

/// Extractor that enforces that the caller has role "Admin".
/// Strictly guards: Only Admin can Add, Remove an Admin, Moderator and all kinds of employees.
pub struct AdminOnly(pub Claims);

impl<S> FromRequestParts<S> for AdminOnly
where
    S: Send + Sync,
    AppConfig: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let CurrentUser(claims) = CurrentUser::from_request_parts(parts, state).await?;

        if !claims.is_admin() {
            return Err(AppError::Forbidden(
                "Access denied: Only Admins can Add or Remove an Admin, Moderator, and all kinds of employees."
                    .to_string(),
            ));
        }

        Ok(AdminOnly(claims))
    }
}

/// Extractor that allows Admin and Moderator (for editing news, tenders, milestones).
pub struct EditorUser(pub Claims);

impl<S> FromRequestParts<S> for EditorUser
where
    S: Send + Sync,
    AppConfig: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let CurrentUser(claims) = CurrentUser::from_request_parts(parts, state).await?;

        if !claims.can_edit_content() {
            return Err(AppError::Forbidden(
                "Access denied: Content modification requires Moderator or Admin privileges."
                    .to_string(),
            ));
        }

        Ok(EditorUser(claims))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_token_from_cookies() {
        let cookie_str = "theme=dark; access_token=my_secret_jwt_token_123; session_id=xyz";
        assert_eq!(extract_token_from_cookies(cookie_str), Some("my_secret_jwt_token_123"));

        let bdai_cookie_str = "bdai_access_token=another_token_456; other=1";
        assert_eq!(extract_token_from_cookies(bdai_cookie_str), Some("another_token_456"));

        let no_token = "theme=light; lang=en";
        assert_eq!(extract_token_from_cookies(no_token), None);
    }
}
