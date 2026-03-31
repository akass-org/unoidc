// 观测性测试
//
// 测试审计日志持久化、健康检查和 Prometheus 指标

mod common;

use backend::{
    build_app_with_state,
    model::{CreateSession, CreateUser, CreateAuditLog},
    repo::{AuditLogRepo, SessionRepo, UserRepo},
    crypto::password,
    AppState,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serial_test::serial;
use tower::ServiceExt;
use uuid::Uuid;

fn unique_username() -> String {
    format!("testuser_{}", Uuid::new_v4().as_simple())
}

async fn cleanup_user(state: &AppState, username: &str) {
    if let Some(user) = UserRepo::find_by_username(&state.db, username).await.unwrap() {
        sqlx::query("DELETE FROM user_sessions WHERE user_id = $1")
            .bind(user.id)
            .execute(&state.db)
            .await
            .ok();
        sqlx::query("DELETE FROM audit_logs WHERE actor_user_id = $1")
            .bind(user.id)
            .execute(&state.db)
            .await
            .ok();
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user.id)
            .execute(&state.db)
            .await
            .ok();
    }
}

// ===== 审计日志测试 =====

#[tokio::test]
#[serial]
async fn test_audit_log_persistence_on_login_success() {
    let state = common::get_test_db().await;
    let username = unique_username();

    let password = "test_password_123";
    let password_hash = password::hash_password(password).unwrap();
    UserRepo::create(&state.db, CreateUser {
        username: username.clone(),
        email: format!("{}@test.com", username),
        password_hash,
        display_name: None,
        given_name: None,
        family_name: None,
    }).await.unwrap();

    let app = build_app_with_state(state.clone());

    let response = app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .header("content-type", "application/json")
            .header("x-correlation-id", "test-correlation-123")
            .body(Body::from(serde_json::json!({
                "username": username,
                "password": "test_password_123"
            }).to_string()))
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let user = UserRepo::find_by_username(&state.db, &username).await.unwrap().unwrap();
    let logs = AuditLogRepo::find_user_logs(&state.db, user.id, 100).await.unwrap();

    let login_logs: Vec<_> = logs.iter()
        .filter(|l| l.action == "login" && l.outcome == "success")
        .collect();
    assert!(!login_logs.is_empty(), "Expected login success audit log");

    let log = login_logs.first().unwrap();
    assert_eq!(log.actor_user_id, Some(user.id));
    assert_eq!(log.correlation_id, "test-correlation-123");
    assert_eq!(log.outcome, "success");
    assert!(log.ip_address.is_some());
    assert!(log.user_agent.is_some());

    cleanup_user(&state, &username).await;
}

#[tokio::test]
#[serial]
async fn test_audit_log_persistence_on_login_failure() {
    let state = common::get_test_db().await;
    let username = unique_username();

    let password_hash = password::hash_password("correct_password").unwrap();
    UserRepo::create(&state.db, CreateUser {
        username: username.clone(),
        email: format!("{}@test.com", username),
        password_hash,
        display_name: None,
        given_name: None,
        family_name: None,
    }).await.unwrap();

    let app = build_app_with_state(state.clone());

    let response = app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::json!({
                "username": username,
                "password": "wrong_password"
            }).to_string()))
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    use backend::model::AuditLogQuery;
    let logs = AuditLogRepo::query(&state.db, AuditLogQuery {
        action: Some("login".to_string()),
        outcome: Some("failure".to_string()),
        limit: Some(100),
        ..Default::default()
    }).await.unwrap();

    let login_failure_logs: Vec<_> = logs.iter()
        .filter(|l| l.target_id == username)
        .collect();
    assert!(!login_failure_logs.is_empty(), "Expected login failure audit log");

    let log = login_failure_logs.first().unwrap();
    assert_eq!(log.outcome, "failure");
    assert!(log.reason_code.is_some());

    cleanup_user(&state, &username).await;
}

#[tokio::test]
#[serial]
async fn test_audit_log_contains_client_info() {
    let state = common::get_test_db().await;
    let username = unique_username();

    let password_hash = password::hash_password("test_password").unwrap();
    let user = UserRepo::create(&state.db, CreateUser {
        username: username.clone(),
        email: format!("{}@test.com", username),
        password_hash,
        display_name: None,
        given_name: None,
        family_name: None,
    }).await.unwrap();

    let audit_log = AuditLogRepo::create(&state.db, CreateAuditLog::success(
        "token_issued",
        "access_token",
        "test-token-id",
    ).with_actor(user.id).with_correlation_id("test-corr-id")).await.unwrap();

    assert_eq!(audit_log.actor_user_id, Some(user.id));
    assert_eq!(audit_log.action, "token_issued");
    assert_eq!(audit_log.outcome, "success");

    cleanup_user(&state, &username).await;
}

// ===== 健康检查测试 =====

#[tokio::test]
#[serial]
async fn test_liveness_returns_alive() {
    let state = common::get_test_db().await;
    let app = build_app_with_state(state);

    let response = app.oneshot(
        Request::builder()
            .method("GET")
            .uri("/health/live")
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["status"], "alive");
}

#[tokio::test]
#[serial]
async fn test_readiness_checks_database_connection() {
    let state = common::get_test_db().await;
    let app = build_app_with_state(state);

    let response = app.oneshot(
        Request::builder()
            .method("GET")
            .uri("/health/ready")
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(body["status"], "ready");
    assert!(body["database"].is_object());
    assert_eq!(body["database"]["status"], "up");
}

#[tokio::test]
#[serial]
async fn test_readiness_checks_jwk_availability() {
    let state = common::get_test_db().await;
    let app = build_app_with_state(state);

    let response = app.oneshot(
        Request::builder()
            .method("GET")
            .uri("/health/ready")
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert!(body["keys"].is_object());
    assert_eq!(body["keys"]["status"], "up");
}

// ===== Prometheus 指标测试 =====

#[tokio::test]
#[serial]
async fn test_auth_request_metrics_incremented() {
    use backend::metrics::AUTH_REQUESTS_TOTAL;

    let state = common::get_test_db().await;

    let before = AUTH_REQUESTS_TOTAL.get();

    let app = build_app_with_state(state.clone());

    let _response = app.oneshot(
        Request::builder()
            .method("GET")
            .uri("/authorize?client_id=test&redirect_uri=http://localhost/callback&response_type=code&scope=openid&code_challenge=test&code_challenge_method=S256")
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    let after = AUTH_REQUESTS_TOTAL.get();
    assert!(after > before, "Expected auth requests metric to increment");

    cleanup_user(&state, &unique_username()).await;
}

#[tokio::test]
#[serial]
async fn test_token_issued_metrics_incremented() {
    use backend::metrics::TOKEN_ISSUED_TOTAL;

    let state = common::get_test_db().await;
    let username = unique_username();

    let password_hash = password::hash_password("test_password").unwrap();
    let _user = UserRepo::create(&state.db, CreateUser {
        username: username.clone(),
        email: format!("{}@test.com", username),
        password_hash,
        display_name: None,
        given_name: None,
        family_name: None,
    }).await.unwrap();

    let app = build_app_with_state(state.clone());

    let response = app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::json!({
                "username": username,
                "password": "test_password"
            }).to_string()))
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let _after = TOKEN_ISSUED_TOTAL.get();

    cleanup_user(&state, &username).await;
}

#[tokio::test]
#[serial]
async fn test_metrics_registry_is_accessible() {
    use backend::metrics::REGISTRY;

    let metrics = REGISTRY.gather();
    assert!(!metrics.is_empty(), "Expected metrics to be registered");

    let metric_names: Vec<_> = metrics.iter()
        .map(|m| m.get_name())
        .collect();

    assert!(metric_names.contains(&"oidc_auth_requests_total"), "Expected auth requests metric");
    assert!(metric_names.contains(&"oidc_token_issued_total"), "Expected token issued metric");
    assert!(metric_names.contains(&"oidc_replay_detected_total"), "Expected replay detected metric");
}

#[tokio::test]
#[serial]
async fn test_session_metrics_tracked() {
    use backend::metrics::SESSION_ACTIVE_TOTAL;

    let state = common::get_test_db().await;
    let username = unique_username();

    let password_hash = password::hash_password("test_password").unwrap();
    let user = UserRepo::create(&state.db, CreateUser {
        username: username.clone(),
        email: format!("{}@test.com", username),
        password_hash,
        display_name: None,
        given_name: None,
        family_name: None,
    }).await.unwrap();

    let session = SessionRepo::create(&state.db, CreateSession::new(
        user.id,
        Some("127.0.0.1".to_string()),
        Some("test-agent".to_string()),
    )).await.unwrap();

    let current = SESSION_ACTIVE_TOTAL.get();
    assert!(current >= 1.0, "Expected at least one active session metric");

    SessionRepo::delete(&state.db, &session.session_id).await.ok();
    cleanup_user(&state, &username).await;
}

// ===== 结构化审计日志字段测试 =====

#[tokio::test]
#[serial]
async fn test_audit_log_has_required_fields() {
    let state = common::get_test_db().await;
    let username = unique_username();

    let password_hash = password::hash_password("test_password").unwrap();
    let user = UserRepo::create(&state.db, CreateUser {
        username: username.clone(),
        email: format!("{}@test.com", username),
        password_hash,
        display_name: None,
        given_name: None,
        family_name: None,
    }).await.unwrap();

    let create_log = CreateAuditLog::success(
        "test_action",
        "test_target",
        "test-id",
    )
    .with_actor(user.id)
    .with_correlation_id("corr-123")
    .with_ip("192.168.1.1")
    .with_user_agent("TestAgent/1.0")
    .with_metadata(serde_json::json!({"key": "value"}));

    let log = AuditLogRepo::create(&state.db, create_log).await.unwrap();

    assert_eq!(log.actor_user_id, Some(user.id));
    assert_eq!(log.correlation_id, "corr-123");
    assert_eq!(log.action, "test_action");
    assert_eq!(log.target_type, "test_target");
    assert_eq!(log.target_id, "test-id");
    assert_eq!(log.outcome, "success");
    assert_eq!(log.ip_address, Some("192.168.1.1".to_string()));
    assert_eq!(log.user_agent, Some("TestAgent/1.0".to_string()));
    assert_eq!(log.metadata, Some(serde_json::json!({"key": "value"})));

    cleanup_user(&state, &username).await;
}

#[tokio::test]
#[serial]
async fn test_audit_log_failure_has_reason_code() {
    let state = common::get_test_db().await;
    let username = unique_username();

    let password_hash = password::hash_password("test_password").unwrap();
    let user = UserRepo::create(&state.db, CreateUser {
        username: username.clone(),
        email: format!("{}@test.com", username),
        password_hash,
        display_name: None,
        given_name: None,
        family_name: None,
    }).await.unwrap();

    let create_log = CreateAuditLog::failure(
        "login",
        "user_session",
        &user.id.to_string(),
        "invalid_credentials",
    );

    let log = AuditLogRepo::create(&state.db, create_log).await.unwrap();

    assert_eq!(log.outcome, "failure");
    assert_eq!(log.reason_code, Some("invalid_credentials".to_string()));

    cleanup_user(&state, &username).await;
}

#[tokio::test]
#[serial]
async fn test_audit_log_query_by_action() {
    let state = common::get_test_db().await;
    let username = unique_username();

    let password_hash = password::hash_password("test_password").unwrap();
    let user = UserRepo::create(&state.db, CreateUser {
        username: username.clone(),
        email: format!("{}@test.com", username),
        password_hash,
        display_name: None,
        given_name: None,
        family_name: None,
    }).await.unwrap();

    for i in 0..5 {
        AuditLogRepo::create(&state.db, CreateAuditLog::success(
            format!("action_{}", i),
            "test_target",
            "test-id",
        ).with_actor(user.id)).await.unwrap();
    }

    use backend::model::AuditLogQuery;
    let logs = AuditLogRepo::query(&state.db, AuditLogQuery {
        actor_user_id: Some(user.id),
        action: Some("action_2".to_string()),
        ..Default::default()
    }).await.unwrap();

    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].action, "action_2");

    cleanup_user(&state, &username).await;
}
