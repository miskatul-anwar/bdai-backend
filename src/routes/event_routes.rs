use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::middleware::EditorUser;
use crate::models::{CreateEventRequest, Event, GalleryItem, SiteSetting, UpdateEventRequest};

#[derive(Debug, Deserialize)]
pub struct EventQuery {
    pub status: Option<String>,
}

fn get_seed_events() -> Vec<Event> {
    vec![
        Event {
            id: "event_workshop_debasish".to_string(),
            title: "Professor Dr. Debasish Ghose from Kristiania University College, Norway visited our lab for collaboration purpose. He delivers an intensive quality paper writing workshop.".to_string(),
            date: "29th July 2026".to_string(),
            status: "held".to_string(),
            category: "Workshop".to_string(),
            location: Some("BDAI Lab & SPMT Office, Department of CSE, University of Chittagong".to_string()),
            description: Some("Professor Dr. Debasish Ghose from Kristiania University College, Norway visited our lab for research collaboration and delivered an intensive quality paper writing workshop for researchers and faculty members.".to_string()),
            banner: "/events/workshop_banner.jpeg".to_string(),
            gallery: vec![
                GalleryItem { src: "/events/workshop_1.jpeg".to_string(), alt: "Workshop participants gathered with Prof. Dr. Debasish Ghose".to_string() },
                GalleryItem { src: "/events/workshop_2.jpeg".to_string(), alt: "Collaborators and researchers in the department hallway".to_string() },
                GalleryItem { src: "/events/workshop_3.jpeg".to_string(), alt: "Prof. Dr. Debasish Ghose, Prof. Dr. Rudra Pratap Deb Nath, and Dr. Abu Nowshed Chy at SPMT office".to_string() },
                GalleryItem { src: "/events/workshop_4.jpeg".to_string(), alt: "Faculty and visiting professor outside SPMT office".to_string() },
                GalleryItem { src: "/events/workshop_5.jpeg".to_string(), alt: "Collaboration meeting at SPMT office".to_string() },
                GalleryItem { src: "/events/workshop_6.jpeg".to_string(), alt: "Research discussion at SPMT office".to_string() },
                GalleryItem { src: "/events/workshop_7.jpeg".to_string(), alt: "Faculty collaboration outside BIKE Lab SPMT office".to_string() },
                GalleryItem { src: "/events/workshop_8.jpeg".to_string(), alt: "Group photo in the BDAI lab".to_string() },
            ],
            order: 1,
            created_at: Some("2026-07-29T10:00:00Z".to_string()),
            updated_at: Some("2026-07-29T10:00:00Z".to_string()),
        },
        Event {
            id: "event_seminar_rag_bi".to_string(),
            title: "RAG-Driven Business Intelligence Platform Integration: Enterprise Data for Real-Time Insight, Predictive, and Prescriptive Decision Analytics".to_string(),
            date: "2.00PM · 19th May 2026".to_string(),
            status: "held".to_string(),
            category: "Seminar".to_string(),
            location: Some("Department of Computer Science and Engineering, University of Chittagong".to_string()),
            description: Some("Seminar on enterprise integration of retrieval-augmented generation and semantic knowledge graphs for real-time analytics and predictive decision systems.".to_string()),
            banner: "/events/seminar2.jpg".to_string(),
            gallery: vec![
                GalleryItem { src: "/events/seminar2_1.jpeg".to_string(), alt: "RAG-Driven BI seminar gallery image 1".to_string() },
                GalleryItem { src: "/events/seminar2_2.jpeg".to_string(), alt: "RAG-Driven BI seminar gallery image 2".to_string() },
                GalleryItem { src: "/events/seminar2_3.jpeg".to_string(), alt: "RAG-Driven BI seminar gallery image 3".to_string() },
                GalleryItem { src: "/events/seminar2_4.jpeg".to_string(), alt: "RAG-Driven BI seminar gallery image 4".to_string() },
                GalleryItem { src: "/events/seminar2_5.jpeg".to_string(), alt: "RAG-Driven BI seminar gallery image 5".to_string() },
                GalleryItem { src: "/events/seminar2_6.jpeg".to_string(), alt: "RAG-Driven BI seminar gallery image 6".to_string() },
            ],
            order: 2,
            created_at: Some("2026-05-19T14:00:00Z".to_string()),
            updated_at: Some("2026-05-19T14:00:00Z".to_string()),
        },
        Event {
            id: "event_phd_cyberbullying".to_string(),
            title: "Identificatin of the Digital Footprints of Cyberbullying and the personality traits of the perpretators to protect the malicious activity".to_string(),
            date: "2.00PM · 14th May 2026".to_string(),
            status: "held".to_string(),
            category: "PhD Seminar".to_string(),
            location: Some("Department of Computer Science and Engineering, University of Chittagong".to_string()),
            description: Some("PhD Open Seminar on machine learning models and digital footprint analysis for cyberbullying detection and perpetrator personality classification in Bengali social text.".to_string()),
            banner: "/events/1.png".to_string(),
            gallery: vec![
                GalleryItem { src: "/events/phd1.png".to_string(), alt: "Event gallery image 1".to_string() },
                GalleryItem { src: "/events/phd2.png".to_string(), alt: "Event gallery image 2".to_string() },
                GalleryItem { src: "/events/phd3.png".to_string(), alt: "Event gallery image 3".to_string() },
                GalleryItem { src: "/events/phd4.png".to_string(), alt: "Event gallery image 4".to_string() },
                GalleryItem { src: "/events/phd5.png".to_string(), alt: "Event gallery image 5".to_string() },
                GalleryItem { src: "/events/phd6.png".to_string(), alt: "Event gallery image 6".to_string() },
            ],
            order: 3,
            created_at: Some("2026-05-14T14:00:00Z".to_string()),
            updated_at: Some("2026-05-14T14:00:00Z".to_string()),
        },
    ]
}

/// Helper to get current events list from public.site_settings
async fn get_events_from_db(pool: &PgPool) -> Result<Vec<Event>, AppError> {
    let row = sqlx::query_as::<_, SiteSetting>(
        "SELECT id, data, updated_at FROM public.site_settings WHERE id = 'events'",
    )
    .fetch_optional(pool)
    .await?;

    match row {
        Some(setting) => {
            let mut events: Vec<Event> = serde_json::from_value(setting.data).unwrap_or_default();
            if events.is_empty() {
                events = get_seed_events();
            } else {
                events.sort_by_key(|e| e.order);
            }
            Ok(events)
        }
        None => Ok(get_seed_events()),
    }
}

/// Helper to persist events list to public.site_settings
async fn save_events_to_db(
    pool: &PgPool,
    events: &[Event],
    action: &str,
    target_name: &str,
    user_name: &str,
) -> Result<(), AppError> {
    let json_val = serde_json::to_value(events)
        .map_err(|e| AppError::Internal(format!("Failed to serialize events: {e}")))?;

    sqlx::query(
        "INSERT INTO public.site_settings (id, data, updated_at)
         VALUES ('events', $1, now())
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
    .bind("Event")
    .bind(target_name)
    .bind(user_name)
    .execute(pool)
    .await;

    Ok(())
}

/// GET /api/events - List events (Public)
pub async fn list_events(
    State(pool): State<PgPool>,
    Query(query): Query<EventQuery>,
) -> Result<Json<Vec<Event>>, AppError> {
    let events = get_events_from_db(&pool).await?;

    let filtered = if let Some(status_filter) = query.status {
        let filter_lower = status_filter.to_lowercase();
        events
            .into_iter()
            .filter(|e| e.status.to_lowercase() == filter_lower)
            .collect()
    } else {
        events
    };

    Ok(Json(filtered))
}

/// GET /api/events/:id - Get a single event (Public)
pub async fn get_event(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> Result<Json<Event>, AppError> {
    let events = get_events_from_db(&pool).await?;
    let event = events
        .into_iter()
        .find(|e| e.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Event '{}' not found", id)))?;

    Ok(Json(event))
}

/// POST /api/events - Create new event (Admin or Moderator)
pub async fn create_event(
    State(pool): State<PgPool>,
    EditorUser(claims): EditorUser,
    Json(payload): Json<CreateEventRequest>,
) -> Result<(StatusCode, Json<Event>), AppError> {
    let mut events = get_events_from_db(&pool).await?;

    let now_str = chrono::Utc::now().to_rfc3339();
    let new_id = payload
        .id
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| format!("event_{}", Uuid::new_v4().simple()));
    let next_order = payload.order.unwrap_or(events.len() as i32 + 1);

    let new_event = Event {
        id: new_id,
        title: payload.title.clone(),
        date: payload.date,
        status: payload.status,
        category: payload.category,
        location: payload.location,
        description: payload.description,
        banner: payload.banner,
        gallery: payload.gallery,
        order: next_order,
        created_at: Some(now_str.clone()),
        updated_at: Some(now_str),
    };

    events.push(new_event.clone());
    events.sort_by_key(|e| e.order);

    save_events_to_db(&pool, &events, "Created", &payload.title, &claims.name).await?;

    Ok((StatusCode::CREATED, Json(new_event)))
}

/// PUT /api/events/:id - Update existing event (Admin or Moderator)
pub async fn update_event(
    State(pool): State<PgPool>,
    EditorUser(claims): EditorUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdateEventRequest>,
) -> Result<Json<Event>, AppError> {
    let mut events = get_events_from_db(&pool).await?;

    let index = events
        .iter()
        .position(|e| e.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Event '{}' not found", id)))?;

    let now_str = chrono::Utc::now().to_rfc3339();
    let current = &mut events[index];

    if let Some(title) = payload.title {
        current.title = title;
    }
    if let Some(date) = payload.date {
        current.date = date;
    }
    if let Some(status) = payload.status {
        current.status = status;
    }
    if let Some(category) = payload.category {
        current.category = category;
    }
    if let Some(location) = payload.location {
        current.location = Some(location);
    }
    if let Some(description) = payload.description {
        current.description = Some(description);
    }
    if let Some(banner) = payload.banner {
        current.banner = banner;
    }
    if let Some(gallery) = payload.gallery {
        current.gallery = gallery;
    }
    if let Some(order) = payload.order {
        current.order = order;
    }
    current.updated_at = Some(now_str);

    let updated_event = current.clone();
    events.sort_by_key(|e| e.order);

    save_events_to_db(&pool, &events, "Updated", &updated_event.title, &claims.name).await?;

    Ok(Json(updated_event))
}

/// DELETE /api/events/:id - Delete an event (Admin or Moderator)
pub async fn delete_event(
    State(pool): State<PgPool>,
    EditorUser(claims): EditorUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let mut events = get_events_from_db(&pool).await?;

    let target_id = id.trim();
    let index = events
        .iter()
        .position(|e| e.id.trim() == target_id || e.id.trim().trim_start_matches("event_") == target_id.trim_start_matches("event_"))
        .ok_or_else(|| AppError::NotFound(format!("Event '{}' not found", id)))?;

    let removed = events.remove(index);

    save_events_to_db(&pool, &events, "Deleted", &removed.title, &claims.name).await?;

    Ok(StatusCode::NO_CONTENT)
}
