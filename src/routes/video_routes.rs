use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;

use crate::error::AppError;
use crate::middleware::EditorUser;
use crate::models::{CreateVideoRequest, SiteSetting, UpdateVideoRequest, Video};

/// Helper to get current videos list from public.site_settings
async fn get_videos_from_db(pool: &PgPool) -> Result<Vec<Video>, AppError> {
    let row = sqlx::query_as::<_, SiteSetting>(
        "SELECT id, data, updated_at FROM public.site_settings WHERE id = 'videos'",
    )
    .fetch_optional(pool)
    .await?;

    match row {
        Some(setting) => {
            let mut videos: Vec<Video> = serde_json::from_value(setting.data).unwrap_or_default();
            videos.sort_by_key(|v| v.order);
            Ok(videos)
        }
        None => Ok(Vec::new()),
    }
}

/// Helper to persist videos list to public.site_settings
async fn save_videos_to_db(
    pool: &PgPool,
    videos: &[Video],
    action: &str,
    target_name: &str,
    user_name: &str,
) -> Result<(), AppError> {
    let json_val = serde_json::to_value(videos)
        .map_err(|e| AppError::Internal(format!("Failed to serialize videos: {e}")))?;

    sqlx::query(
        "INSERT INTO public.site_settings (id, data, updated_at)
         VALUES ('videos', $1, now())
         ON CONFLICT (id) DO UPDATE
         SET data = EXCLUDED.data, updated_at = now()",
    )
    .bind(&json_val)
    .execute(pool)
    .await?;

    let _ = sqlx::query(
        "INSERT INTO public.activity_logs (action, entity, target_name, user_name)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(action)
    .bind("Video")
    .bind(target_name)
    .bind(user_name)
    .execute(pool)
    .await;

    Ok(())
}

/// GET /api/videos - List all videos
pub async fn list_videos(State(pool): State<PgPool>) -> Result<Json<Vec<Video>>, AppError> {
    let videos = get_videos_from_db(&pool).await?;
    Ok(Json(videos))
}

/// GET /api/videos/:id - Get a single video by ID
pub async fn get_video(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> Result<Json<Video>, AppError> {
    let videos = get_videos_from_db(&pool).await?;
    let video = videos
        .into_iter()
        .find(|v| v.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Video with ID '{id}' not found")))?;

    Ok(Json(video))
}

/// POST /api/videos - Create a new video entry (EditorUser)
pub async fn create_video(
    State(pool): State<PgPool>,
    EditorUser(claims): EditorUser,
    Json(payload): Json<CreateVideoRequest>,
) -> Result<(StatusCode, Json<Video>), AppError> {
    let mut videos = get_videos_from_db(&pool).await?;

    let id = payload
        .id
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| {
            // Check if YouTube video ID can be extracted
            if let Some(pos) = payload.url.find("v=") {
                let sub = &payload.url[pos + 2..];
                let vid_id: String = sub.chars().take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_').collect();
                if !vid_id.is_empty() {
                    return vid_id;
                }
            }
            payload
                .title
                .to_lowercase()
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { '-' })
                .collect::<String>()
                .trim_matches('-')
                .to_string()
        });

    if videos.iter().any(|v| v.id == id) {
        return Err(AppError::Conflict(format!(
            "Video with ID '{id}' already exists"
        )));
    }

    let order = payload.order.unwrap_or((videos.len() as i32) + 1);

    let new_video = Video {
        id: id.clone(),
        title: payload.title,
        url: payload.url,
        thumbnail: payload.thumbnail,
        description: payload.description,
        posted_at: payload.posted_at,
        order,
        created_at: Some(chrono::Utc::now()),
        updated_at: Some(chrono::Utc::now()),
    };

    videos.push(new_video.clone());
    videos.sort_by_key(|v| v.order);

    save_videos_to_db(
        &pool,
        &videos,
        "Created Video",
        &new_video.title,
        &claims.name,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(new_video)))
}

/// PUT /api/videos/:id - Update an existing video (EditorUser)
pub async fn update_video(
    State(pool): State<PgPool>,
    EditorUser(claims): EditorUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdateVideoRequest>,
) -> Result<Json<Video>, AppError> {
    let mut videos = get_videos_from_db(&pool).await?;

    let idx = videos
        .iter()
        .position(|v| v.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Video with ID '{id}' not found")))?;

    let current = &mut videos[idx];

    if let Some(title) = payload.title {
        current.title = title;
    }
    if let Some(url) = payload.url {
        current.url = url;
    }
    if let Some(thumbnail) = payload.thumbnail {
        current.thumbnail = Some(thumbnail);
    }
    if let Some(description) = payload.description {
        current.description = Some(description);
    }
    if let Some(posted_at) = payload.posted_at {
        current.posted_at = Some(posted_at);
    }
    if let Some(order) = payload.order {
        current.order = order;
    }
    current.updated_at = Some(chrono::Utc::now());

    let updated_video = current.clone();
    videos.sort_by_key(|v| v.order);

    save_videos_to_db(
        &pool,
        &videos,
        "Updated Video",
        &updated_video.title,
        &claims.name,
    )
    .await?;

    Ok(Json(updated_video))
}

/// DELETE /api/videos/:id - Remove a video (EditorUser)
pub async fn delete_video(
    State(pool): State<PgPool>,
    EditorUser(claims): EditorUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let mut videos = get_videos_from_db(&pool).await?;

    let idx = videos
        .iter()
        .position(|v| v.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Video with ID '{id}' not found")))?;

    let removed = videos.remove(idx);

    save_videos_to_db(
        &pool,
        &videos,
        "Deleted Video",
        &removed.title,
        &claims.name,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}
