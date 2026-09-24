use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::error::AppError;
use crate::middleware::{AdminOnly, EditorUser};
use crate::models::{CreateVacancyRequest, UpdateVacancyRequest, Vacancy};

#[derive(Debug, Deserialize)]
pub struct VacancyQuery {
    pub notice_type: Option<String>,
    pub status: Option<String>,
}

/// GET /api/vacancies - List all vacancies / tenders / notices (Public, Parameterized against SQLi)
pub async fn list_vacancies(
    State(pool): State<PgPool>,
    Query(query): Query<VacancyQuery>,
) -> Result<Json<Vec<Vacancy>>, AppError> {
    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
        "SELECT id, title, department, work_package, notice_type, location, deadline, status, description, requirements, applicant_count, created_at, updated_at FROM public.vacancies WHERE 1=1"
    );

    if let Some(ref t) = query.notice_type {
        if t != "all" && t != "All" {
            builder.push(" AND LOWER(notice_type) = LOWER(");
            builder.push_bind(t);
            builder.push(")");
        }
    }

    if let Some(ref stat) = query.status {
        if stat != "all" {
            builder.push(" AND status = ");
            builder.push_bind(stat);
        }
    }

    builder.push(" ORDER BY deadline ASC, created_at DESC");

    let vacancies = builder
        .build_query_as::<Vacancy>()
        .fetch_all(&pool)
        .await?;

    Ok(Json(vacancies))
}

/// GET /api/vacancies/:id - Get single notice
pub async fn get_vacancy(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vacancy>, AppError> {
    let vacancy = sqlx::query_as::<_, Vacancy>(
        "SELECT id, title, department, work_package, notice_type, location, deadline, status, description, requirements, applicant_count, created_at, updated_at
         FROM public.vacancies
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Notice not found".to_string()))?;

    Ok(Json(vacancy))
}

/// POST /api/vacancies - Post Notice / Vacancy (Admin or Moderator)
pub async fn create_vacancy(
    State(pool): State<PgPool>,
    EditorUser(claims): EditorUser,
    Json(payload): Json<CreateVacancyRequest>,
) -> Result<Json<Vacancy>, AppError> {
    let title = payload.title.trim();
    let notice_type = payload.notice_type.trim();

    if title.is_empty() || notice_type.is_empty() {
        return Err(AppError::BadRequest(
            "Notice title and notice_type are required".to_string(),
        ));
    }

    let department = payload
        .department
        .unwrap_or_else(|| "Department of Computer Science and Engineering".to_string());
    let work_package = payload
        .work_package
        .unwrap_or_else(|| "HEAT-13211-CU ATF Sub-Project".to_string());
    let location = payload
        .location
        .unwrap_or_else(|| "University of Chittagong, Chattogram".to_string());
    let status = payload.status.unwrap_or_else(|| "open".to_string());
    let requirements = payload.requirements.unwrap_or_default();
    let applicant_count = payload.applicant_count.unwrap_or(0);

    let vacancy = sqlx::query_as::<_, Vacancy>(
        "INSERT INTO public.vacancies (title, department, work_package, notice_type, location, deadline, status, description, requirements, applicant_count)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         RETURNING id, title, department, work_package, notice_type, location, deadline, status, description, requirements, applicant_count, created_at, updated_at",
    )
    .bind(title)
    .bind(&department)
    .bind(&work_package)
    .bind(notice_type)
    .bind(&location)
    .bind(payload.deadline)
    .bind(&status)
    .bind(&payload.description)
    .bind(&requirements)
    .bind(applicant_count)
    .fetch_one(&pool)
    .await?;

    let _ = sqlx::query(
        "INSERT INTO public.activity_logs (action, entity, target_name, user_name)
         VALUES ($1, $2, $3, $4)",
    )
    .bind("Posted Notice")
    .bind("Vacancy / Tender")
    .bind(&vacancy.title)
    .bind(&claims.name)
    .execute(&pool)
    .await;

    Ok(Json(vacancy))
}

/// PUT /api/vacancies/:id - Update Notice (Admin or Moderator)
pub async fn update_vacancy(
    State(pool): State<PgPool>,
    EditorUser(claims): EditorUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateVacancyRequest>,
) -> Result<Json<Vacancy>, AppError> {
    let existing = sqlx::query_as::<_, Vacancy>(
        "SELECT id, title, department, work_package, notice_type, location, deadline, status, description, requirements, applicant_count, created_at, updated_at
         FROM public.vacancies
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Notice not found".to_string()))?;

    let title = payload.title.unwrap_or(existing.title);
    let department = payload.department.unwrap_or(existing.department);
    let work_package = payload.work_package.unwrap_or(existing.work_package);
    let notice_type = payload.notice_type.unwrap_or(existing.notice_type);
    let location = payload.location.unwrap_or(existing.location);
    let deadline = payload.deadline.unwrap_or(existing.deadline);
    let status = payload.status.unwrap_or(existing.status);
    let description = payload.description.unwrap_or(existing.description);
    let requirements = payload.requirements.unwrap_or(existing.requirements);
    let applicant_count = payload.applicant_count.unwrap_or(existing.applicant_count);

    let updated = sqlx::query_as::<_, Vacancy>(
        "UPDATE public.vacancies
         SET title = $1, department = $2, work_package = $3, notice_type = $4, location = $5, deadline = $6, status = $7, description = $8, requirements = $9, applicant_count = $10, updated_at = now()
         WHERE id = $11
         RETURNING id, title, department, work_package, notice_type, location, deadline, status, description, requirements, applicant_count, created_at, updated_at",
    )
    .bind(&title)
    .bind(&department)
    .bind(&work_package)
    .bind(&notice_type)
    .bind(&location)
    .bind(deadline)
    .bind(&status)
    .bind(&description)
    .bind(&requirements)
    .bind(applicant_count)
    .bind(id)
    .fetch_one(&pool)
    .await?;

    let _ = sqlx::query(
        "INSERT INTO public.activity_logs (action, entity, target_name, user_name)
         VALUES ($1, $2, $3, $4)",
    )
    .bind("Updated Notice")
    .bind("Vacancy / Tender")
    .bind(&updated.title)
    .bind(&claims.name)
    .execute(&pool)
    .await;

    Ok(Json(updated))
}

/// DELETE /api/vacancies/:id - Delete Notice (STRICTLY Admin Only)
pub async fn delete_vacancy(
    State(pool): State<PgPool>,
    AdminOnly(admin_claims): AdminOnly,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let vac = sqlx::query_as::<_, Vacancy>(
        "SELECT id, title, department, work_package, notice_type, location, deadline, status, description, requirements, applicant_count, created_at, updated_at
         FROM public.vacancies
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Notice not found".to_string()))?;

    sqlx::query("DELETE FROM public.vacancies WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;

    let _ = sqlx::query(
        "INSERT INTO public.activity_logs (action, entity, target_name, user_name)
         VALUES ($1, $2, $3, $4)",
    )
    .bind("Deleted Notice")
    .bind("Vacancy / Tender")
    .bind(&vac.title)
    .bind(&admin_claims.name)
    .execute(&pool)
    .await;

    Ok(Json(serde_json::json!({
        "message": format!("Notice '{}' was deleted successfully", vac.title),
        "id": id,
    })))
}
