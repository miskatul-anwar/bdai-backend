use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NewsArticle {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub excerpt: String,
    pub content: String,
    pub category: String, // news | event | workshop | announcement
    pub publish_date: NaiveDate,
    pub author: String,
    pub status: String, // published | draft
    pub featured: bool,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNewsRequest {
    pub title: String,
    pub slug: Option<String>,
    pub excerpt: String,
    pub content: String,
    pub category: String,
    pub publish_date: Option<NaiveDate>,
    pub author: Option<String>,
    pub status: Option<String>,
    pub featured: Option<bool>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNewsRequest {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub excerpt: Option<String>,
    pub content: Option<String>,
    pub category: Option<String>,
    pub publish_date: Option<NaiveDate>,
    pub author: Option<String>,
    pub status: Option<String>,
    pub featured: Option<bool>,
    pub tags: Option<Vec<String>>,
}
