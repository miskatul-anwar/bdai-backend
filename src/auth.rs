use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{AuthResponse, User, UserResponse};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // User ID (UUID string)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    pub email: String,
    pub name: String,
    pub role: String, // Admin | Moderator
    #[serde(default = "default_token_type")]
    pub token_type: Option<String>, // "access"
    pub exp: usize,
    pub iat: usize,
}

fn default_token_type() -> Option<String> {
    Some("access".to_string())
}

impl Claims {
    pub fn is_admin(&self) -> bool {
        self.role.eq_ignore_ascii_case("Admin")
    }

    pub fn is_moderator(&self) -> bool {
        self.role.eq_ignore_ascii_case("Moderator")
    }

    pub fn can_edit_content(&self) -> bool {
        self.is_admin() || self.is_moderator()
    }

    pub fn user_uuid(&self) -> Result<Uuid, AppError> {
        Uuid::parse_str(&self.sub).map_err(|_| AppError::Unauthorized("Invalid user ID in token".to_string()))
    }
}

/// Create an explicit JWT Access Token with standard claims
pub fn create_access_token(user: &User, secret: &str, expiration_hours: i64) -> Result<String, AppError> {
    let now = Utc::now();
    let exp = (now + Duration::hours(expiration_hours)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = Claims {
        sub: user.id.to_string(),
        username: user.username.clone(),
        email: user.email.clone(),
        name: user.name.clone(),
        role: user.role.clone(),
        token_type: Some("access".to_string()),
        exp,
        iat,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to sign JWT access token: {e}")))
}

/// Helper that bundles access token, metadata, and user info into standard AuthResponse
pub fn create_auth_response(user: User, secret: &str, expiration_hours: i64) -> Result<AuthResponse, AppError> {
    let access_token = create_access_token(&user, secret, expiration_hours)?;
    let expires_in = expiration_hours * 3600;

    Ok(AuthResponse {
        token: access_token.clone(),
        access_token,
        token_type: "Bearer".to_string(),
        expires_in,
        user: UserResponse::from(user),
    })
}

/// Backwards-compatible alias for create_access_token
#[allow(dead_code)]
pub fn create_token(user: &User, secret: &str, expiration_hours: i64) -> Result<String, AppError> {
    create_access_token(user, secret, expiration_hours)
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.validate_exp = true;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_hash() {
        let valid_hash = hash_password("admin123").unwrap();
        println!("VALID_HASH_FOR_ADMIN123: {}", valid_hash);
        assert!(verify_password("admin123", &valid_hash));
    }

    #[test]
    fn test_jwt_access_token_creation_and_verification() {
        let user = User {
            id: Uuid::new_v4(),
            name: "Miskat Hasan".to_string(),
            username: Some("miskat".to_string()),
            email: "miskat.cse@cu.ac.bd".to_string(),
            password_hash: "hash".to_string(),
            role: "Admin".to_string(),
            avatar: None,
            department: "CSE CU".to_string(),
            status: "active".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let secret = "super_secret_test_key_for_jwt_validation";
        let token = create_access_token(&user, secret, 24).expect("Token generation should succeed");
        let claims = verify_token(&token, secret).expect("Token verification should succeed");

        assert_eq!(claims.sub, user.id.to_string());
        assert_eq!(claims.username.as_deref(), Some("miskat"));
        assert_eq!(claims.email, "miskat.cse@cu.ac.bd");
        assert_eq!(claims.role, "Admin");
        assert_eq!(claims.token_type.as_deref(), Some("access"));
        assert!(claims.is_admin());
        assert!(claims.can_edit_content());
    }
}
