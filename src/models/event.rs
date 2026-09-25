use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GalleryItem {
    pub src: String,
    pub alt: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    pub id: String,
    pub title: String,
    pub date: String, // e.g. "29th July 2026", "2.00PM · 19th May 2026"
    #[serde(default = "default_status")]
    pub status: String, // "held" | "upcoming"
    #[serde(default = "default_category")]
    pub category: String, // "Workshop", "Seminar", "PhD Seminar", "Guest Lecture"
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    pub banner: String,
    #[serde(default)]
    pub gallery: Vec<GalleryItem>,
    #[serde(default)]
    pub order: i32,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

fn default_status() -> String {
    "held".to_string()
}

fn default_category() -> String {
    "Event".to_string()
}

#[derive(Debug, Deserialize)]
pub struct CreateEventRequest {
    #[serde(default)]
    pub id: Option<String>,
    pub title: String,
    pub date: String,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    pub banner: String,
    #[serde(default)]
    pub gallery: Vec<GalleryItem>,
    #[serde(default)]
    pub order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEventRequest {
    pub title: Option<String>,
    pub date: Option<String>,
    pub status: Option<String>,
    pub category: Option<String>,
    pub location: Option<String>,
    pub description: Option<String>,
    pub banner: Option<String>,
    pub gallery: Option<Vec<GalleryItem>>,
    pub order: Option<i32>,
}
