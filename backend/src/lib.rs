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
    use tower::ServiceBuilder;
    use crate::middleware::request_context_middleware;

    Router::new()
        // 健康检查
        .route("/health/live", get(|| async { "OK" }))
        .route("/health/ready", get(|| async { "OK" }))

        // 认证路由
        .route("/api/v1/auth/login", post(handler::auth::login))
        .route("/api/v1/auth/logout", post(handler::auth::logout))
        .route("/api/v1/auth/register", post(handler::auth::register))
        .route("/api/v1/auth/forgot-password", post(handler::auth::forgot_password))

        // OIDC 路由
        .route("/.well-known/openid-configuration", get(handler::oidc::discovery))
        .route("/jwks.json", get(handler::oidc::jwks))
        .route("/authorize", get(handler::oidc::authorize_get))
        .route("/authorize/consent", post(handler::oidc::authorize_consent))
        .route("/token", post(handler::oidc::token))
        .route("/userinfo", get(handler::oidc::userinfo))

        // 添加中间件层
        .layer(
            ServiceBuilder::new()
                // 请求上下文中间件（添加请求 ID 和关联 ID）
                .layer(axum::middleware::from_fn(request_context_middleware))
                // 添加客户端地址扩展
                .layer(axum::Extension::<Option<std::net::SocketAddr>>(None))
        )

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
