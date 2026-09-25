use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Partner {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub logo: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub website: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default, rename = "type")]
    pub partner_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePartnerRequest {
    pub name: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub logo: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default, rename = "type")]
    pub partner_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePartnerRequest {
    pub name: Option<String>,
    pub logo: Option<String>,
    pub description: Option<String>,
    pub website: Option<String>,
    pub url: Option<String>,
    pub role: Option<String>,
    #[serde(rename = "type")]
    pub partner_type: Option<String>,
}
