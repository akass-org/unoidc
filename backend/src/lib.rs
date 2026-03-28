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
    Router::new()
}

pub struct AppState {
    pub config: config::Config,
    pub db: sqlx::Pool<sqlx::Any>,
}

impl AppState {
    pub fn new(config: config::Config, db: sqlx::Pool<sqlx::Any>) -> Arc<Self> {
        Arc::new(Self { config, db })
    }
}
