use std::env;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub database_url: String,
    pub supabase_url: Option<String>,
    pub supabase_anon_key: Option<String>,
    pub port: u16,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub cors_origins: Vec<String>,
    pub cloudinary_cloud_name: Option<String>,
    pub cloudinary_api_key: Option<String>,
    pub cloudinary_api_secret: Option<String>,
    pub cloudinary_upload_preset: Option<String>,
    pub cloudinary_folder: String,
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
    pub google_redirect_uri: Option<String>,
    pub frontend_url: String,
    pub rate_limit_per_second: u64,
    pub rate_limit_burst: u32,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@127.0.0.1:54322/postgres".to_string());

        let supabase_url = env::var("SUPABASE_URL").ok();
        let supabase_anon_key = env::var("SUPABASE_ANON_KEY").ok();

        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);

        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "bdai_fallback_secret_key_change_in_production".to_string());

        let jwt_expiration_hours = env::var("JWT_EXPIRATION_HOURS")
            .ok()
            .and_then(|h| h.parse().ok())
            .unwrap_or(72);

        let cors_origins_raw = env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000,http://localhost:3001".to_string());

        let cors_origins = cors_origins_raw
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let cloudinary_cloud_name = env::var("CLOUDINARY_CLOUD_NAME").ok();
        let cloudinary_api_key = env::var("CLOUDINARY_API_KEY").ok();
        let cloudinary_api_secret = env::var("CLOUDINARY_API_SECRET").ok();
        let cloudinary_upload_preset = env::var("CLOUDINARY_UPLOAD_PRESET").ok();
        let cloudinary_folder = env::var("CLOUDINARY_FOLDER").unwrap_or_else(|_| "bdai".to_string());
        let google_client_id = env::var("GOOGLE_CLIENT_ID").ok();
        let google_client_secret = env::var("GOOGLE_CLIENT_SECRET").ok();
        let google_redirect_uri = env::var("GOOGLE_REDIRECT_URI").ok();
        let frontend_url = env::var("FRONTEND_URL")
            .unwrap_or_else(|_| "http://localhost:3001".to_string());

        let rate_limit_per_second = env::var("RATE_LIMIT_PER_SECOND")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);

        let rate_limit_burst = env::var("RATE_LIMIT_BURST")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(50);

        Self {
            database_url,
            supabase_url,
            supabase_anon_key,
            port,
            jwt_secret,
            jwt_expiration_hours,
            cors_origins,
            cloudinary_cloud_name,
            cloudinary_api_key,
            cloudinary_api_secret,
            cloudinary_upload_preset,
            cloudinary_folder,
            google_client_id,
            google_client_secret,
            google_redirect_uri,
            frontend_url,
            rate_limit_per_second,
            rate_limit_burst,
        }
    }
}
