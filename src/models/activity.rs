use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ActivityLog {
    pub id: Uuid,
    pub action: String,
    pub entity: String,
    pub target_name: String,
    pub user_name: String,
    pub created_at: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct CreateActivityRequest {
    pub action: String,
    pub entity: String,
    pub target_name: String,
    pub user_name: String,
}
