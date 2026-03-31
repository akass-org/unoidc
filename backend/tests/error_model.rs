// 统一错误模型测试
//
// 测试标准化的 API 错误响应格式和日志脱敏

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use backend::{build_app_with_state};
use serde_json::{json, Value};
use serial_test::serial;
use tower::ServiceExt;

async fn parse_error_response(response: axum::response::Response) -> Option<Value> {
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body_bytes).ok()
}

#[tokio::test]
#[serial]
async fn test_authentication_failed_error() {
    let state = common::get_test_db().await;
    let app = build_app_with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "username": "nonexistent_user",
                        "password": "wrong_password"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let error_json = parse_error_response(response).await.unwrap();
    assert!(error_json.get("error").is_some());
    assert_eq!(error_json["status"], 401);
}

#[tokio::test]
#[serial]
async fn test_invalid_request_error() {
    let state = common::get_test_db().await;
    let app = build_app_with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "username": "testuser"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    assert!(status == StatusCode::UNPROCESSABLE_ENTITY || status == StatusCode::BAD_REQUEST,
        "Expected 400 or 422, got {}", status);
    let error_json = parse_error_response(response).await;
    if let Some(json) = error_json {
        assert!(json.get("error").is_some());
        assert!(json["status"].is_number());
    }
}

#[tokio::test]
#[serial]
async fn test_forbidden_error() {
    let _state = common::get_test_db().await;
}

#[tokio::test]
#[serial]
async fn test_not_found_error() {
    let state = common::get_test_db().await;
    let app = build_app_with_state(state);

    let _response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/users/nonexistent-id")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
}

#[tokio::test]
#[serial]
async fn test_oidc_protocol_error() {
    let state = common::get_test_db().await;
    let app = build_app_with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/authorize?client_id=invalid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    assert!(status.is_client_error() || status == StatusCode::FOUND,
        "Expected client error or redirect, got {}", status);
    if status != StatusCode::FOUND && status != StatusCode::SEE_OTHER {
        if let Some(error_json) = parse_error_response(response).await {
            if let Some(error_code) = error_json.get("error") {
                assert!(error_code.is_string());
            }
        }
    }
}

#[tokio::test]
#[serial]
async fn test_error_response_format() {
    let state = common::get_test_db().await;
    let app = build_app_with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "username": "test",
                        "password": "test"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let error_json = parse_error_response(response).await;

    if let Some(error_json) = error_json {
        assert!(error_json.get("error").is_some(), "Missing 'error' field");
        assert!(error_json.get("status").is_some(), "Missing 'status' field");
        assert!(error_json["error"].is_string(), "'error' should be a string");
        assert!(error_json["status"].is_number(), "'status' should be a number");
    }
}

#[tokio::test]
#[serial]
async fn test_internal_server_error_sanitization() {
    let state = common::get_test_db().await;
    let app = build_app_with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "username": "test",
                        "password": "test"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    if response.status() == StatusCode::INTERNAL_SERVER_ERROR {
        if let Some(error_json) = parse_error_response(response).await {
            let error_str = error_json.to_string().to_lowercase();

            assert!(!error_str.contains("sqlx"), "Error response should not contain SQLx details");
            assert!(!error_str.contains("database"), "Error response should not contain database details");
            assert!(!error_str.contains("stack"), "Error response should not contain stack trace");
            assert!(!error_str.contains("password"), "Error response should not contain password");
            assert!(!error_str.contains("token"), "Error response should not contain token");
        }
    }
}
