use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::hash_password;
use crate::error::AppError;
use crate::middleware::{AdminOnly, CurrentUser};
use crate::models::{CreateUserRequest, UpdateUserRequest, User, UserResponse, UserRole};

/// GET /api/users - List all users (Accessible to authenticated users)
pub async fn list_users(
    State(pool): State<PgPool>,
    _auth: CurrentUser,
) -> Result<Json<Vec<UserResponse>>, AppError> {
    let users = sqlx::query_as::<_, User>(
        "SELECT id, name, username, email, password_hash, role, avatar, department, status, created_at, updated_at
         FROM public.users
         ORDER BY created_at ASC",
    )
    .fetch_all(&pool)
    .await?;

    let response = users.into_iter().map(UserResponse::from).collect();
    Ok(Json(response))
}

fn validate_username(username: &str) -> Result<(), AppError> {
    let trimmed = username.trim();
    if trimmed.len() < 2 || trimmed.len() > 50 {
        return Err(AppError::BadRequest(
            "Username must be between 2 and 50 characters".to_string(),
        ));
    }
    if !trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-') {
        return Err(AppError::BadRequest(
            "Username can only contain alphanumeric characters, underscores, dots, and hyphens".to_string(),
        ));
    }
    Ok(())
}

fn validate_email(email: &str) -> Result<(), AppError> {
    if email.len() < 5 || email.len() > 254 {
        return Err(AppError::BadRequest(
            "Email must be between 5 and 254 characters".to_string(),
        ));
    }
    if !email.contains('@') || !email.contains('.') {
        return Err(AppError::BadRequest("Invalid email format".to_string()));
    }
    if email.contains(char::is_whitespace) || email.contains('\0') {
        return Err(AppError::BadRequest(
            "Email contains invalid characters".to_string(),
        ));
    }
    Ok(())
}

fn validate_password(password: &str) -> Result<(), AppError> {
    if password.len() < 8 {
        return Err(AppError::BadRequest(
            "Password must be at least 8 characters long".to_string(),
        ));
    }
    if password.len() > 128 {
        return Err(AppError::BadRequest(
            "Password cannot exceed 128 characters".to_string(),
        ));
    }
    Ok(())
}

fn validate_name(name: &str) -> Result<(), AppError> {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.len() > 150 {
        return Err(AppError::BadRequest(
            "Name must be between 1 and 150 characters".to_string(),
        ));
    }
    if name.contains('\0') {
        return Err(AppError::BadRequest(
            "Name contains invalid characters".to_string(),
        ));
    }
    Ok(())
}

/// POST /api/users - Create User (STRICTLY Admin Only: Only Admin can add Admin, Moderator)
pub async fn create_user(
    State(pool): State<PgPool>,
    AdminOnly(admin_claims): AdminOnly,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    // Validate role
    let role = UserRole::from_str(&payload.role).ok_or_else(|| {
        AppError::BadRequest("Invalid role. Must be 'Admin' or 'Moderator'".to_string())
    })?;

    let email = payload.email.trim().to_lowercase();
    let name = payload.name.trim();

    validate_email(&email)?;
    validate_name(name)?;

    let username = match payload.username {
        Some(ref u) => {
            validate_username(u)?;
            Some(u.trim().to_lowercase())
        }
        None => email.split('@').next().map(|s| s.to_string()),
    };

    let raw_password = payload.password.unwrap_or_else(|| "Admin@123456".to_string());
    validate_password(&raw_password)?;
    let password_hash = hash_password(&raw_password)?;

    let avatar = payload.avatar.unwrap_or_else(|| "/team/miskat.jpg".to_string());
    let department = payload
        .department
        .unwrap_or_else(|| "Department of CSE, University of Chittagong".to_string());
    let status = payload.status.unwrap_or_else(|| "active".to_string());

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO public.users (name, username, email, password_hash, role, avatar, department, status)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING id, name, username, email, password_hash, role, avatar, department, status, created_at, updated_at",
    )
    .bind(name)
    .bind(&username)
    .bind(&email)
    .bind(&password_hash)
    .bind(role.as_str())
    .bind(&avatar)
    .bind(&department)
    .bind(&status)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.is_unique_violation() {
                return AppError::Conflict("A user with this username or email already exists".to_string());
            }
        }
        AppError::Database(e)
    })?;

    // Log admin activity
    let _ = sqlx::query(
        "INSERT INTO public.activity_logs (action, entity, target_name, user_name)
         VALUES ($1, $2, $3, $4)",
    )
    .bind("Created User")
    .bind("User")
    .bind(format!("{} ({})", user.name, user.role))
    .bind(&admin_claims.name)
    .execute(&pool)
    .await;

    Ok(Json(UserResponse::from(user)))
}

/// PUT /api/users/:id - Update User (STRICTLY Admin Only)
pub async fn update_user(
    State(pool): State<PgPool>,
    AdminOnly(admin_claims): AdminOnly,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    let existing = sqlx::query_as::<_, User>(
        "SELECT id, name, username, email, password_hash, role, avatar, department, status, created_at, updated_at
         FROM public.users
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let name = match payload.name {
        Some(n) => {
            validate_name(&n)?;
            n
        }
        None => existing.name,
    };

    let username = match payload.username {
        Some(ref u) => {
            validate_username(u)?;
            Some(u.trim().to_lowercase())
        }
        None => existing.username,
    };

    let email = match payload.email {
        Some(e) => {
            let em = e.trim().to_lowercase();
            validate_email(&em)?;
            em
        }
        None => existing.email,
    };

    let role = if let Some(r) = payload.role {
        UserRole::from_str(&r)
            .ok_or_else(|| {
                AppError::BadRequest("Invalid role. Must be 'Admin' or 'Moderator'".to_string())
            })?
            .as_str()
            .to_string()
    } else {
        existing.role
    };

    let password_hash = if let Some(pwd) = payload.password {
        if !pwd.trim().is_empty() {
            validate_password(&pwd)?;
            hash_password(&pwd)?
        } else {
            existing.password_hash
        }
    } else {
        existing.password_hash
    };

    let avatar = payload.avatar.or(existing.avatar);
    let department = payload.department.unwrap_or(existing.department);
    let status = payload.status.unwrap_or(existing.status);

    let updated = sqlx::query_as::<_, User>(
        "UPDATE public.users
         SET name = $1, username = $2, email = $3, password_hash = $4, role = $5, avatar = $6, department = $7, status = $8, updated_at = now()
         WHERE id = $9
         RETURNING id, name, username, email, password_hash, role, avatar, department, status, created_at, updated_at",
    )
    .bind(&name)
    .bind(&username)
    .bind(&email)
    .bind(&password_hash)
    .bind(&role)
    .bind(&avatar)
    .bind(&department)
    .bind(&status)
    .bind(id)
    .fetch_one(&pool)
    .await?;

    let _ = sqlx::query(
        "INSERT INTO public.activity_logs (action, entity, target_name, user_name)
         VALUES ($1, $2, $3, $4)",
    )
    .bind("Updated User")
    .bind("User")
    .bind(format!("{} ({})", updated.name, updated.role))
    .bind(&admin_claims.name)
    .execute(&pool)
    .await;

    Ok(Json(UserResponse::from(updated)))
}

/// DELETE /api/users/:id - Delete User (STRICTLY Admin Only: Only Admin can remove an Admin, Moderator)
pub async fn delete_user(
    State(pool): State<PgPool>,
    AdminOnly(admin_claims): AdminOnly,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let current_admin_uuid = admin_claims.user_uuid()?;
    if current_admin_uuid == id {
        return Err(AppError::BadRequest(
            "Cannot delete your own active administrator account".to_string(),
        ));
    }

    let user = sqlx::query_as::<_, User>(
        "SELECT id, name, username, email, password_hash, role, avatar, department, status, created_at, updated_at
         FROM public.users
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    sqlx::query("DELETE FROM public.users WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;

    let _ = sqlx::query(
        "INSERT INTO public.activity_logs (action, entity, target_name, user_name)
         VALUES ($1, $2, $3, $4)",
    )
    .bind("Deleted User")
    .bind("User")
    .bind(format!("{} ({})", user.name, user.role))
    .bind(&admin_claims.name)
    .execute(&pool)
    .await;

    Ok(Json(serde_json::json!({
        "message": format!("User {} ({}) was successfully deleted", user.name, user.role),
        "id": id,
    })))
}
