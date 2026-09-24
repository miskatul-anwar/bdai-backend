use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ResearchObjective {
    pub id: String, // e.g. "OB1", "OB2"
    pub title: String,
    pub details: String,
    pub researcher: String,
    pub sector: String,
    pub status: String, // in-progress | completed | planned
    pub progress: i32,  // 0 - 100
    pub deliverables: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateObjectiveRequest {
    pub id: String, // e.g. "OB5"
    pub title: String,
    pub details: String,
    pub researcher: String,
    pub sector: String,
    pub status: Option<String>,
    pub progress: Option<i32>,
    pub deliverables: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateObjectiveRequest {
    pub title: Option<String>,
    pub details: Option<String>,
    pub researcher: Option<String>,
    pub sector: Option<String>,
    pub status: Option<String>,
    pub progress: Option<i32>,
    pub deliverables: Option<i32>,
}
