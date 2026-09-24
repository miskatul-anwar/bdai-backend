use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::User;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // User ID (UUID string)
    pub email: String,
    pub name: String,
    pub role: String, // Admin | Moderator | Member
    pub exp: usize,
    pub iat: usize,
}

impl Claims {
    pub fn is_admin(&self) -> bool {
        self.role.eq_ignore_ascii_case("Admin")
    }

    pub fn is_moderator(&self) -> bool {
        self.role.eq_ignore_ascii_case("Moderator")
    }

    #[allow(dead_code)]
    pub fn is_member(&self) -> bool {
        self.role.eq_ignore_ascii_case("Member")
    }

    pub fn can_edit_content(&self) -> bool {
        self.is_admin() || self.is_moderator()
    }

    pub fn user_uuid(&self) -> Result<Uuid, AppError> {
        Uuid::parse_str(&self.sub).map_err(|_| AppError::Unauthorized("Invalid user ID in token".to_string()))
    }
}

pub fn create_token(user: &User, secret: &str, expiration_hours: i64) -> Result<String, AppError> {
    let now = Utc::now();
    let exp = (now + Duration::hours(expiration_hours)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        name: user.name.clone(),
        role: user.role.clone(),
        exp,
        iat,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to sign JWT token: {e}")))
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| AppError::Unauthorized(format!("Invalid or expired token: {e}")))?;

    Ok(token_data.claims)
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(format!("Password hashing failed: {e}")))
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    // If hash starts with $2, use bcrypt
    if hash.starts_with("$2") {
        bcrypt::verify(password, hash).unwrap_or(false)
    } else {
        // Fallback for development/seed plain comparisons if any
        password == hash
    }
}
