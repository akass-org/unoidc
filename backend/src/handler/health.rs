// Health Check Handlers
//
// 提供 liveness 和 readiness 健康检查端点

use axum::{extract::State, Json};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::{service::KeyService, AppState};

pub async fn liveness() -> Json<Value> {
    Json(json!({ "status": "alive" }))
}

pub async fn readiness(State(state): State<Arc<AppState>>) -> Json<Value> {
    let db_status = check_database(&state).await;
    let keys_status = check_keys(&state).await;

    let overall_status = if db_status == "up" && keys_status == "up" {
        "ready"
    } else {
        "not_ready"
    };

    Json(json!({
        "status": overall_status,
        "database": {
            "status": db_status,
        },
        "keys": {
            "status": keys_status,
        }
    }))
}

async fn check_database(state: &Arc<AppState>) -> &'static str {
    match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => "up",
        Err(_) => "down",
    }
}

async fn check_keys(state: &Arc<AppState>) -> &'static str {
    match KeyService::get_active_key(&state.db, &state.config.private_key_encryption_key).await {
        Ok(jwk) if !jwk.public_key_jwk.is_null() => "up",
        Err(_) => "down",
        _ => "down",
    }
}
