use axum::{
    extract::{Multipart, State},
    Json,
};
use chrono::Utc;

use crate::cloudinary::{CloudinaryService, CloudinaryUploadResponse, SignatureResponse};
use crate::config::AppConfig;
use crate::error::AppError;
use crate::middleware::CurrentUser;

/// POST /api/upload - Upload an image to Cloudinary (Multipart form data)
pub async fn upload_image(
    State(config): State<AppConfig>,
    _auth: CurrentUser, // Requires authentication
    mut multipart: Multipart,
) -> Result<Json<CloudinaryUploadResponse>, AppError> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut file_name = "upload.jpg".to_string();
    let mut folder: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to read multipart field: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" || name == "image" {
            if let Some(original_name) = field.file_name() {
                file_name = original_name.to_string();
            }
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(format!("Failed to read image bytes: {e}")))?;
            file_bytes = Some(data.to_vec());
        } else if name == "folder" {
            let val = field.text().await.unwrap_or_default();
            if !val.trim().is_empty() {
                folder = Some(val.trim().to_string());
            }
        }
    }

    let bytes = file_bytes.ok_or_else(|| {
        AppError::BadRequest("No image file provided in multipart payload".to_string())
    })?;

    if bytes.is_empty() {
        return Err(AppError::BadRequest("Uploaded file is empty".to_string()));
    }

    // Limit image size to 15MB
    if bytes.len() > 15 * 1024 * 1024 {
        return Err(AppError::BadRequest(
            "File size exceeds maximum allowed limit (15MB)".to_string(),
        ));
    }

    let response = CloudinaryService::upload_image(
        &config,
        bytes,
        &file_name,
        folder.as_deref(),
    )
    .await?;

    Ok(Json(response))
}

/// GET /api/upload/signature - Get signed Cloudinary parameters for direct client-side upload
pub async fn get_upload_signature(
    State(config): State<AppConfig>,
    _auth: CurrentUser,
) -> Result<Json<SignatureResponse>, AppError> {
    let cloud_name = config
        .cloudinary_cloud_name
        .clone()
        .unwrap_or_else(|| "demo".to_string());
    let api_key = config
        .cloudinary_api_key
        .clone()
        .unwrap_or_default();
    let api_secret = config
        .cloudinary_api_secret
        .clone()
        .unwrap_or_default();

    let timestamp = Utc::now().timestamp();
    let folder = config.cloudinary_folder.clone();

    let params = [
        ("folder", folder.as_str()),
        ("timestamp", &timestamp.to_string()),
    ];

    let signature = CloudinaryService::generate_signature(&params, &api_secret);

    Ok(Json(SignatureResponse {
        signature,
        timestamp,
        api_key,
        cloud_name,
        folder,
    }))
}
