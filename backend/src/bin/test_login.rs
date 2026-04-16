// 调试登录功能

use axum::{body::Body, http::Request};
use backend::{
    build_app_with_state, config::Config, crypto::password, db, model::CreateUser, repo::UserRepo,
    AppState,
};
use serde_json::json;
use std::{net::SocketAddr, sync::Arc};
use tower::ServiceExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = db::connect(&database_url).await?;
    db::run_migrations(&pool).await?;

    let config = Config::default();
    let origin_url = webauthn_rs::prelude::Url::parse(&config.webauthn_origin)
        .expect("WEBAUTHN_ORIGIN must be a valid URL");
    let webauthn = webauthn_rs::WebauthnBuilder::new(&config.webauthn_rp_id, &origin_url)
        .expect("Invalid WebAuthn configuration")
        .rp_name("unoidc")
        .build()
        .expect("Failed to build WebAuthn instance");
    let state = Arc::new(AppState {
        config,
        db: pool,
        email_service: None,
        webauthn,
    });

    // 清理数据
    sqlx::query("DELETE FROM user_sessions")
        .execute(&state.db)
        .await
        .ok();
    sqlx::query("DELETE FROM users")
        .execute(&state.db)
        .await
        .ok();

    // 创建测试用户
    let password = "test_password_123";
    let password_hash = password::hash_password(password)?;
    let _user = UserRepo::create(
        &state.db,
        CreateUser {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: Some(password_hash),
            display_name: None,
            given_name: None,
            family_name: None,
        },
    )
    .await?;

    println!("✅ User created successfully");

    // 设置客户端地址
    let addr: SocketAddr = "127.0.0.1:12345".parse().unwrap();
    let app = build_app_with_state(state).layer(axum::Extension(Some(addr)));

    // 测试登录请求
    println!("\n📡 Sending login request...");
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "username": "testuser",
                        "password": "test_password_123"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await?;

    println!("📥 Response status: {}", response.status());

    let body_bytes = axum::body::to_bytes(response.into_body(), 10000).await?;
    let body_str = String::from_utf8(body_bytes.to_vec())?;
    println!("📄 Response body: {}", body_str);

    Ok(())
}
