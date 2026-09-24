use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::middleware::{AdminOnly, EditorUser};
use crate::models::{CreateTeamMemberRequest, TeamMember, UpdateTeamMemberRequest};

/// GET /api/team - List all personnel / employees (Public endpoint)
pub async fn list_team(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<TeamMember>>, AppError> {
    let team = sqlx::query_as::<_, TeamMember>(
        "SELECT id, name, designation, role, category, institution, email, bio, image, scholar_url, linkedin_url, display_order, created_at, updated_at
         FROM public.team_members
         ORDER BY display_order ASC, created_at ASC",
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(team))
}

/// GET /api/team/:id - Get single team member
pub async fn get_team_member(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<TeamMember>, AppError> {
    let member = sqlx::query_as::<_, TeamMember>(
        "SELECT id, name, designation, role, category, institution, email, bio, image, scholar_url, linkedin_url, display_order, created_at, updated_at
         FROM public.team_members
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Team member not found".to_string()))?;

    Ok(Json(member))
}

fn validate_safe_url(url_str: &str, field_name: &str) -> Result<(), AppError> {
    let trimmed = url_str.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    if !trimmed.starts_with("https://") && !trimmed.starts_with("http://") {
        return Err(AppError::BadRequest(format!(
            "Security violation: {field_name} must begin with 'https://' or 'http://' (untrusted URI schemes prohibited)."
        )));
    }
    if trimmed.len() > 1000 {
        return Err(AppError::BadRequest(format!(
            "{field_name} cannot exceed 1000 characters."
        )));
    }
    Ok(())
}

/// POST /api/team - Add Employee (STRICTLY Admin Only: Only Admin can add all kinds of employees)
pub async fn create_team_member(
    State(pool): State<PgPool>,
    AdminOnly(admin_claims): AdminOnly,
    Json(payload): Json<CreateTeamMemberRequest>,
) -> Result<Json<TeamMember>, AppError> {
    let name = payload.name.trim();
    let designation = payload.designation.trim();

    if name.is_empty() || designation.is_empty() {
        return Err(AppError::BadRequest(
            "Employee name and designation are required".to_string(),
        ));
    }

    if name.len() > 150 {
        return Err(AppError::BadRequest("Employee name cannot exceed 150 characters".to_string()));
    }

    if designation.len() > 150 {
        return Err(AppError::BadRequest("Designation cannot exceed 150 characters".to_string()));
    }

    if let Some(ref s_url) = payload.scholar_url {
        validate_safe_url(s_url, "Google Scholar URL")?;
    }

    if let Some(ref l_url) = payload.linkedin_url {
        validate_safe_url(l_url, "LinkedIn URL")?;
    }

    let institution = payload
        .institution
        .unwrap_or_else(|| "Department of Computer Science and Engineering, University of Chittagong".to_string());
    let image = payload.image.unwrap_or_else(|| "/team/miskat.jpg".to_string());
    let display_order = payload.display_order.unwrap_or(0);

    let member = sqlx::query_as::<_, TeamMember>(
        "INSERT INTO public.team_members (name, designation, role, category, institution, email, bio, image, scholar_url, linkedin_url, display_order)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
         RETURNING id, name, designation, role, category, institution, email, bio, image, scholar_url, linkedin_url, display_order, created_at, updated_at",
    )
    .bind(name)
    .bind(designation)
    .bind(payload.role)
    .bind(payload.category)
    .bind(&institution)
    .bind(payload.email)
    .bind(payload.bio)
    .bind(&image)
    .bind(payload.scholar_url)
    .bind(payload.linkedin_url)
    .bind(display_order)
    .fetch_one(&pool)
    .await?;

    let _ = sqlx::query(
        "INSERT INTO public.activity_logs (action, entity, target_name, user_name)
         VALUES ($1, $2, $3, $4)",
    )
    .bind("Added Employee")
    .bind("Team Member")
    .bind(format!("{} ({})", member.name, member.designation))
    .bind(&admin_claims.name)
    .execute(&pool)
    .await;

    Ok(Json(member))
}

/// PUT /api/team/:id - Update Employee Details (Admin or Moderator)
pub async fn update_team_member(
    State(pool): State<PgPool>,
    EditorUser(claims): EditorUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTeamMemberRequest>,
) -> Result<Json<TeamMember>, AppError> {
    let existing = sqlx::query_as::<_, TeamMember>(
        "SELECT id, name, designation, role, category, institution, email, bio, image, scholar_url, linkedin_url, display_order, created_at, updated_at
         FROM public.team_members
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Team member not found".to_string()))?;

    if let Some(ref s_url) = payload.scholar_url {
        validate_safe_url(s_url, "Google Scholar URL")?;
    }
    if let Some(ref l_url) = payload.linkedin_url {
        validate_safe_url(l_url, "LinkedIn URL")?;
    }
    if let Some(ref n) = payload.name {
        if n.len() > 150 {
            return Err(AppError::BadRequest("Employee name cannot exceed 150 characters".to_string()));
        }
    }
    if let Some(ref d) = payload.designation {
        if d.len() > 150 {
            return Err(AppError::BadRequest("Designation cannot exceed 150 characters".to_string()));
        }
    }

    let name = payload.name.unwrap_or(existing.name);
    let designation = payload.designation.unwrap_or(existing.designation);
    let role = payload.role.or(existing.role);
    let category = payload.category.or(existing.category);
    let institution = payload.institution.unwrap_or(existing.institution);
    let email = payload.email.or(existing.email);
    let bio = payload.bio.or(existing.bio);
    let image = payload.image.or(existing.image);
    let scholar_url = payload.scholar_url.or(existing.scholar_url);
    let linkedin_url = payload.linkedin_url.or(existing.linkedin_url);
    let display_order = payload.display_order.unwrap_or(existing.display_order);

    let updated = sqlx::query_as::<_, TeamMember>(
        "UPDATE public.team_members
         SET name = $1, designation = $2, role = $3, category = $4, institution = $5, email = $6, bio = $7, image = $8, scholar_url = $9, linkedin_url = $10, display_order = $11, updated_at = now()
         WHERE id = $12
         RETURNING id, name, designation, role, category, institution, email, bio, image, scholar_url, linkedin_url, display_order, created_at, updated_at",
    )
    .bind(&name)
    .bind(&designation)
    .bind(&role)
    .bind(&category)
    .bind(&institution)
    .bind(&email)
    .bind(&bio)
    .bind(&image)
    .bind(&scholar_url)
    .bind(&linkedin_url)
    .bind(display_order)
    .bind(id)
    .fetch_one(&pool)
    .await?;

    let _ = sqlx::query(
        "INSERT INTO public.activity_logs (action, entity, target_name, user_name)
         VALUES ($1, $2, $3, $4)",
    )
    .bind("Updated Employee")
    .bind("Team Member")
    .bind(&updated.name)
    .bind(&claims.name)
    .execute(&pool)
    .await;

    Ok(Json(updated))
}

/// DELETE /api/team/:id - Remove Employee (STRICTLY Admin Only: Only Admin can remove all kinds of employees)
pub async fn delete_team_member(
    State(pool): State<PgPool>,
    AdminOnly(admin_claims): AdminOnly,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let member = sqlx::query_as::<_, TeamMember>(
        "SELECT id, name, designation, role, category, institution, email, bio, image, scholar_url, linkedin_url, display_order, created_at, updated_at
         FROM public.team_members
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Team member not found".to_string()))?;

    sqlx::query("DELETE FROM public.team_members WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;

    let _ = sqlx::query(
        "INSERT INTO public.activity_logs (action, entity, target_name, user_name)
         VALUES ($1, $2, $3, $4)",
    )
    .bind("Removed Employee")
    .bind("Team Member")
    .bind(format!("{} ({})", member.name, member.designation))
    .bind(&admin_claims.name)
    .execute(&pool)
    .await;

    Ok(Json(serde_json::json!({
        "message": format!("Employee {} was removed from team roster", member.name),
        "id": id,
    })))
}
