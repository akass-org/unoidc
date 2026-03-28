// 统一错误模型测试
//
// 测试标准化的 API 错误响应格式和日志脱敏

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use backend::{build_app_with_state, config::Config, db, AppState};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::OnceCell;
use tower::ServiceExt;

static TEST_DB: OnceCell<Arc<AppState>> = OnceCell::const_new();

/// 获取或创建测试数据库连接池（全局单例）
async fn get_test_db() -> Arc<AppState> {
    TEST_DB
        .get_or_init(|| async {
            let database_url = std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set for tests");
            let pool = db::connect(&database_url).await.unwrap();

            // 只在第一次运行迁移
            db::run_migrations(&pool).await.unwrap();

            let config = Config::default();
            Arc::new(AppState { config, db: pool })
        })
        .await
        .clone()
}

/// 辅助函数：解析错误响应
async fn parse_error_response(response: axum::response::Response) -> Value {
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body_bytes).unwrap()
}

#[tokio::test]
async fn test_authentication_failed_error() {
    let state = get_test_db().await;
    let app = build_app_with_state(state.clone());

    // 测试登录失败（用户不存在）
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

    let error_json = parse_error_response(response).await;
    assert!(error_json.get("error").is_some());
    assert_eq!(error_json["status"], 401);
}

#[tokio::test]
async fn test_invalid_request_error() {
    let state = get_test_db().await;
    let app = build_app_with_state(state.clone());

    // 测试无效的登录请求（缺少必需字段）
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "username": "testuser"
                        // 缺少 password 字段
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let error_json = parse_error_response(response).await;
    assert!(error_json.get("error").is_some());
    assert_eq!(error_json["status"], 400);
}

#[tokio::test]
async fn test_forbidden_error() {
    let state = get_test_db().await;
    let app = build_app_with_state(state.clone());

    // 测试访问被锁定的账户
    // 这需要先创建一个被锁定的账户，然后尝试登录
    // 这里只是测试框架，具体实现会在后续补充
}

#[tokio::test]
async fn test_not_found_error() {
    let state = get_test_db().await;
    let app = build_app_with_state(state.clone());

    // 测试访问不存在的资源
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/users/nonexistent-id")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // 目前这个路由可能还不存在，所以可能返回 404
    // 这是一个占位测试
}

#[tokio::test]
async fn test_oidc_protocol_error() {
    let state = get_test_db().await;
    let app = build_app_with_state(state.clone());

    // 测试 OIDC 协议错误（例如 invalid_request）
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/authorize?client_id=invalid") // 缺少必需参数
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // OIDC 错误应该符合 RFC 6749 标准格式
    let status = response.status();
    // authorize 端点可能返回 400 或重定向
    if status == StatusCode::BAD_REQUEST {
        let error_json = parse_error_response(response).await;
        assert!(error_json.get("error").is_some());
        // OIDC 标准错误字段
        if let Some(error_code) = error_json.get("error") {
            assert!(error_code.is_string());
        }
    }
}

#[tokio::test]
async fn test_error_response_format() {
    let state = get_test_db().await;
    let app = build_app_with_state(state.clone());

    // 测试错误响应格式是否统一
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

    // 所有错误响应应该包含这些字段
    assert!(error_json.get("error").is_some(), "Missing 'error' field");
    assert!(
        error_json.get("status").is_some(),
        "Missing 'status' field"
    );

    // error 字段应该是字符串
    assert!(error_json["error"].is_string(), "'error' should be a string");

    // status 字段应该是数字
    assert!(
        error_json["status"].is_number(),
        "'status' should be a number"
    );
}

#[tokio::test]
async fn test_internal_server_error_sanitization() {
    let state = get_test_db().await;
    let app = build_app_with_state(state.clone());

    // 测试内部服务器错误不应该暴露敏感信息
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
        let error_json = parse_error_response(response).await;

        // 确保不包含敏感信息（数据库错误详情、堆栈跟踪等）
        let error_str = error_json.to_string().to_lowercase();

        // 不应该包含这些关键词
        assert!(
            !error_str.contains("sqlx"),
            "Error response should not contain SQLx details"
        );
        assert!(
            !error_str.contains("database"),
            "Error response should not contain database details"
        );
        assert!(
            !error_str.contains("stack"),
            "Error response should not contain stack trace"
        );
        assert!(
            !error_str.contains("password"),
            "Error response should not contain password"
        );
        assert!(
            !error_str.contains("token"),
            "Error response should not contain token"
        );
    }
}
