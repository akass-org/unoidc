use axum::Json;
use serde_json::{json, Value};
use crate::error::Result;

pub async fn liveness() -> Result<Json<Value>> {
    Ok(Json(json!({ "status": "alive" })))
}

pub async fn readiness() -> Result<Json<Value>> {
    // TODO: 检查数据库连接和 JWK 可用性
    Ok(Json(json!({ "status": "ready" })))
}
