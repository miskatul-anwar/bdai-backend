use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tool {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub abstract_text: Option<String>,
    #[serde(default)]
    pub paper_url: Option<String>,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default)]
    pub platform_url: Option<String>,
    #[serde(default)]
    pub video_url: Option<String>,
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub authors: Option<String>,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub display_order: i32,
    #[serde(default)]
    pub badge: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateToolRequest {
    pub title: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub abstract_text: Option<String>,
    #[serde(default)]
    pub paper_url: Option<String>,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default)]
    pub platform_url: Option<String>,
    #[serde(default)]
    pub video_url: Option<String>,
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub authors: Option<String>,
    #[serde(default)]
    pub features: Option<Vec<String>>,
    #[serde(default)]
    pub display_order: Option<i32>,
    #[serde(default)]
    pub badge: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateToolRequest {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub abstract_text: Option<String>,
    pub paper_url: Option<String>,
    pub source_url: Option<String>,
    pub platform_url: Option<String>,
    pub video_url: Option<String>,
    pub image_url: Option<String>,
    pub authors: Option<String>,
    pub features: Option<Vec<String>>,
    pub display_order: Option<i32>,
    pub badge: Option<String>,
}
