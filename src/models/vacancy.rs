use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Vacancy {
    pub id: Uuid,
    pub title: String,
    pub department: String,
    pub work_package: String,
    pub notice_type: String, // Free-form notice type (e.g. e-Tender Notice, Research Fellowship)
    pub location: String,
    pub deadline: NaiveDate,
    pub status: String, // open | closed
    pub description: String,
    pub requirements: Vec<String>,
    pub applicant_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateVacancyRequest {
    pub title: String,
    pub department: Option<String>,
    pub work_package: Option<String>,
    pub notice_type: String, // Free-form string
    pub location: Option<String>,
    pub deadline: NaiveDate,
    pub status: Option<String>,
    pub description: String,
    pub requirements: Option<Vec<String>>,
    pub applicant_count: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVacancyRequest {
    pub title: Option<String>,
    pub department: Option<String>,
    pub work_package: Option<String>,
    pub notice_type: Option<String>, // Free-form string
    pub location: Option<String>,
    pub deadline: Option<NaiveDate>,
    pub status: Option<String>,
    pub description: Option<String>,
    pub requirements: Option<Vec<String>>,
    pub applicant_count: Option<i32>,
}
