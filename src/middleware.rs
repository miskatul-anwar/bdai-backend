use axum::{
    extract::{FromRef, FromRequestParts},
    http::{header, request::Parts},
};

use crate::auth::{verify_token, Claims};
use crate::config::AppConfig;
use crate::error::AppError;

pub struct CurrentUser(pub Claims);

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
    AppConfig: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|val| val.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("Invalid Bearer token format".to_string()))?;

        let config = AppConfig::from_ref(state);
        let claims = verify_token(token, &config.jwt_secret)?;

        Ok(CurrentUser(claims))
    }
}

/// Extractor that enforces that the caller has role "Admin".
/// Strictly guards: Only Admin can Add, Remove an Admin, Moderator, Member and all kinds of employees.
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
                "Access denied: Only Admins can Add or Remove an Admin, Moderator, Member, and all kinds of employees."
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
