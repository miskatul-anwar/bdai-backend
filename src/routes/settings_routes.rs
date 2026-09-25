use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::PgPool;
use std::collections::HashMap;

use crate::error::AppError;
use crate::middleware::EditorUser;
use crate::models::{SiteSetting, UpdateSiteSettingRequest};

/// GET /api/settings - List all site settings as a key-value map (Public)
pub async fn get_all_settings(
    State(pool): State<PgPool>,
) -> Result<Json<HashMap<String, serde_json::Value>>, AppError> {
    let rows = sqlx::query_as::<_, SiteSetting>(
        "SELECT id, data, updated_at FROM public.site_settings ORDER BY id ASC",
    )
    .fetch_all(&pool)
    .await?;

    let mut map = HashMap::new();
    for row in rows {
        map.insert(row.id, row.data);
    }

    Ok(Json(map))
}

/// GET /api/settings/:id - Get single setting by key (Public)
pub async fn get_setting(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let row = sqlx::query_as::<_, SiteSetting>(
        "SELECT id, data, updated_at FROM public.site_settings WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Setting '{}' not found", id)))?;

    Ok(Json(row.data))
}

/// PUT /api/settings/:id - Upsert site setting (Admin or Moderator)
pub async fn update_setting(
    State(pool): State<PgPool>,
    EditorUser(claims): EditorUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdateSiteSettingRequest>,
) -> Result<Json<SiteSetting>, AppError> {
    let setting = sqlx::query_as::<_, SiteSetting>(
        "INSERT INTO public.site_settings (id, data, updated_at)
         VALUES ($1, $2, now())
         ON CONFLICT (id) DO UPDATE
         SET data = EXCLUDED.data, updated_at = now()
         RETURNING id, data, updated_at",
    )
    .bind(&id)
    .bind(&payload.data)
    .fetch_one(&pool)
    .await?;

    let _ = sqlx::query(
        "INSERT INTO public.activity_logs (action, entity, target_name, user_name)
         VALUES ($1, $2, $3, $4)",
    )
    .bind("Updated Site Setting")
    .bind("SiteSetting")
    .bind(&id)
    .bind(&claims.name)
    .execute(&pool)
    .await;

    Ok(Json(setting))
}
