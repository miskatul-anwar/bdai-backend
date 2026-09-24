use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::error::AppError;
use crate::middleware::{AdminOnly, EditorUser};
use crate::models::{CreateNewsRequest, NewsArticle, UpdateNewsRequest};

#[derive(Debug, Deserialize)]
pub struct NewsQuery {
    pub category: Option<String>,
    pub status: Option<String>,
    pub featured: Option<bool>,
}

/// GET /api/news - List news articles (Public, Parameterized against SQLi)
pub async fn list_news(
    State(pool): State<PgPool>,
    Query(query): Query<NewsQuery>,
) -> Result<Json<Vec<NewsArticle>>, AppError> {
    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
        "SELECT id, title, slug, excerpt, content, category, publish_date, author, status, featured, tags, created_at, updated_at FROM public.news_articles WHERE 1=1"
    );

    if let Some(ref cat) = query.category {
        if cat != "all" {
            builder.push(" AND category = ");
            builder.push_bind(cat);
        }
    }

    if let Some(ref stat) = query.status {
        if stat != "all" {
            builder.push(" AND status = ");
            builder.push_bind(stat);
        }
    }

    if let Some(feat) = query.featured {
        builder.push(" AND featured = ");
        builder.push_bind(feat);
    }

    builder.push(" ORDER BY publish_date DESC, created_at DESC");

    let articles = builder
        .build_query_as::<NewsArticle>()
        .fetch_all(&pool)
        .await?;

    Ok(Json(articles))
}

/// GET /api/news/:slug - Get single news article by slug (Public)
pub async fn get_news_by_slug(
    State(pool): State<PgPool>,
    Path(slug): Path<String>,
) -> Result<Json<NewsArticle>, AppError> {
    let article = sqlx::query_as::<_, NewsArticle>(
        "SELECT id, title, slug, excerpt, content, category, publish_date, author, status, featured, tags, created_at, updated_at
         FROM public.news_articles
         WHERE slug = $1",
    )
    .bind(slug)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("News article not found".to_string()))?;

    Ok(Json(article))
}

/// POST /api/news - Create News (Admin or Moderator)
pub async fn create_news(
    State(pool): State<PgPool>,
    EditorUser(claims): EditorUser,
    Json(payload): Json<CreateNewsRequest>,
) -> Result<Json<NewsArticle>, AppError> {
    let title = payload.title.trim();
    if title.is_empty() {
        return Err(AppError::BadRequest("Article title is required".to_string()));
    }

    let slug = payload.slug.unwrap_or_else(|| {
        title
            .to_lowercase()
            .replace(|c: char| !c.is_alphanumeric(), "-")
            .trim_matches('-')
            .to_string()
    });

    let publish_date = payload
        .publish_date
        .unwrap_or_else(|| Utc::now().date_naive());
    let author = payload.author.unwrap_or_else(|| claims.name.clone());
    let status = payload.status.unwrap_or_else(|| "published".to_string());
    let featured = payload.featured.unwrap_or(false);
    let tags = payload.tags.unwrap_or_default();

    let article = sqlx::query_as::<_, NewsArticle>(
        "INSERT INTO public.news_articles (title, slug, excerpt, content, category, publish_date, author, status, featured, tags)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         RETURNING id, title, slug, excerpt, content, category, publish_date, author, status, featured, tags, created_at, updated_at",
    )
    .bind(title)
    .bind(&slug)
    .bind(&payload.excerpt)
    .bind(&payload.content)
    .bind(&payload.category)
    .bind(publish_date)
    .bind(&author)
    .bind(&status)
    .bind(featured)
    .bind(&tags)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.is_unique_violation() {
                return AppError::Conflict("An article with this slug already exists".to_string());
            }
        }
        AppError::Database(e)
    })?;

    let _ = sqlx::query(
        "INSERT INTO public.activity_logs (action, entity, target_name, user_name)
         VALUES ($1, $2, $3, $4)",
    )
    .bind("Published Article")
    .bind("News")
    .bind(&article.title)
    .bind(&claims.name)
    .execute(&pool)
    .await;

    Ok(Json(article))
}

/// PUT /api/news/:id - Update News Article (Admin or Moderator)
pub async fn update_news(
    State(pool): State<PgPool>,
    EditorUser(claims): EditorUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateNewsRequest>,
) -> Result<Json<NewsArticle>, AppError> {
    let existing = sqlx::query_as::<_, NewsArticle>(
        "SELECT id, title, slug, excerpt, content, category, publish_date, author, status, featured, tags, created_at, updated_at
         FROM public.news_articles
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Article not found".to_string()))?;

    let title = payload.title.unwrap_or(existing.title);
    let slug = payload.slug.unwrap_or(existing.slug);
    let excerpt = payload.excerpt.unwrap_or(existing.excerpt);
    let content = payload.content.unwrap_or(existing.content);
    let category = payload.category.unwrap_or(existing.category);
    let publish_date = payload.publish_date.unwrap_or(existing.publish_date);
    let author = payload.author.unwrap_or(existing.author);
    let status = payload.status.unwrap_or(existing.status);
    let featured = payload.featured.unwrap_or(existing.featured);
    let tags = payload.tags.unwrap_or(existing.tags);

    let updated = sqlx::query_as::<_, NewsArticle>(
        "UPDATE public.news_articles
         SET title = $1, slug = $2, excerpt = $3, content = $4, category = $5, publish_date = $6, author = $7, status = $8, featured = $9, tags = $10, updated_at = now()
         WHERE id = $11
         RETURNING id, title, slug, excerpt, content, category, publish_date, author, status, featured, tags, created_at, updated_at",
    )
    .bind(&title)
    .bind(&slug)
    .bind(&excerpt)
    .bind(&content)
    .bind(&category)
    .bind(publish_date)
    .bind(&author)
    .bind(&status)
    .bind(featured)
    .bind(&tags)
    .bind(id)
    .fetch_one(&pool)
    .await?;

    let _ = sqlx::query(
        "INSERT INTO public.activity_logs (action, entity, target_name, user_name)
         VALUES ($1, $2, $3, $4)",
    )
    .bind("Updated Article")
    .bind("News")
    .bind(&updated.title)
    .bind(&claims.name)
    .execute(&pool)
    .await;

    Ok(Json(updated))
}

/// DELETE /api/news/:id - Delete News Article (STRICTLY Admin Only)
pub async fn delete_news(
    State(pool): State<PgPool>,
    AdminOnly(admin_claims): AdminOnly,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let article = sqlx::query_as::<_, NewsArticle>(
        "SELECT id, title, slug, excerpt, content, category, publish_date, author, status, featured, tags, created_at, updated_at
         FROM public.news_articles
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Article not found".to_string()))?;

    sqlx::query("DELETE FROM public.news_articles WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;

    let _ = sqlx::query(
        "INSERT INTO public.activity_logs (action, entity, target_name, user_name)
         VALUES ($1, $2, $3, $4)",
    )
    .bind("Deleted Article")
    .bind("News")
    .bind(&article.title)
    .bind(&admin_claims.name)
    .execute(&pool)
    .await;

    Ok(Json(serde_json::json!({
        "message": format!("Article '{}' deleted successfully", article.title),
        "id": id,
    })))
}
