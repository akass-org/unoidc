// 登录认证测试
//
// 测试登录成功、失败计数和账户锁定功能

mod common;

use backend::{
    build_app_with_state,
    model::{CreateSession, CreateUser},
    repo::{SessionRepo, UserRepo},
    crypto::{self, password},
    AppState,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::json;
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
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user.id)
            .execute(&state.db)
            .await
            .ok();
    }
}

#[tokio::test]
#[serial]
async fn test_successful_login() {
    let state = common::get_test_db().await;
    let username = unique_username();

    let password = "test_password_123";
    let password_hash = password::hash_password(password).unwrap();
    let user = UserRepo::create(&state.db, CreateUser {
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
            .body(Body::from(json!({
                "username": username,
                "password": "test_password_123"
            }).to_string()))
            .unwrap(),
    ).await.unwrap();

    let status = response.status();
    if status != StatusCode::OK {
        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        eprintln!("Login response body: {}", String::from_utf8_lossy(&body_bytes));
    }
    assert_eq!(status, StatusCode::OK);

    let updated_user = UserRepo::find_by_id(&state.db, user.id).await.unwrap().unwrap();
    assert_eq!(updated_user.failed_login_attempts, 0);
    assert!(updated_user.last_login_at.is_some());

    cleanup_user(&state, &username).await;
}

#[tokio::test]
#[serial]
async fn test_failed_login_increments_counter() {
    let state = common::get_test_db().await;
    let username = unique_username();

    let password = "correct_password";
    let password_hash = password::hash_password(password).unwrap();
    let user = UserRepo::create(&state.db, CreateUser {
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
            .body(Body::from(json!({
                "username": username,
                "password": "wrong_password"
            }).to_string()))
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let updated_user = UserRepo::find_by_id(&state.db, user.id).await.unwrap().unwrap();
    assert_eq!(updated_user.failed_login_attempts, 1);

    cleanup_user(&state, &username).await;
}

#[tokio::test]
#[serial]
async fn test_account_lockout_after_repeated_failures() {
    let state = common::get_test_db().await;
    let username = unique_username();

    let password = "correct_password";
    let password_hash = password::hash_password(password).unwrap();
    let user = UserRepo::create(&state.db, CreateUser {
        username: username.clone(),
        email: format!("{}@test.com", username),
        password_hash,
        display_name: None,
        given_name: None,
        family_name: None,
    }).await.unwrap();

    for _ in 0..5 {
        let app = build_app_with_state(state.clone());
        let _response = app.oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(json!({
                    "username": username,
                    "password": "wrong_password"
                }).to_string()))
                .unwrap(),
        ).await.unwrap();
    }

    let locked_user = UserRepo::find_by_id(&state.db, user.id).await.unwrap().unwrap();
    assert!(locked_user.is_locked());

    let app = build_app_with_state(state.clone());
    let response = app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .header("content-type", "application/json")
            .body(Body::from(json!({
                "username": username,
                "password": "correct_password"
            }).to_string()))
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    cleanup_user(&state, &username).await;
}

#[tokio::test]
#[serial]
async fn test_login_creates_session() {
    let state = common::get_test_db().await;
    let username = unique_username();

    let password = "test_password";
    let password_hash = password::hash_password(password).unwrap();
    let user = UserRepo::create(&state.db, CreateUser {
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
            .body(Body::from(json!({
                "username": username,
                "password": "test_password"
            }).to_string()))
            .unwrap(),
    ).await.unwrap();

    let status = response.status();
    if status != StatusCode::OK {
        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        eprintln!("Login response body: {}", String::from_utf8_lossy(&body_bytes));
    }
    assert_eq!(status, StatusCode::OK);

    let sessions = SessionRepo::find_user_sessions(&state.db, user.id).await.unwrap();
    assert_eq!(sessions.len(), 1);

    cleanup_user(&state, &username).await;
}

#[tokio::test]
#[serial]
async fn test_logout_destroys_session() {
    let state = common::get_test_db().await;
    let username = unique_username();

    let password = "test_password";
    let password_hash = password::hash_password(password).unwrap();
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

    let app = build_app_with_state(state.clone());

    // 生成带签名的 cookie
    let signature = crypto::sign_session(&session.session_id, &state.config.session_secret).unwrap();
    let cookie_value = format!("{}.{}", session.session_id, signature);

    let response = app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/logout")
            .header("content-type", "application/json")
            .header("cookie", format!("unoidc_session={}", cookie_value))
            .body(Body::from(json!({}).to_string()))
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let deleted_session = SessionRepo::find_by_session_id(&state.db, &session.session_id).await.unwrap();
    assert!(deleted_session.is_none());

    cleanup_user(&state, &username).await;
}
