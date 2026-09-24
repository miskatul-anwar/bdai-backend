use axum::{extract::State, Json};
use sqlx::PgPool;

use crate::error::AppError;
use crate::middleware::CurrentUser;
use crate::models::ActivityLog;

/// GET /api/activities - List recent activity logs (requires authentication)
pub async fn list_activities(
    State(pool): State<PgPool>,
    _auth: CurrentUser,
) -> Result<Json<Vec<ActivityLog>>, AppError> {
    let logs = sqlx::query_as::<_, ActivityLog>(
        "SELECT id, action, entity, target_name, user_name, created_at
         FROM public.activity_logs
         ORDER BY created_at DESC
         LIMIT 50",
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(logs))
}
