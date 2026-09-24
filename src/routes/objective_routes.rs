use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::PgPool;

use crate::error::AppError;
use crate::middleware::EditorUser;
use crate::models::{ResearchObjective, UpdateObjectiveRequest};

/// GET /api/objectives - List all research objectives (Public)
pub async fn list_objectives(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<ResearchObjective>>, AppError> {
    let objectives = sqlx::query_as::<_, ResearchObjective>(
        "SELECT id, title, details, researcher, sector, status, progress, deliverables, created_at, updated_at
         FROM public.research_objectives
         ORDER BY id ASC",
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(objectives))
}

/// PUT /api/objectives/:id - Update Research Objective (Admin or Moderator)
pub async fn update_objective(
    State(pool): State<PgPool>,
    EditorUser(claims): EditorUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdateObjectiveRequest>,
) -> Result<Json<ResearchObjective>, AppError> {
    let existing = sqlx::query_as::<_, ResearchObjective>(
        "SELECT id, title, details, researcher, sector, status, progress, deliverables, created_at, updated_at
         FROM public.research_objectives
         WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Research objective not found".to_string()))?;

    let title = payload.title.unwrap_or(existing.title);
    let details = payload.details.unwrap_or(existing.details);
    let researcher = payload.researcher.unwrap_or(existing.researcher);
    let sector = payload.sector.unwrap_or(existing.sector);
    let status = payload.status.unwrap_or(existing.status);
    let progress = payload.progress.unwrap_or(existing.progress);
    let deliverables = payload.deliverables.unwrap_or(existing.deliverables);

    let updated = sqlx::query_as::<_, ResearchObjective>(
        "UPDATE public.research_objectives
         SET title = $1, details = $2, researcher = $3, sector = $4, status = $5, progress = $6, deliverables = $7, updated_at = now()
         WHERE id = $8
         RETURNING id, title, details, researcher, sector, status, progress, deliverables, created_at, updated_at",
    )
    .bind(&title)
    .bind(&details)
    .bind(&researcher)
    .bind(&sector)
    .bind(&status)
    .bind(progress)
    .bind(deliverables)
    .bind(&id)
    .fetch_one(&pool)
    .await?;

    let _ = sqlx::query(
        "INSERT INTO public.activity_logs (action, entity, target_name, user_name)
         VALUES ($1, $2, $3, $4)",
    )
    .bind("Updated Milestone")
    .bind("Objective")
    .bind(format!("{}: {}%", updated.id, updated.progress))
    .bind(&claims.name)
    .execute(&pool)
    .await;

    Ok(Json(updated))
}

/// POST /api/objectives - Create new Research Objective (Admin Only)
pub async fn create_objective(
    State(pool): State<PgPool>,
    crate::middleware::AdminOnly(claims): crate::middleware::AdminOnly,
    Json(payload): Json<crate::models::CreateObjectiveRequest>,
) -> Result<Json<ResearchObjective>, AppError> {
    let status = payload.status.unwrap_or_else(|| "in-progress".to_string());
    let progress = payload.progress.unwrap_or(0);
    let deliverables = payload.deliverables.unwrap_or(1);

    let created = sqlx::query_as::<_, ResearchObjective>(
        "INSERT INTO public.research_objectives (id, title, details, researcher, sector, status, progress, deliverables)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING id, title, details, researcher, sector, status, progress, deliverables, created_at, updated_at",
    )
    .bind(&payload.id)
    .bind(&payload.title)
    .bind(&payload.details)
    .bind(&payload.researcher)
    .bind(&payload.sector)
    .bind(&status)
    .bind(progress)
    .bind(deliverables)
    .fetch_one(&pool)
    .await?;

    let _ = sqlx::query(
        "INSERT INTO public.activity_logs (action, entity, target_name, user_name)
         VALUES ($1, $2, $3, $4)",
    )
    .bind("Created Milestone")
    .bind("Objective")
    .bind(&created.id)
    .bind(&claims.name)
    .execute(&pool)
    .await;

    Ok(Json(created))
}

/// DELETE /api/objectives/:id - Remove Research Objective (Admin Only)
pub async fn delete_objective(
    State(pool): State<PgPool>,
    crate::middleware::AdminOnly(claims): crate::middleware::AdminOnly,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let res = sqlx::query("DELETE FROM public.research_objectives WHERE id = $1")
        .bind(&id)
        .execute(&pool)
        .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("Research objective not found".to_string()));
    }

    let _ = sqlx::query(
        "INSERT INTO public.activity_logs (action, entity, target_name, user_name)
         VALUES ($1, $2, $3, $4)",
    )
    .bind("Deleted Milestone")
    .bind("Objective")
    .bind(&id)
    .bind(&claims.name)
    .execute(&pool)
    .await;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Research objective '{}' deleted", id)
    })))
}

