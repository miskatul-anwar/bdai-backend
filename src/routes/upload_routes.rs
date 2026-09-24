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

    // 1. Limit image size to 15MB (Denial of Service protection)
    if bytes.len() > 15 * 1024 * 1024 {
        return Err(AppError::BadRequest(
            "File size exceeds maximum allowed limit (15MB)".to_string(),
        ));
    }

    // 2. Strict Magic Byte / File Signature Validation (prevents polyglot & XSS uploads)
    let is_jpeg = bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF;
    let is_png = bytes.len() >= 8 && &bytes[0..8] == b"\x89PNG\r\n\x1a\n";
    let is_gif = bytes.len() >= 6 && (&bytes[0..6] == b"GIF87a" || &bytes[0..6] == b"GIF89a");
    let is_webp = bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP";
    let is_avif = bytes.len() >= 12 && &bytes[4..8] == b"ftyp" && (&bytes[8..12] == b"avif" || &bytes[8..12] == b"avis");

    if !is_jpeg && !is_png && !is_gif && !is_webp && !is_avif {
        return Err(AppError::BadRequest(
            "Security check failed: File header does not match a valid image format. Only JPEG, PNG, WebP, GIF, and AVIF image formats are allowed (SVG and executable formats are prohibited)."
                .to_string(),
        ));
    }

    // 3. Path Traversal & Filename Sanitization
    let base_name = std::path::Path::new(&file_name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("upload.jpg");

    let sanitized_name: String = base_name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect();

    let safe_file_name = if sanitized_name.is_empty() {
        "upload.jpg".to_string()
    } else {
        sanitized_name
    };

    // 4. File extension check against whitelist
    let ext = safe_file_name
        .rsplit('.')
        .next()
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    let allowed_extensions = ["jpg", "jpeg", "png", "webp", "gif", "avif"];
    if !allowed_extensions.contains(&ext.as_str()) {
        return Err(AppError::BadRequest(
            "Disallowed file extension. Permitted extensions: jpg, jpeg, png, webp, gif, avif"
                .to_string(),
        ));
    }

    // 5. Sanitize target folder
    let sanitized_folder = folder.map(|f| {
        f.chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .collect::<String>()
    });

    let response = CloudinaryService::upload_image(
        &config,
        bytes,
        &safe_file_name,
        sanitized_folder.as_deref(),
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
