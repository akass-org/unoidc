// UserInfo 端点测试
//
// 测试 bearer token 访问、scope 过滤、用户信息返回

mod common;

use backend::{
    build_app_with_state,
    crypto::jwt::{self, AccessTokenClaims},
    model::{CreateClient, CreateGroup, CreateUser},
    repo::{ClientRepo, GroupRepo, UserRepo},
    service::KeyService,
    AppState,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serial_test::serial;
use tower::ServiceExt;

async fn cleanup_test_data(state: &AppState) {
    sqlx::query("DELETE FROM user_groups")
        .execute(&state.db)
        .await
        .ok();
    sqlx::query("DELETE FROM groups")
        .execute(&state.db)
        .await
        .ok();
    sqlx::query("DELETE FROM refresh_tokens")
        .execute(&state.db)
        .await
        .ok();
    sqlx::query("DELETE FROM authorization_codes")
        .execute(&state.db)
        .await
        .ok();
    sqlx::query("DELETE FROM clients")
        .execute(&state.db)
        .await
        .ok();
    sqlx::query("DELETE FROM user_sessions")
        .execute(&state.db)
        .await
        .ok();
    sqlx::query("DELETE FROM users")
        .execute(&state.db)
        .await
        .ok();
}

/// 创建测试用户
async fn create_test_user(state: &AppState, username: &str, email: &str) -> backend::model::User {
    let password_hash = backend::crypto::password::hash_password("password123").unwrap();
    UserRepo::create(
        &state.db,
        CreateUser {
            username: username.to_string(),
            email: email.to_string(),
            password_hash,
            given_name: Some("Test".to_string()),
            family_name: Some("User".to_string()),
            display_name: None,
        },
    )
    .await
    .unwrap()
}

/// 创建测试客户端
async fn create_test_client(state: &AppState, client_id: &str) -> backend::model::Client {
    let client_secret_hash = backend::crypto::password::hash_password("secret123").ok();
    ClientRepo::create(
        &state.db,
        CreateClient {
            client_id: client_id.to_string(),
            name: format!("{} Client", client_id),
            description: None,
            app_url: None,
            redirect_uris: vec!["http://localhost:8080/callback".to_string()],
            post_logout_redirect_uris: None,
            client_secret_hash,
            is_public: false,
            grant_types: vec!["authorization_code".to_string()],
            response_types: vec!["code".to_string()],
            token_endpoint_auth_method: "client_secret_post".to_string(),
        },
    )
    .await
    .unwrap()
}

/// 创建测试用的 access token
async fn create_access_token(state: &AppState, user_id: uuid::Uuid, client_id: uuid::Uuid, scope: &str) -> String {
    let jwk = KeyService::get_active_key(&state.db, &state.config.private_key_encryption_key)
        .await
        .expect("Failed to get signing key");

    let now = jwt::now_timestamp();
    let claims = AccessTokenClaims {
        iss: state.config.issuer.clone(),
        sub: user_id.to_string(),
        aud: client_id.to_string(),
        iat: now,
        exp: now + 3600,
        jti: jwt::generate_jti().expect("Failed to generate jti"),
        scope: scope.to_string(),
        token_type: "oauth-access-token".to_string(),
    };

    jwt::sign_jwt(&claims, &jwk.kid, &jwk.private_key_pem)
        .expect("Failed to sign access token")
}

#[tokio::test]
#[serial]
async fn test_userinfo_all_scopes() {
    let state = common::get_test_db().await;
    cleanup_test_data(&state).await;

    let user = create_test_user(&state, "testuser", "test@example.com").await;
    let client = create_test_client(&state, "test-client").await;

    // 创建组并加入用户
    let group = GroupRepo::create(
        &state.db,
        CreateGroup {
            name: "admins".to_string(),
            description: Some("Admin team".to_string()),
        },
    )
    .await
    .unwrap();

    GroupRepo::add_user_to_group(&state.db, user.id, group.id)
        .await
        .unwrap();
    let token = create_access_token(
        &state,
        user.id,
        client.id,
        "openid profile email groups",
    )
    .await;

    let app = build_app_with_state(state.clone());
    let response = app
        .oneshot(
            Request::get("/userinfo")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
