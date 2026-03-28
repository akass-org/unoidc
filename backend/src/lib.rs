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

pub async fn build_app() -> Router {
    // TODO: 从环境变量加载配置和数据库连接
    // 现在先返回一个空的router用于测试
    Router::new()
}

pub fn build_app_with_state(state: Arc<AppState>) -> Router {
    use axum::routing::{get, post};

    Router::new()
        // 健康检查
        .route("/health/live", get(|| async { "OK" }))
        .route("/health/ready", get(|| async { "OK" }))

        // 认证路由
        .route("/api/v1/auth/login", post(handler::auth::login))
        .route("/api/v1/auth/logout", post(handler::auth::logout))
        .route("/api/v1/auth/register", post(handler::auth::register))
        .route("/api/v1/auth/forgot-password", post(handler::auth::forgot_password))

        // 添加客户端地址的layer (可选)
        .layer(axum::Extension::<Option<std::net::SocketAddr>>(None))

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
