// Logout Flow Tests
//
// 测试 RP-Initiated Logout 和 consent revoke 逻辑

use sqlx::PgPool;
use time::Duration;

use backend::crypto;
use backend::model::{Client, CreateClient, CreateRefreshToken, User};
use backend::repo::{ClientRepo, ConsentRepo, RefreshTokenRepo, SessionRepo, UserRepo};
use backend::service::{AuthService, ConsentService, LogoutService};

#[sqlx::test]
async fn test_rp_initiated_logout_success(pool: PgPool) -> anyhow::Result<()> {
    let user = create_test_user(&pool, "logout_user").await?;
    let _client = create_test_client(&pool).await?;

    let session = SessionRepo::create(
        &pool,
        backend::model::CreateSession {
            user_id: user.id,
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("test-agent".to_string()),
            duration_seconds: 3600,
        },
    )
    .await?;

    let found = SessionRepo::find_by_session_id(&pool, &session.session_id).await?;
    assert!(found.is_some());

    LogoutService::logout_by_session(&pool, &session.session_id).await?;

    let found = SessionRepo::find_by_session_id(&pool, &session.session_id).await?;
    assert!(found.is_none());

    Ok(())
}

#[sqlx::test]
async fn test_rp_initiated_logout_with_id_token_hint(pool: PgPool) -> anyhow::Result<()> {
    let user = create_test_user(&pool, "logout_hint_user").await?;
    let _client = create_test_client(&pool).await?;

    let _session = SessionRepo::create(
        &pool,
        backend::model::CreateSession {
            user_id: user.id,
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("test-agent".to_string()),
            duration_seconds: 3600,
        },
    )
    .await?;

    // 无效的 id_token_hint 格式
    let result =
        LogoutService::validate_id_token_hint::<()>(&pool, "invalid-token-hint", None::<&str>)
            .await;
    assert!(result.is_err());

    // 空的 id_token_hint 应该也可以（可选参数）
    let result = LogoutService::validate_id_token_hint::<()>(&pool, "", None::<&str>).await;
    assert!(result.is_err());

    Ok(())
}

#[sqlx::test]
async fn test_consent_revoke_removes_refresh_tokens(pool: PgPool) -> anyhow::Result<()> {
    let user = create_test_user(&pool, "revoke_test_user").await?;
    let client = create_test_client(&pool).await?;

    ConsentRepo::create(
        &pool,
        backend::model::CreateConsent {
            user_id: user.id,
            client_id: client.id,
            scope: "openid profile offline_access".to_string(),
        },
    )
    .await?;

    let token1_hash = crypto::hash_token("refresh-token-1");
    let token2_hash = crypto::hash_token("refresh-token-2");

    RefreshTokenRepo::create(
        &pool,
        CreateRefreshToken {
            token_hash: token1_hash.clone(),
            parent_token_hash: None,
            user_id: user.id,
            client_id: client.id,
            scope: "openid profile offline_access".to_string(),
            expires_at: time::OffsetDateTime::now_utc() + Duration::hours(24),
        },
    )
    .await?;

    RefreshTokenRepo::create(
        &pool,
        CreateRefreshToken {
            token_hash: token2_hash.clone(),
            parent_token_hash: Some(token1_hash.clone()),
            user_id: user.id,
            client_id: client.id,
            scope: "openid profile offline_access".to_string(),
            expires_at: time::OffsetDateTime::now_utc() + Duration::hours(24),
        },
    )
    .await?;

    let rt1 = RefreshTokenRepo::find_by_hash(&pool, &token1_hash).await?;
    let rt2 = RefreshTokenRepo::find_by_hash(&pool, &token2_hash).await?;
    assert!(rt1.is_some());
    assert!(rt2.is_some());

    ConsentService::revoke_consent(&pool, user.id, client.id).await?;

    let rt1 = RefreshTokenRepo::find_by_hash(&pool, &token1_hash).await?;
    let rt2 = RefreshTokenRepo::find_by_hash(&pool, &token2_hash).await?;
    assert!(rt1.unwrap().revoked_at.is_some());
    assert!(rt2.unwrap().revoked_at.is_some());

    let consent = ConsentRepo::find_by_user_and_client(&pool, user.id, client.id).await?;
    assert!(consent.unwrap().revoked_at.is_some());

    Ok(())
}

#[sqlx::test]
async fn test_logout_all_user_sessions(pool: PgPool) -> anyhow::Result<()> {
    let user = create_test_user(&pool, "multi_session_user").await?;

    let session1 = SessionRepo::create(
        &pool,
        backend::model::CreateSession {
            user_id: user.id,
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("Chrome".to_string()),
            duration_seconds: 3600,
        },
    )
    .await?;

    let session2 = SessionRepo::create(
        &pool,
        backend::model::CreateSession {
            user_id: user.id,
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: Some("Firefox".to_string()),
            duration_seconds: 3600,
        },
    )
    .await?;

    let s1 = SessionRepo::find_by_session_id(&pool, &session1.session_id).await?;
    let s2 = SessionRepo::find_by_session_id(&pool, &session2.session_id).await?;
    assert!(s1.is_some());
    assert!(s2.is_some());

    let count = AuthService::logout_all_sessions(&pool, user.id).await?;
    assert_eq!(count, 2);

    let s1 = SessionRepo::find_by_session_id(&pool, &session1.session_id).await?;
    let s2 = SessionRepo::find_by_session_id(&pool, &session2.session_id).await?;
    assert!(s1.is_none());
    assert!(s2.is_none());

    Ok(())
}

#[sqlx::test]
async fn test_validate_post_logout_redirect_uri_valid(pool: PgPool) -> anyhow::Result<()> {
    let client = create_test_client(&pool).await?;

    let result =
        LogoutService::validate_post_logout_redirect(&pool, &client.id, "https://example.com/")
            .await;
    assert!(result.is_ok());

    Ok(())
}

#[sqlx::test]
async fn test_validate_post_logout_redirect_uri_invalid(pool: PgPool) -> anyhow::Result<()> {
    let client = create_test_client(&pool).await?;

    let result = LogoutService::validate_post_logout_redirect(
        &pool,
        &client.id,
        "https://evil.com/callback",
    )
    .await;
    assert!(result.is_err());

    Ok(())
}

#[sqlx::test]
async fn test_validate_post_logout_redirect_uri_missing_for_client(
    pool: PgPool,
) -> anyhow::Result<()> {
    let client = ClientRepo::create(
        &pool,
        CreateClient {
            client_id: "no-redirect-client".to_string(),
            client_secret_hash: None,
            is_public: false,
            name: "no-redirect-client".to_string(),
            description: None,
            app_url: None,
            redirect_uris: vec!["https://example.com/callback".to_string()],
            post_logout_redirect_uris: None,
            grant_types: vec!["authorization_code".to_string()],
            response_types: vec!["code".to_string()],
            token_endpoint_auth_method: "client_secret_post".to_string(),
        },
    )
    .await?;

    let result = LogoutService::validate_post_logout_redirect(
        &pool,
        &client.id,
        "https://example.com/callback",
    )
    .await;
    assert!(result.is_err());

    Ok(())
}

#[sqlx::test]
async fn test_session_not_found_returns_ok(pool: PgPool) -> anyhow::Result<()> {
    // 即使 session 不存在，logout_by_session 也应该返回成功（幂等性）
    let result = LogoutService::logout_by_session(&pool, "non-existent-session").await;
    assert!(result.is_ok());

    Ok(())
}

// ============================================================
// Helper Functions
// ============================================================

async fn create_test_user(pool: &PgPool, username: &str) -> anyhow::Result<User> {
    let password_hash = backend::crypto::password::hash_password("TestPassword123!")
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;

    UserRepo::create(
        pool,
        backend::model::CreateUser {
            username: username.to_string(),
            email: format!("{}@example.com", username),
            password_hash: Some(password_hash),
            display_name: None,
            given_name: Some("Test".to_string()),
            family_name: Some("User".to_string()),
        },
    )
    .await
    .map_err(|e| anyhow::anyhow!("Failed to create user: {}", e))
}

async fn create_test_client(pool: &PgPool) -> anyhow::Result<Client> {
    ClientRepo::create(
        pool,
        CreateClient {
            client_id: format!("client-{}", uuid::Uuid::new_v4()),
            client_secret_hash: None,
            is_public: false,
            name: "Test Logout Client".to_string(),
            description: None,
            app_url: None,
            redirect_uris: vec!["https://example.com/callback".to_string()],
            post_logout_redirect_uris: Some(vec!["https://example.com/".to_string()]),
            grant_types: vec!["authorization_code".to_string()],
            response_types: vec!["code".to_string()],
            token_endpoint_auth_method: "client_secret_post".to_string(),
        },
    )
    .await
    .map_err(|e| anyhow::anyhow!("Failed to create client: {}", e))
}
