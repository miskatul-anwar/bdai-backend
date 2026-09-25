use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;

use crate::error::AppError;
use crate::middleware::EditorUser;
use crate::models::{CreateToolRequest, Tool, SiteSetting, UpdateToolRequest};

/// Helper to get current tools list from public.site_settings
async fn get_tools_from_db(pool: &PgPool) -> Result<Vec<Tool>, AppError> {
    let row = sqlx::query_as::<_, SiteSetting>(
        "SELECT id, data, updated_at FROM public.site_settings WHERE id = 'tools'",
    )
    .fetch_optional(pool)
    .await?;

    match row {
        Some(setting) => {
            let mut tools: Vec<Tool> = serde_json::from_value(setting.data).unwrap_or_default();
            tools.sort_by_key(|t| t.display_order);
            Ok(tools)
        }
        None => Ok(Vec::new()),
    }
}

/// Helper to persist tools list to public.site_settings
async fn save_tools_to_db(
    pool: &PgPool,
    tools: &[Tool],
    action: &str,
    target_name: &str,
    user_name: &str,
) -> Result<(), AppError> {
    let json_val = serde_json::to_value(tools)
        .map_err(|e| AppError::Internal(format!("Failed to serialize tools: {e}")))?;

    sqlx::query(
        "INSERT INTO public.site_settings (id, data, updated_at)
         VALUES ('tools', $1, now())
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
    .bind("Tool")
    .bind(target_name)
    .bind(user_name)
    .execute(pool)
    .await;

    Ok(())
}

/// GET /api/tools - List all showcase tools (Public)
pub async fn list_tools(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Tool>>, AppError> {
    let tools = get_tools_from_db(&pool).await?;
    Ok(Json(tools))
}

/// GET /api/tools/{id} - Get single tool by ID
pub async fn get_tool(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> Result<Json<Tool>, AppError> {
    let tools = get_tools_from_db(&pool).await?;
    let tool = tools
        .into_iter()
        .find(|t| t.id.eq_ignore_ascii_case(&id))
        .ok_or_else(|| AppError::NotFound(format!("Tool '{id}' not found")))?;

    Ok(Json(tool))
}

/// POST /api/tools - Add a new showcase tool (Admin or Moderator)
pub async fn create_tool(
    State(pool): State<PgPool>,
    EditorUser(claims): EditorUser,
    Json(payload): Json<CreateToolRequest>,
) -> Result<(StatusCode, Json<Tool>), AppError> {
    let mut tools = get_tools_from_db(&pool).await?;

    let tool_id = payload.id.unwrap_or_else(|| {
        let slug = payload
            .title
            .to_lowercase()
            .replace(|c: char| !c.is_alphanumeric(), "-")
            .trim_matches('-')
            .to_string();
        if slug.is_empty() {
            format!("tool-{}", uuid::Uuid::new_v4().to_string().chars().take(8).collect::<String>())
        } else {
            slug
        }
    });

    if tools.iter().any(|t| t.id.eq_ignore_ascii_case(&tool_id)) {
        return Err(AppError::BadRequest(format!(
            "Tool with ID '{tool_id}' already exists."
        )));
    }

    let now_str = chrono::Utc::now().to_rfc3339();
    let display_order = payload.display_order.unwrap_or((tools.len() as i32) + 1);

    let new_tool = Tool {
        id: tool_id,
        title: payload.title.trim().to_string(),
        subtitle: payload.subtitle.map(|s| s.trim().to_string()),
        description: payload.description.unwrap_or_default().trim().to_string(),
        abstract_text: payload.abstract_text.map(|s| s.trim().to_string()),
        paper_url: payload.paper_url.map(|s| s.trim().to_string()),
        source_url: payload.source_url.map(|s| s.trim().to_string()),
        platform_url: payload.platform_url.map(|s| s.trim().to_string()),
        video_url: payload.video_url.map(|s| s.trim().to_string()),
        image_url: payload.image_url.map(|s| s.trim().to_string()),
        authors: payload.authors.map(|s| s.trim().to_string()),
        features: payload.features.unwrap_or_default(),
        display_order,
        badge: payload.badge.map(|s| s.trim().to_string()),
        created_at: Some(now_str.clone()),
        updated_at: Some(now_str),
    };

    tools.push(new_tool.clone());
    tools.sort_by_key(|t| t.display_order);

    save_tools_to_db(&pool, &tools, "Added Tool", &new_tool.title, &claims.name).await?;

    Ok((StatusCode::CREATED, Json(new_tool)))
}

/// PUT /api/tools/{id} - Update an existing tool (Admin or Moderator)
pub async fn update_tool(
    State(pool): State<PgPool>,
    EditorUser(claims): EditorUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdateToolRequest>,
) -> Result<Json<Tool>, AppError> {
    let mut tools = get_tools_from_db(&pool).await?;

    let index = tools
        .iter()
        .position(|t| t.id.eq_ignore_ascii_case(&id))
        .ok_or_else(|| AppError::NotFound(format!("Tool '{id}' not found")))?;

    let now_str = chrono::Utc::now().to_rfc3339();

    if let Some(title) = payload.title {
        tools[index].title = title.trim().to_string();
    }
    if let Some(subtitle) = payload.subtitle {
        tools[index].subtitle = Some(subtitle.trim().to_string());
    }
    if let Some(desc) = payload.description {
        tools[index].description = desc.trim().to_string();
    }
    if let Some(abs) = payload.abstract_text {
        tools[index].abstract_text = Some(abs.trim().to_string());
    }
    if let Some(paper) = payload.paper_url {
        tools[index].paper_url = Some(paper.trim().to_string());
    }
    if let Some(source) = payload.source_url {
        tools[index].source_url = Some(source.trim().to_string());
    }
    if let Some(platform) = payload.platform_url {
        tools[index].platform_url = Some(platform.trim().to_string());
    }
    if let Some(video) = payload.video_url {
        tools[index].video_url = Some(video.trim().to_string());
    }
    if let Some(image) = payload.image_url {
        tools[index].image_url = Some(image.trim().to_string());
    }
    if let Some(authors) = payload.authors {
        tools[index].authors = Some(authors.trim().to_string());
    }
    if let Some(features) = payload.features {
        tools[index].features = features;
    }
    if let Some(order) = payload.display_order {
        tools[index].display_order = order;
    }
    if let Some(badge) = payload.badge {
        tools[index].badge = Some(badge.trim().to_string());
    }
    tools[index].updated_at = Some(now_str);

    tools.sort_by_key(|t| t.display_order);

    let updated = tools
        .iter()
        .find(|t| t.id.eq_ignore_ascii_case(&id))
        .cloned()
        .unwrap();

    save_tools_to_db(&pool, &tools, "Updated Tool", &updated.title, &claims.name).await?;

    Ok(Json(updated))
}

/// DELETE /api/tools/{id} - Delete a tool (Admin or Moderator)
pub async fn delete_tool(
    State(pool): State<PgPool>,
    EditorUser(claims): EditorUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let mut tools = get_tools_from_db(&pool).await?;

    let index = tools
        .iter()
        .position(|t| t.id.eq_ignore_ascii_case(&id))
        .ok_or_else(|| AppError::NotFound(format!("Tool '{id}' not found")))?;

    let removed = tools.remove(index);

    save_tools_to_db(&pool, &tools, "Deleted Tool", &removed.title, &claims.name).await?;

    Ok(StatusCode::NO_CONTENT)
}
