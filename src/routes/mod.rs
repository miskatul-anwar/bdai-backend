pub mod activity_routes;
pub mod auth_routes;
pub mod health_routes;
pub mod news_routes;
pub mod objective_routes;
pub mod settings_routes;
pub mod team_routes;
pub mod upload_routes;
pub mod user_routes;
pub mod vacancy_routes;

use axum::{
    routing::{get, post, put},
    Router,
};

use crate::state::AppState;

pub fn create_router() -> Router<AppState> {
    Router::new()
        // Health
        .route("/api/health", get(health_routes::health_check))
        // Authentication
        .route("/api/auth/login", post(auth_routes::login))
        .route("/api/auth/me", get(auth_routes::me))
        .route("/api/auth/google", post(auth_routes::google_auth))
        .route("/api/auth/google/url", get(auth_routes::google_auth_url))
        .route("/api/auth/google/callback", get(auth_routes::google_callback))
        // User Management (Admin Only for Add/Remove/Edit)
        .route(
            "/api/users",
            get(user_routes::list_users).post(user_routes::create_user),
        )
        .route(
            "/api/users/{id}",
            put(user_routes::update_user).delete(user_routes::delete_user),
        )
        // Team Members / Employees (Admin Only for Add/Remove)
        .route(
            "/api/team",
            get(team_routes::list_team).post(team_routes::create_team_member),
        )
        .route(
            "/api/team/{id}",
            get(team_routes::get_team_member)
                .put(team_routes::update_team_member)
                .delete(team_routes::delete_team_member),
        )
        // News Articles
        .route(
            "/api/news",
            get(news_routes::list_news).post(news_routes::create_news),
        )
        .route("/api/news/slug/{slug}", get(news_routes::get_news_by_slug))
        .route(
            "/api/news/{id}",
            put(news_routes::update_news).delete(news_routes::delete_news),
        )
        // Vacancies & Notices
        .route(
            "/api/vacancies",
            get(vacancy_routes::list_vacancies).post(vacancy_routes::create_vacancy),
        )
        .route(
            "/api/vacancies/{id}",
            get(vacancy_routes::get_vacancy)
                .put(vacancy_routes::update_vacancy)
                .delete(vacancy_routes::delete_vacancy),
        )
        // Research Objectives
        .route(
            "/api/objectives",
            get(objective_routes::list_objectives).post(objective_routes::create_objective),
        )
        .route(
            "/api/objectives/{id}",
            put(objective_routes::update_objective).delete(objective_routes::delete_objective),
        )
        // Activity Logs
        .route("/api/activities", get(activity_routes::list_activities))
        // Site Settings (Dynamic Portal Content)
        .route("/api/settings", get(settings_routes::get_all_settings))
        .route(
            "/api/settings/{id}",
            get(settings_routes::get_setting).put(settings_routes::update_setting),
        )
        // Cloudinary Image Upload
        .route("/api/upload", post(upload_routes::upload_image))
        .route(
            "/api/upload/signature",
            get(upload_routes::get_upload_signature),
        )
}
