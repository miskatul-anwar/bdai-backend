use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UserRole {
    Admin,
    Moderator,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Admin => "Admin",
            UserRole::Moderator => "Moderator",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Admin" => Some(UserRole::Admin),
            "Moderator" => Some(UserRole::Moderator),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    pub avatar: Option<String>,
    pub department: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: String,
    pub avatar: Option<String>,
    pub department: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            name: u.name,
            email: u.email,
            role: u.role,
            avatar: u.avatar,
            department: u.department,
            status: u.status,
            created_at: u.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub password: Option<String>,
    pub role: String, // Admin | Moderator
    pub avatar: Option<String>,
    pub department: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub role: Option<String>,
    pub avatar: Option<String>,
    pub department: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Deserialize)]
pub struct GoogleAuthRequest {
    /// Google ID token (from Google Identity Services / One Tap / Credential response)
    pub credential: Option<String>,
    /// Or authorization code (from OAuth 2.0 authorization code flow)
    pub code: Option<String>,
    pub redirect_uri: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct GoogleTokenInfo {
    pub iss: Option<String>,
    pub sub: Option<String>,
    pub aud: Option<String>,
    pub email: String,
    pub email_verified: Option<serde_json::Value>,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub error_description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GoogleAuthUrlResponse {
    pub url: String,
    pub client_id: Option<String>,
}

