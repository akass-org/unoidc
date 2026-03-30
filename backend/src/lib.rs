pub mod config;
pub mod error;
pub mod db;
pub mod handler;
pub mod service;
pub mod repo;
pub mod model;
pub mod crypto;
pub mod middleware;
pub mod metrics;

use axum::Router;
use std::sync::Arc;

pub fn build_app_with_state(state: Arc<AppState>) -> Router {
    use axum::routing::{get, post};
    use crate::middleware::{
        request_context_middleware, rate_limit_middleware, create_rate_limiter,
        create_cors_layer, CorsConfig, csrf_middleware,
    };

    let rate_limiter = create_rate_limiter(
        state.config.rate_limit_max_requests,
        state.config.rate_limit_window_secs,
        state.config.rate_limit_login_max_requests,
        state.config.rate_limit_login_window_secs,
        state.config.rate_limit_token_max_requests,
        state.config.rate_limit_token_window_secs,
    );

    let cors_config = CorsConfig {
        allowed_origins: state.config.cors_allowed_origins.clone(),
    };
    let cors_layer = create_cors_layer(&cors_config);

    Router::new()
        .route("/health/live", get(|| async { "OK" }))
        .route("/health/ready", get(|| async { "OK" }))

        .route("/api/v1/auth/login", post(handler::auth::login))
        .route("/api/v1/auth/logout", post(handler::auth::logout))
        .route("/api/v1/auth/register", post(handler::auth::register))
        .route("/api/v1/auth/forgot-password", post(handler::auth::forgot_password))

        .route("/.well-known/openid-configuration", get(handler::oidc::discovery))
        .route("/jwks.json", get(handler::oidc::jwks))
        .route("/authorize", get(handler::oidc::authorize_get))
        .route("/authorize/consent", post(handler::oidc::authorize_consent))
        .route("/token", post(handler::oidc::token))
        .route("/userinfo", get(handler::oidc::userinfo))
        .route("/logout", get(handler::oidc::logout))

        // 请求流程（从外到内）：
        // CORS → rate_limit → csrf → request_context → handler
        .layer(axum::middleware::from_fn(csrf_middleware))
        .layer(axum::middleware::from_fn(rate_limit_middleware))
        .layer(axum::Extension(rate_limiter))
        .layer(axum::Extension::<Option<String>>(None))
        .layer(axum::middleware::from_fn(request_context_middleware))
        .layer(cors_layer)

        .with_state(state)
}

pub struct AppState {
    pub config: config::Config,
    pub db: sqlx::PgPool,
}

impl AppState {
    pub fn new(config: config::Config, db: sqlx::PgPool) -> Arc<Self> {
        Arc::new(Self { config, db })
    }
}
