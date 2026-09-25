use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;

use crate::error::AppError;
use crate::middleware::EditorUser;
use crate::models::{CreatePartnerRequest, Partner, SiteSetting, UpdatePartnerRequest};

/// Helper to get current partners list from public.site_settings
async fn get_partners_from_db(pool: &PgPool) -> Result<Vec<Partner>, AppError> {
    let row = sqlx::query_as::<_, SiteSetting>(
        "SELECT id, data, updated_at FROM public.site_settings WHERE id = 'partners'",
    )
    .fetch_optional(pool)
    .await?;

    match row {
        Some(setting) => {
            let partners: Vec<Partner> = serde_json::from_value(setting.data).unwrap_or_default();
            Ok(partners)
        }
        None => Ok(Vec::new()),
    }
}

/// Helper to persist partners list to public.site_settings
async fn save_partners_to_db(
    pool: &PgPool,
    partners: &[Partner],
    action: &str,
    target_name: &str,
    user_name: &str,
) -> Result<(), AppError> {
    let json_val = serde_json::to_value(partners)
        .map_err(|e| AppError::Internal(format!("Failed to serialize partners: {e}")))?;

    sqlx::query(
        "INSERT INTO public.site_settings (id, data, updated_at)
         VALUES ('partners', $1, now())
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
    .bind("Partner")
    .bind(target_name)
    .bind(user_name)
    .execute(pool)
    .await;

    Ok(())
}

/// GET /api/partners - List all consortium partners and affiliates (Public)
pub async fn list_partners(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Partner>>, AppError> {
    let partners = get_partners_from_db(&pool).await?;
    Ok(Json(partners))
}

/// GET /api/partners/{id} - Get partner by ID
pub async fn get_partner(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> Result<Json<Partner>, AppError> {
    let partners = get_partners_from_db(&pool).await?;
    let partner = partners
        .into_iter()
        .find(|p| p.id.eq_ignore_ascii_case(&id))
        .ok_or_else(|| AppError::NotFound(format!("Partner '{id}' not found")))?;

    Ok(Json(partner))
}

/// POST /api/partners - Add a new consortium partner (Admin or Moderator)
pub async fn create_partner(
    State(pool): State<PgPool>,
    EditorUser(claims): EditorUser,
    Json(payload): Json<CreatePartnerRequest>,
) -> Result<(StatusCode, Json<Partner>), AppError> {
    let mut partners = get_partners_from_db(&pool).await?;

    let partner_id = payload.id.unwrap_or_else(|| {
        let slug = payload
            .name
            .to_lowercase()
            .replace(|c: char| !c.is_alphanumeric(), "-")
            .trim_matches('-')
            .to_string();
        if slug.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            slug
        }
    });

    let website_val = payload.website.or_else(|| payload.url.clone());
    let url_val = payload.url.or_else(|| website_val.clone());

    let new_partner = Partner {
        id: partner_id,
        name: payload.name,
        logo: payload.logo.unwrap_or_default(),
        description: payload.description.unwrap_or_default(),
        website: website_val,
        url: url_val,
        role: payload.role,
        partner_type: payload.partner_type,
    };

    partners.push(new_partner.clone());
    save_partners_to_db(&pool, &partners, "Created Partner", &new_partner.name, &claims.name).await?;

    Ok((StatusCode::CREATED, Json(new_partner)))
}

/// PUT /api/partners/{id} - Update a consortium partner (Admin or Moderator)
pub async fn update_partner(
    State(pool): State<PgPool>,
    EditorUser(claims): EditorUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdatePartnerRequest>,
) -> Result<Json<Partner>, AppError> {
    let mut partners = get_partners_from_db(&pool).await?;

    let pos = partners
        .iter()
        .position(|p| p.id.eq_ignore_ascii_case(&id))
        .ok_or_else(|| AppError::NotFound(format!("Partner '{id}' not found")))?;

    let current = &mut partners[pos];

    if let Some(name) = payload.name {
        current.name = name;
    }
    if let Some(logo) = payload.logo {
        current.logo = logo;
    }
    if let Some(desc) = payload.description {
        current.description = desc;
    }
    if let Some(site) = payload.website {
        current.website = Some(site.clone());
        if current.url.is_none() {
            current.url = Some(site);
        }
    }
    if let Some(url) = payload.url {
        current.url = Some(url.clone());
        if current.website.is_none() {
            current.website = Some(url);
        }
    }
    if let Some(role) = payload.role {
        current.role = Some(role);
    }
    if let Some(ptype) = payload.partner_type {
        current.partner_type = Some(ptype);
    }

    let updated = current.clone();
    save_partners_to_db(&pool, &partners, "Updated Partner", &updated.name, &claims.name).await?;

    Ok(Json(updated))
}

/// DELETE /api/partners/{id} - Delete a consortium partner (Admin or Moderator)
pub async fn delete_partner(
    State(pool): State<PgPool>,
    EditorUser(claims): EditorUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let mut partners = get_partners_from_db(&pool).await?;

    let initial_len = partners.len();
    partners.retain(|p| !p.id.eq_ignore_ascii_case(&id));

    if partners.len() == initial_len {
        return Err(AppError::NotFound(format!("Partner '{id}' not found")));
    }

    save_partners_to_db(&pool, &partners, "Deleted Partner", &id, &claims.name).await?;

    Ok(StatusCode::NO_CONTENT)
}
