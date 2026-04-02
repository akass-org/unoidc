mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use backend::{
    build_app_with_state,
    crypto::{self, password},
    model::{CreateClient, CreateRefreshToken, CreateSession, CreateUser},
    repo::{ClientRepo, RefreshTokenRepo, SessionRepo, UserRepo},
};
use serde_json::json;
use serial_test::serial;
use tower::ServiceExt;
use uuid::Uuid;

fn unique_username() -> String {
    format!("pwd_user_{}", Uuid::new_v4().as_simple())
}

#[tokio::test]
#[serial]
async fn test_change_password_revokes_sessions_and_refresh_tokens() {
    let state = common::get_test_db().await;
    let username = unique_username();
    let plain_password = "OldPassword123!";

    let user = UserRepo::create(
        &state.db,
        CreateUser {
            username: username.clone(),
            email: format!("{}@test.dev", username),
            password_hash: password::hash_password(plain_password).unwrap(),
            display_name: None,
            given_name: None,
            family_name: None,
        },
    )
    .await
    .unwrap();

    let session = SessionRepo::create(
        &state.db,
        CreateSession::new(user.id, Some("127.0.0.1".to_string()), Some("test-agent".to_string())),
    )
    .await
    .unwrap();

    let client = ClientRepo::create(
        &state.db,
        CreateClient {
            client_id: format!("test-client-{}", Uuid::new_v4().as_simple()),
            client_secret_hash: None,
            is_public: true,
            name: "Test Client".to_string(),
            description: Some("for tests".to_string()),
            app_url: Some("http://localhost:5173".to_string()),
            redirect_uris: vec!["http://localhost:5173/callback".to_string()],
            post_logout_redirect_uris: None,
            grant_types: vec!["authorization_code".to_string(), "refresh_token".to_string()],
            response_types: vec!["code".to_string()],
            token_endpoint_auth_method: "none".to_string(),
        },
    )
    .await
    .unwrap();

    let plain_refresh = format!("refresh-{}", Uuid::new_v4());
    let refresh_hash = crypto::hash_token(&plain_refresh);
    RefreshTokenRepo::create(
        &state.db,
        CreateRefreshToken {
            token_hash: refresh_hash.clone(),
            parent_token_hash: None,
            user_id: user.id,
            client_id: client.id,
            scope: "openid profile offline_access".to_string(),
            expires_at: time::OffsetDateTime::now_utc() + time::Duration::days(7),
        },
    )
    .await
    .unwrap();

    let app = build_app_with_state(state.clone());

    let signature = crypto::sign_session(&session.session_id, &state.config.session_secret).unwrap();
    let cookie_value = format!("{}.{}", session.session_id, signature);
    let csrf = "test-csrf-token";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/me/password")
                .header("content-type", "application/json")
                .header(
                    "cookie",
                    format!("unoidc_session={}; unoidc_csrf={}", cookie_value, csrf),
                )
                .header("x-csrf-token", csrf)
                .body(Body::from(
                    json!({
                        "current_password": plain_password,
                        "new_password": "NewPassword123!"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let session_after = SessionRepo::find_by_session_id(&state.db, &session.session_id)
        .await
        .unwrap();
    assert!(session_after.is_none());

    let token_after = RefreshTokenRepo::find_by_hash(&state.db, &refresh_hash)
        .await
        .unwrap()
        .expect("refresh token should still exist but be revoked");
    assert!(token_after.revoked_at.is_some());

    // cleanup
    sqlx::query("DELETE FROM user_sessions WHERE user_id = $1")
        .bind(user.id)
        .execute(&state.db)
        .await
        .ok();
    sqlx::query("DELETE FROM refresh_tokens WHERE user_id = $1")
        .bind(user.id)
        .execute(&state.db)
        .await
        .ok();
    sqlx::query("DELETE FROM client_groups WHERE client_id = $1")
        .bind(client.id)
        .execute(&state.db)
        .await
        .ok();
    sqlx::query("DELETE FROM clients WHERE id = $1")
        .bind(client.id)
        .execute(&state.db)
        .await
        .ok();
    sqlx::query("DELETE FROM user_groups WHERE user_id = $1")
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
