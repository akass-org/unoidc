// 登录认证测试
//
// 测试登录成功、失败计数和账户锁定功能

use backend::{
    build_app_with_state, config::Config,
    db, model::{CreateSession, CreateUser},
    repo::{SessionRepo, UserRepo},
    crypto::password,
    AppState,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
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

/// 清理测试数据
async fn cleanup_test_data(state: &AppState) {
    sqlx::query("DELETE FROM user_sessions").execute(&state.db).await.ok();
    sqlx::query("DELETE FROM users").execute(&state.db).await.ok();
}

#[tokio::test]
async fn test_successful_login() {
    let state = get_test_db().await;
    cleanup_test_data(&state).await;

    // 创建测试用户
    let password = "test_password_123";
    let password_hash = password::hash_password(password).unwrap();
    let user = UserRepo::create(&state.db, CreateUser {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password_hash,
        display_name: None,
        given_name: None,
        family_name: None,
    }).await.unwrap();

    let app = build_app_with_state(state.clone());

    // 测试登录请求
    let response = app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .header("content-type", "application/json")
            .body(Body::from(json!({
                "username": "testuser",
                "password": "test_password_123"
            }).to_string()))
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 验证用户的失败计数被重置
    let updated_user = UserRepo::find_by_id(&state.db, user.id).await.unwrap().unwrap();
    assert_eq!(updated_user.failed_login_attempts, 0);
    assert!(updated_user.last_login_at.is_some());

    cleanup_test_data(&state).await;
}

#[tokio::test]
async fn test_failed_login_increments_counter() {
    let state = get_test_db().await;
    cleanup_test_data(&state).await;

    // 创建测试用户
    let password = "correct_password";
    let password_hash = password::hash_password(password).unwrap();
    let user = UserRepo::create(&state.db, CreateUser {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password_hash,
        display_name: None,
        given_name: None,
        family_name: None,
    }).await.unwrap();

    let app = build_app_with_state(state.clone());

    // 测试错误密码登录
    let response = app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .header("content-type", "application/json")
            .body(Body::from(json!({
                "username": "testuser",
                "password": "wrong_password"
            }).to_string()))
            .unwrap(),
    ).await.unwrap();

    // 应该返回401未授权
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // 验证失败计数增加
    let updated_user = UserRepo::find_by_id(&state.db, user.id).await.unwrap().unwrap();
    assert_eq!(updated_user.failed_login_attempts, 1);

    cleanup_test_data(&state).await;
}

#[tokio::test]
async fn test_account_lockout_after_repeated_failures() {
    let state = get_test_db().await;
    cleanup_test_data(&state).await;

    // 创建测试用户
    let password = "correct_password";
    let password_hash = password::hash_password(password).unwrap();
    let user = UserRepo::create(&state.db, CreateUser {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password_hash,
        display_name: None,
        given_name: None,
        family_name: None,
    }).await.unwrap();

    // 连续失败5次登录（锁定阈值是5）
    for _ in 0..5 {
        let app = build_app_with_state(state.clone());
        let _response = app.oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(json!({
                    "username": "testuser",
                    "password": "wrong_password"
                }).to_string()))
                .unwrap(),
        ).await.unwrap();
    }

    // 验证账户被锁定
    let locked_user = UserRepo::find_by_id(&state.db, user.id).await.unwrap().unwrap();
    assert!(locked_user.is_locked());

    // 即使使用正确密码，也应该登录失败
    let app = build_app_with_state(state.clone());
    let response = app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .header("content-type", "application/json")
            .body(Body::from(json!({
                "username": "testuser",
                "password": "correct_password"
            }).to_string()))
            .unwrap(),
    ).await.unwrap();

    // 应该返回403禁止访问（账户被锁定）
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    cleanup_test_data(&state).await;
}

#[tokio::test]
async fn test_login_creates_session() {
    let state = get_test_db().await;
    cleanup_test_data(&state).await;

    // 创建测试用户
    let password = "test_password";
    let password_hash = password::hash_password(password).unwrap();
    let user = UserRepo::create(&state.db, CreateUser {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password_hash,
        display_name: None,
        given_name: None,
        family_name: None,
    }).await.unwrap();

    let app = build_app_with_state(state.clone());

    // 登录
    let response = app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .header("content-type", "application/json")
            .body(Body::from(json!({
                "username": "testuser",
                "password": "test_password"
            }).to_string()))
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 验证会话被创建
    let sessions = SessionRepo::find_user_sessions(&state.db, user.id).await.unwrap();
    assert_eq!(sessions.len(), 1);

    cleanup_test_data(&state).await;
}

#[tokio::test]
async fn test_logout_destroys_session() {
    let state = get_test_db().await;
    cleanup_test_data(&state).await;

    // 创建测试用户
    let password = "test_password";
    let password_hash = password::hash_password(password).unwrap();
    let user = UserRepo::create(&state.db, CreateUser {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password_hash,
        display_name: None,
        given_name: None,
        family_name: None,
    }).await.unwrap();

    // 创建会话
    let session = SessionRepo::create(&state.db, CreateSession::new(
        user.id,
        Some("127.0.0.1".to_string()),
        Some("test-agent".to_string()),
    )).await.unwrap();

    let app = build_app_with_state(state.clone());

    // 登出
    let response = app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/logout")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", session.session_id))
            .body(Body::from(json!({}).to_string()))
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 验证会话被删除
    let deleted_session = SessionRepo::find_by_session_id(&state.db, &session.session_id).await.unwrap();
    assert!(deleted_session.is_none());

    cleanup_test_data(&state).await;
}
