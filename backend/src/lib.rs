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
    use axum::routing::{get, post, patch, delete};
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
        state.config.trusted_proxy_ips.clone(),
    );

    let cors_config = CorsConfig {
        allowed_origins: state.config.cors_allowed_origins.clone(),
    };
    let cors_layer = create_cors_layer(&cors_config);

    Router::new()
        // Health
        .route("/health/live", get(handler::health::liveness))
        .route("/health/ready", get(handler::health::readiness))

        // Auth
        .route("/api/v1/auth/login", post(handler::auth::login))
        .route("/api/v1/auth/logout", post(handler::auth::logout))
        .route("/api/v1/auth/register", post(handler::auth::register))
        .route("/api/v1/auth/forgot-password", post(handler::auth::forgot_password))
        .route("/api/v1/auth/session", get(handler::auth::get_session))

        // Me (User Self-Service)
        .route("/api/v1/me", get(handler::me::get_profile))
        .route("/api/v1/me", patch(handler::me::update_profile))
        .route("/api/v1/me/password", post(handler::me::change_password))
        .route("/api/v1/me/avatar", post(handler::me::upload_avatar))
        .route("/api/v1/me/apps", get(handler::me::get_apps))
        .route("/api/v1/me/consents", get(handler::me::get_consents))
        .route("/api/v1/me/consents/{client_id}", delete(handler::me::revoke_consent))

        // Admin
        .route("/api/v1/admin/users", get(handler::admin::get_users))
        .route("/api/v1/admin/users", post(handler::admin::create_user))
        .route("/api/v1/admin/users/{id}", patch(handler::admin::update_user))
        .route("/api/v1/admin/users/{id}/reset-password", post(handler::admin::reset_user_password))
        .route("/api/v1/admin/groups", get(handler::admin::get_groups))
        .route("/api/v1/admin/groups", post(handler::admin::create_group))
        .route("/api/v1/admin/groups/{id}", patch(handler::admin::update_group))
        .route("/api/v1/admin/groups/{id}", delete(handler::admin::delete_group))
        .route("/api/v1/admin/clients", get(handler::admin::get_clients))
        .route("/api/v1/admin/clients", post(handler::admin::create_client))
        .route("/api/v1/admin/clients/{id}", patch(handler::admin::update_client))
        .route("/api/v1/admin/clients/{id}", delete(handler::admin::delete_client))
        .route("/api/v1/admin/clients/{id}/reset-secret", post(handler::admin::reset_client_secret))
        .route("/api/v1/admin/audit-logs", get(handler::admin::get_audit_logs))
        .route("/api/v1/admin/settings", get(handler::admin::get_settings))
        .route("/api/v1/admin/settings", patch(handler::admin::update_settings))
        .route("/api/v1/admin/keys/rotate", post(handler::admin::rotate_key))

        // Public config (for login page branding)
        .route("/api/v1/public/config", get(handler::auth::get_public_config))

        // OIDC
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
        .layer(axum::Extension::<Option<std::net::SocketAddr>>(None))
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
