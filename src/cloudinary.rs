use chrono::Utc;
use reqwest::multipart::{Form, Part};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use tracing::{info, warn};

use crate::config::AppConfig;
use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct CloudinaryUploadResponse {
    pub url: String,
    pub secure_url: String,
    pub public_id: String,
    pub format: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub bytes: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct SignatureResponse {
    pub signature: String,
    pub timestamp: i64,
    pub api_key: String,
    pub cloud_name: String,
    pub folder: String,
}

pub struct CloudinaryService;

impl CloudinaryService {
    /// Computes the Cloudinary SHA-1 signature from sorted parameters and api_secret
    pub fn generate_signature(
        params_to_sign: &[(&str, &str)],
        api_secret: &str,
    ) -> String {
        let mut sorted = params_to_sign.to_vec();
        sorted.sort_by_key(|&(k, _)| k);

        let query_string = sorted
            .into_iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("&");

        let to_hash = format!("{query_string}{api_secret}");
        let mut hasher = Sha1::new();
        hasher.update(to_hash.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Uploads an image directly to Cloudinary
    pub async fn upload_image(
        config: &AppConfig,
        file_bytes: Vec<u8>,
        file_name: &str,
        folder_override: Option<&str>,
    ) -> Result<CloudinaryUploadResponse, AppError> {
        let cloud_name = match &config.cloudinary_cloud_name {
            Some(name) if !name.is_empty() && name != "your_cloud_name" => name,
            _ => {
                warn!("Cloudinary credentials not configured or using placeholders. Returning mock development image URL.");
                warn!("TIP: Add your CLOUDINARY_CLOUD_NAME, CLOUDINARY_API_KEY, and CLOUDINARY_API_SECRET in bdai_backend/.env");
                let random_suffix = uuid::Uuid::new_v4().simple().to_string();
                let sanitized_name = file_name.replace(' ', "_");
                let mock_url = format!(
                    "https://res.cloudinary.com/demo/image/upload/v1/bdai/{}_{}",
                    random_suffix, sanitized_name
                );
                return Ok(CloudinaryUploadResponse {
                    url: mock_url.clone(),
                    secure_url: mock_url,
                    public_id: format!("bdai/{}_{}", random_suffix, sanitized_name),
                    format: Some("jpg".to_string()),
                    width: Some(800),
                    height: Some(800),
                    bytes: Some(file_bytes.len() as i64),
                });
            }
        };

        let upload_url = format!("https://api.cloudinary.com/v1_1/{cloud_name}/image/upload");
        let timestamp = Utc::now().timestamp().to_string();
        let folder = folder_override.unwrap_or(&config.cloudinary_folder);

        let client = reqwest::Client::new();
        let part = Part::bytes(file_bytes)
            .file_name(file_name.to_string())
            .mime_str("image/*")
            .map_err(|e| AppError::BadRequest(format!("Invalid file MIME: {e}")))?;

        let mut form = Form::new()
            .part("file", part)
            .text("timestamp", timestamp.clone())
            .text("folder", folder.to_string());

        // Authenticated signed upload (if API Key & Secret are present)
        if let (Some(api_key), Some(api_secret)) = (
            &config.cloudinary_api_key,
            &config.cloudinary_api_secret,
        ) {
            if !api_key.is_empty() && !api_secret.is_empty() && api_key != "your_api_key" {
                let params = [
                    ("folder", folder),
                    ("timestamp", &timestamp),
                ];
                let signature = Self::generate_signature(&params, api_secret);
                form = form
                    .text("api_key", api_key.clone())
                    .text("signature", signature);
            } else if let Some(preset) = &config.cloudinary_upload_preset {
                form = form.text("upload_preset", preset.clone());
            }
        } else if let Some(preset) = &config.cloudinary_upload_preset {
            form = form.text("upload_preset", preset.clone());
        }

        info!("Uploading image '{}' to Cloudinary (cloud: {})...", file_name, cloud_name);

        let response = client
            .post(&upload_url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Cloudinary HTTP request failed: {e}")))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            warn!("Cloudinary returned error: {}", error_text);
            return Err(AppError::Internal(format!(
                "Cloudinary upload rejected: {error_text}"
            )));
        }

        let upload_result: CloudinaryUploadResponse = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse Cloudinary response: {e}")))?;

        info!("Successfully uploaded to Cloudinary: {}", upload_result.secure_url);
        Ok(upload_result)
    }
}
