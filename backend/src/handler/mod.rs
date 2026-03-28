pub mod auth;
pub mod oidc;
pub mod health;
pub mod admin;
pub mod me;

use std::sync::Arc;

use axum::Router;
use crate::AppState;

pub fn create_routes(_state: Arc<AppState>) -> Router {
    Router::new()
        // TODO: 添加具体的路由
}
