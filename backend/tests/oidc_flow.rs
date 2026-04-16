mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use backend::{
    build_app_with_state,
    crypto::{self, password},
    model::{CreateClient, CreateGroup, CreateSession, CreateUser},
    repo::{ClientRepo, GroupRepo, SessionRepo, UserRepo},
    AppState,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use http_body_util::BodyExt;
use serde_json::Value;
use serial_test::serial;
use tower::ServiceExt;
use uuid::Uuid;

async fn cleanup_test_data(state: &AppState) {
    sqlx::query(
        "DELETE FROM user_groups WHERE user_id IN (SELECT id FROM users WHERE username LIKE 'oidc_user_%' OR username LIKE 'oidc_logout_user_%') OR group_id IN (SELECT id FROM groups WHERE name LIKE 'oidc-group-%' OR name LIKE 'oidc-logout-group-%')",
    )
        .execute(&state.db)
        .await
        .ok();
    sqlx::query(
        "DELETE FROM client_groups WHERE client_id IN (SELECT id FROM clients WHERE client_id LIKE 'oidc-client-%') OR group_id IN (SELECT id FROM groups WHERE name LIKE 'oidc-group-%' OR name LIKE 'oidc-logout-group-%')",
    )
        .execute(&state.db)
        .await
        .ok();
    sqlx::query(
        "DELETE FROM user_consents WHERE user_id IN (SELECT id FROM users WHERE username LIKE 'oidc_user_%' OR username LIKE 'oidc_logout_user_%') OR client_id IN (SELECT id FROM clients WHERE client_id LIKE 'oidc-client-%')",
    )
        .execute(&state.db)
        .await
        .ok();
    sqlx::query(
        "DELETE FROM refresh_tokens WHERE user_id IN (SELECT id FROM users WHERE username LIKE 'oidc_user_%' OR username LIKE 'oidc_logout_user_%') OR client_id IN (SELECT id FROM clients WHERE client_id LIKE 'oidc-client-%')",
    )
        .execute(&state.db)
        .await
        .ok();
    sqlx::query(
        "DELETE FROM authorization_codes WHERE user_id IN (SELECT id FROM users WHERE username LIKE 'oidc_user_%' OR username LIKE 'oidc_logout_user_%') OR client_id IN (SELECT id FROM clients WHERE client_id LIKE 'oidc-client-%')",
    )
        .execute(&state.db)
        .await
        .ok();
    sqlx::query(
        "DELETE FROM user_sessions WHERE user_id IN (SELECT id FROM users WHERE username LIKE 'oidc_user_%' OR username LIKE 'oidc_logout_user_%')",
    )
        .execute(&state.db)
        .await
        .ok();
    sqlx::query("DELETE FROM clients WHERE client_id LIKE 'oidc-client-%'")
        .execute(&state.db)
        .await
        .ok();
    sqlx::query(
        "DELETE FROM groups WHERE name LIKE 'oidc-group-%' OR name LIKE 'oidc-logout-group-%'",
    )
    .execute(&state.db)
    .await
    .ok();
    sqlx::query(
        "DELETE FROM users WHERE username LIKE 'oidc_user_%' OR username LIKE 'oidc_logout_user_%'",
    )
    .execute(&state.db)
    .await
    .ok();
}

async fn create_test_user(state: &AppState, username: &str) -> backend::model::User {
    let password_hash = password::hash_password("password123").unwrap();
    UserRepo::create(
        &state.db,
        CreateUser {
            username: username.to_string(),
            email: format!("{}@example.com", username),
            password_hash: Some(password_hash),
            display_name: Some("Test User".to_string()),
            given_name: Some("Test".to_string()),
            family_name: Some("User".to_string()),
        },
    )
    .await
    .unwrap()
}

async fn create_test_client(state: &AppState, client_id: &str) -> backend::model::Client {
    let client_secret_hash = Some(password::hash_client_secret("secret123").unwrap());
    ClientRepo::create(
        &state.db,
        CreateClient {
            client_id: client_id.to_string(),
            client_secret_hash,
            is_public: false,
            name: format!("{} Client", client_id),
            description: Some("OIDC integration test client".to_string()),
            app_url: None,
            redirect_uris: vec!["http://localhost:5173/callback".to_string()],
            post_logout_redirect_uris: Some(vec!["http://localhost:5173/logout".to_string()]),
            grant_types: vec!["authorization_code".to_string()],
            response_types: vec!["code".to_string()],
            token_endpoint_auth_method: "client_secret_basic".to_string(),
        },
    )
    .await
    .unwrap()
}

async fn create_logged_in_app_user(
    state: &AppState,
    username: &str,
) -> (backend::model::User, backend::model::Session) {
    let user = create_test_user(state, username).await;
    let session = SessionRepo::create(
        &state.db,
        CreateSession::new(
            user.id,
            Some("127.0.0.1".to_string()),
            Some("test-agent".to_string()),
        ),
    )
    .await
    .unwrap();
    (user, session)
}

fn session_cookie_value(session_id: &str, secret: &str) -> String {
    let signature = crypto::sign_session(session_id, secret).unwrap();
    format!("{}.{}", session_id, signature)
}

#[tokio::test]
#[serial]
async fn test_oidc_authorize_token_refresh_flow() {
    let state = common::get_test_db().await;
    cleanup_test_data(&state).await;

    let username = format!("oidc_user_{}", Uuid::new_v4().as_simple());
    let client_id = format!("oidc-client-{}", Uuid::new_v4().as_simple());
    let user_group = GroupRepo::create(
        &state.db,
        CreateGroup {
            name: format!("oidc-group-{}", Uuid::new_v4().as_simple()),
            description: Some("OIDC test group".to_string()),
        },
    )
    .await
    .unwrap();

    let (user, session) = create_logged_in_app_user(&state, &username).await;
    GroupRepo::add_user_to_group(&state.db, user.id, user_group.id)
        .await
        .unwrap();

    let client = create_test_client(&state, &client_id).await;
    ClientRepo::add_client_to_group(&state.db, client.id, user_group.id)
        .await
        .unwrap();

    let verifier = crypto::generate_pkce_code_verifier().unwrap();
    let challenge = crypto::hash_token(&verifier);
    let csrf = crypto::generate_csrf_token().unwrap();
    let cookie_value = session_cookie_value(&session.session_id, &state.config.session_secret);
    let app = build_app_with_state(state.clone());

    let authorize_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/authorize?client_id={}&redirect_uri={}&response_type=code&scope=openid%20profile%20offline_access&state=test-state&code_challenge={}&code_challenge_method=S256&nonce=test-nonce",
                    client.client_id,
                    urlencoding::encode("http://localhost:5173/callback"),
                    challenge,
                ))
                .header("cookie", format!("unoidc_session={}", cookie_value))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(authorize_response.status(), StatusCode::OK);
    let authorize_body = authorize_response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let authorize_json: Value = serde_json::from_slice(&authorize_body).unwrap();
    assert_eq!(authorize_json["client_id"], client.client_id);
    assert_eq!(authorize_json["requires_consent"], true);

    let consent_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/authorize/consent")
                .header("content-type", "application/json")
                .header(
                    "cookie",
                    format!("unoidc_session={}; unoidc_csrf={}", cookie_value, csrf),
                )
                .header("x-csrf-token", csrf)
                .body(Body::from(
                    serde_json::json!({
                        "client_id": client.client_id,
                        "redirect_uri": "http://localhost:5173/callback",
                        "state": "test-state",
                        "code_challenge": challenge,
                        "code_challenge_method": "S256",
                        "nonce": "test-nonce",
                        "scopes": ["openid", "profile", "offline_access"],
                        "approved": true
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(consent_response.status(), StatusCode::OK);
    let consent_body = consent_response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let consent_json: Value = serde_json::from_slice(&consent_body).unwrap();
    let code = consent_json["code"].as_str().unwrap().to_string();
    assert!(!code.is_empty());

    let basic = STANDARD.encode(format!("{}:{}", client.client_id, "secret123"));
    let token_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/token")
                .header("content-type", "application/x-www-form-urlencoded")
                .header("authorization", format!("Basic {}", basic))
                .body(Body::from(format!(
                    "grant_type=authorization_code&code={}&redirect_uri={}&code_verifier={}",
                    urlencoding::encode(&code),
                    urlencoding::encode("http://localhost:5173/callback"),
                    urlencoding::encode(&verifier),
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(token_response.status(), StatusCode::OK);
    let token_body = token_response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let token_json: Value = serde_json::from_slice(&token_body).unwrap();
    let refresh_token = token_json["refresh_token"].as_str().unwrap().to_string();
    assert!(!refresh_token.is_empty());

    let replay_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/token")
                .header("content-type", "application/x-www-form-urlencoded")
                .header("authorization", format!("Basic {}", basic))
                .body(Body::from(format!(
                    "grant_type=authorization_code&code={}&redirect_uri={}&code_verifier={}",
                    urlencoding::encode(&code),
                    urlencoding::encode("http://localhost:5173/callback"),
                    urlencoding::encode(&verifier),
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(replay_response.status(), StatusCode::UNAUTHORIZED);

    let refresh_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/token")
                .header("content-type", "application/x-www-form-urlencoded")
                .header("authorization", format!("Basic {}", basic))
                .body(Body::from(format!(
                    "grant_type=refresh_token&refresh_token={}",
                    urlencoding::encode(&refresh_token),
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(refresh_response.status(), StatusCode::OK);
    let refresh_body = refresh_response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let refresh_json: Value = serde_json::from_slice(&refresh_body).unwrap();
    assert!(refresh_json["refresh_token"].is_string());

    cleanup_test_data(&state).await;
}

#[tokio::test]
#[serial]
async fn test_oidc_authorize_rejects_invalid_redirect_uri() {
    let state = common::get_test_db().await;
    cleanup_test_data(&state).await;

    let client = create_test_client(
        &state,
        &format!("oidc-client-{}", Uuid::new_v4().as_simple()),
    )
    .await;
    let app = build_app_with_state(state.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/authorize?client_id={}&redirect_uri={}&response_type=code&scope=openid&state=test-state&code_challenge=test&code_challenge_method=S256",
                    client.client_id,
                    urlencoding::encode("http://evil.example/callback"),
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    cleanup_test_data(&state).await;
}

#[tokio::test]
#[serial]
async fn test_oidc_logout_accepts_id_token_hint_with_string_audience() {
    let state = common::get_test_db().await;
    cleanup_test_data(&state).await;

    let username = format!("oidc_logout_user_{}", Uuid::new_v4().as_simple());
    let client_id = format!("oidc-client-{}", Uuid::new_v4().as_simple());
    let user_group = GroupRepo::create(
        &state.db,
        CreateGroup {
            name: format!("oidc-logout-group-{}", Uuid::new_v4().as_simple()),
            description: Some("OIDC logout test group".to_string()),
        },
    )
    .await
    .unwrap();

    let (user, session) = create_logged_in_app_user(&state, &username).await;
    GroupRepo::add_user_to_group(&state.db, user.id, user_group.id)
        .await
        .unwrap();

    let client = create_test_client(&state, &client_id).await;
    ClientRepo::add_client_to_group(&state.db, client.id, user_group.id)
        .await
        .unwrap();

    let verifier = crypto::generate_pkce_code_verifier().unwrap();
    let challenge = crypto::hash_token(&verifier);
    let csrf = crypto::generate_csrf_token().unwrap();
    let cookie_value = session_cookie_value(&session.session_id, &state.config.session_secret);
    let app = build_app_with_state(state.clone());

    let consent_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/authorize/consent")
                .header("content-type", "application/json")
                .header(
                    "cookie",
                    format!("unoidc_session={}; unoidc_csrf={}", cookie_value, csrf),
                )
                .header("x-csrf-token", csrf)
                .body(Body::from(
                    serde_json::json!({
                        "client_id": client.client_id,
                        "redirect_uri": "http://localhost:5173/callback",
                        "state": "logout-state",
                        "code_challenge": challenge,
                        "code_challenge_method": "S256",
                        "nonce": "logout-nonce",
                        "scopes": ["openid", "profile", "offline_access"],
                        "approved": true
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(consent_response.status(), StatusCode::OK);
    let consent_body = consent_response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let consent_json: Value = serde_json::from_slice(&consent_body).unwrap();
    let code = consent_json["code"].as_str().unwrap().to_string();

    let basic = STANDARD.encode(format!("{}:{}", client.client_id, "secret123"));
    let token_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/token")
                .header("content-type", "application/x-www-form-urlencoded")
                .header("authorization", format!("Basic {}", basic))
                .body(Body::from(format!(
                    "grant_type=authorization_code&code={}&redirect_uri={}&code_verifier={}",
                    urlencoding::encode(&code),
                    urlencoding::encode("http://localhost:5173/callback"),
                    urlencoding::encode(&verifier),
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(token_response.status(), StatusCode::OK);
    let token_body = token_response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let token_json: Value = serde_json::from_slice(&token_body).unwrap();
    let id_token = token_json["id_token"].as_str().unwrap().to_string();

    let logout_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/logout?id_token_hint={}&post_logout_redirect_uri={}&state={}",
                    urlencoding::encode(&id_token),
                    urlencoding::encode("http://localhost:5173/logout"),
                    urlencoding::encode("logout-state"),
                ))
                .header("cookie", format!("unoidc_session={}", cookie_value))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(logout_response.status(), StatusCode::FOUND);
    let location = logout_response
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert_eq!(location, "http://localhost:5173/logout?state=logout-state");

    cleanup_test_data(&state).await;
}
