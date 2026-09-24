use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TeamMember {
    pub id: Uuid,
    pub name: String,
    pub designation: String, // Fully editable, no fixed categories
    pub role: Option<String>,
    pub category: Option<String>,
    pub institution: String,
    pub email: Option<String>,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub scholar_url: Option<String>,
    pub linkedin_url: Option<String>,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTeamMemberRequest {
    pub name: String,
    pub designation: String, // Free-text designation
    pub role: Option<String>,
    pub category: Option<String>,
    pub institution: Option<String>,
    pub email: Option<String>,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub scholar_url: Option<String>,
    pub linkedin_url: Option<String>,
    pub display_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTeamMemberRequest {
    pub name: Option<String>,
    pub designation: Option<String>, // Free-text designation
    pub role: Option<String>,
    pub category: Option<String>,
    pub institution: Option<String>,
    pub email: Option<String>,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub scholar_url: Option<String>,
    pub linkedin_url: Option<String>,
    pub display_order: Option<i32>,
}
