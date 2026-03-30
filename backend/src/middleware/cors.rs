use axum::http::Method;
use std::time::Duration;
use tower_http::cors::{AllowOrigin, CorsLayer, MaxAge};

#[derive(Debug, Clone)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec![
                "http://localhost:5173".to_string(),
                "http://localhost:3000".to_string(),
            ],
        }
    }
}

pub fn create_cors_layer(config: &CorsConfig) -> CorsLayer {
    use axum::http::header;

    let origins: Vec<_> = config
        .allowed_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();

    let allow_origin = if origins.is_empty() {
        AllowOrigin::any()
    } else {
        AllowOrigin::list(origins)
    };

    CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::COOKIE,
            header::HeaderName::from_static("x-csrf-token"),
            header::HeaderName::from_static("x-request-id"),
            header::HeaderName::from_static("x-correlation-id"),
        ])
        .expose_headers([
            header::HeaderName::from_static("x-request-id"),
            header::HeaderName::from_static("x-correlation-id"),
        ])
        .allow_credentials(true)
        .max_age(MaxAge::exact(Duration::from_secs(3600)))
}
