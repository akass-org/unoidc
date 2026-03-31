use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use backend::{build_app_with_state, config::Config, AppState};
use http_body_util::BodyExt;
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

fn test_config() -> Config {
    Config::default()
}

fn test_pool(config: &Config) -> sqlx::PgPool {
    PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy(&config.database_url)
        .expect("Failed to create pool")
}

fn test_app(config: Config) -> axum::Router {
    let pool = test_pool(&config);
    let state = AppState::new(config, pool);
    build_app_with_state(state)
}

fn test_app_with_remote_addr(config: Config, remote_addr: Option<String>) -> axum::Router {
    use axum::Extension;
    let pool = test_pool(&config);
    let state = AppState::new(config.clone(), pool);
    let rate_limiter = backend::middleware::create_rate_limiter(
        config.rate_limit_max_requests,
        config.rate_limit_window_secs,
        config.rate_limit_login_max_requests,
        config.rate_limit_login_window_secs,
        config.rate_limit_token_max_requests,
        config.rate_limit_token_window_secs,
        config.trusted_proxy_ips.clone(),
    );

    // 重新构建 router，插入自定义的 remote_addr
    use axum::routing::{get, post};
    use backend::middleware::{
        request_context_middleware, rate_limit_middleware,
        create_cors_layer, CorsConfig, csrf_middleware,
    };

    let cors_config = CorsConfig {
        allowed_origins: config.cors_allowed_origins.clone(),
    };
    let cors_layer = create_cors_layer(&cors_config);

    axum::Router::new()
        .route("/health/live", get(backend::handler::health::liveness))
        .route("/health/ready", get(backend::handler::health::readiness))
        .route("/api/v1/auth/login", post(backend::handler::auth::login))
        .route("/api/v1/auth/logout", post(backend::handler::auth::logout))
        .route("/api/v1/auth/register", post(backend::handler::auth::register))
        .route("/api/v1/auth/forgot-password", post(backend::handler::auth::forgot_password))
        .route("/.well-known/openid-configuration", get(backend::handler::oidc::discovery))
        .route("/jwks.json", get(backend::handler::oidc::jwks))
        .route("/authorize", get(backend::handler::oidc::authorize_get))
        .route("/authorize/consent", post(backend::handler::oidc::authorize_consent))
        .route("/token", post(backend::handler::oidc::token))
        .route("/userinfo", get(backend::handler::oidc::userinfo))
        .route("/logout", get(backend::handler::oidc::logout))
        .layer(axum::middleware::from_fn(csrf_middleware))
        .layer(axum::middleware::from_fn(rate_limit_middleware))
        .layer(Extension(rate_limiter))
        .layer(Extension(remote_addr))
        .layer(axum::Extension::<Option<std::net::SocketAddr>>(None))
        .layer(axum::middleware::from_fn(request_context_middleware))
        .layer(cors_layer)
        .with_state(state)
}

#[tokio::test]
async fn test_cors_allows_configured_origin() {
    let config = test_config();
    let app = test_app(config);

    let response = app.oneshot(
        Request::builder()
            .method("OPTIONS")
            .uri("/.well-known/openid-configuration")
            .header("origin", "http://localhost:5173")
            .header("access-control-request-method", "GET")
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let allow_origin = response.headers().get("access-control-allow-origin");
    assert!(allow_origin.is_some());
    assert_eq!(allow_origin.unwrap(), "http://localhost:5173");
}

#[tokio::test]
async fn test_cors_rejects_unknown_origin() {
    let config = test_config();
    let app = test_app(config);

    let response = app.oneshot(
        Request::builder()
            .method("OPTIONS")
            .uri("/.well-known/openid-configuration")
            .header("origin", "https://evil.example.com")
            .header("access-control-request-method", "GET")
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    let allow_origin = response.headers().get("access-control-allow-origin");
    assert!(allow_origin.is_none());
}

#[tokio::test]
async fn test_cors_exposes_custom_headers() {
    let config = test_config();
    let app = test_app(config);

    let response = app.oneshot(
        Request::builder()
            .method("GET")
            .uri("/.well-known/openid-configuration")
            .header("origin", "http://localhost:5173")
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    let expose = response.headers().get("access-control-expose-headers");
    assert!(expose.is_some());
    let expose_str = expose.unwrap().to_str().unwrap();
    assert!(expose_str.contains("x-request-id"));
    assert!(expose_str.contains("x-correlation-id"));
}

#[tokio::test]
async fn test_csrf_allows_exempt_login_path() {
    let config = test_config();
    let app = test_app(config);

    let response = app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .header("content-type", "application/json")
            .body(Body::from(json!({
                "username": "test",
                "password": "test"
            }).to_string()))
            .unwrap(),
    ).await.unwrap();

    assert_ne!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_csrf_allows_exempt_token_path() {
    let config = test_config();
    let app = test_app(config);

    let response = app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/token")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from("grant_type=authorization_code&code=test"))
            .unwrap(),
    ).await.unwrap();

    assert_ne!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_csrf_rejects_post_without_token() {
    let config = test_config();
    let app = test_app(config);

    let response = app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/register")
            .header("content-type", "application/json")
            .body(Body::from(json!({
                "username": "test",
                "email": "test@test.com",
                "password": "password123"
            }).to_string()))
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("CSRF"));
}

#[tokio::test]
async fn test_csrf_allows_matching_tokens() {
    let config = test_config();
    let app = test_app(config);

    let csrf_token = "test-csrf-token-value";

    let response = app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/register")
            .header("content-type", "application/json")
            .header("cookie", format!("unoidc_csrf={}", csrf_token))
            .header("x-csrf-token", csrf_token)
            .body(Body::from(json!({
                "username": "test",
                "email": "test@test.com",
                "password": "password123"
            }).to_string()))
            .unwrap(),
    ).await.unwrap();

    assert_ne!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_csrf_rejects_mismatched_tokens() {
    let config = test_config();
    let app = test_app(config);

    let response = app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/register")
            .header("content-type", "application/json")
            .header("cookie", "unoidc_csrf=token-a")
            .header("x-csrf-token", "token-b")
            .body(Body::from(json!({
                "username": "test",
                "email": "test@test.com",
                "password": "password123"
            }).to_string()))
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_csrf_allows_get_requests() {
    let config = test_config();
    let app = test_app(config);

    let response = app.oneshot(
        Request::builder()
            .method("GET")
            .uri("/.well-known/openid-configuration")
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_rate_limit_includes_retry_after_header() {
    let mut config = test_config();
    config.rate_limit_max_requests = 2;
    config.rate_limit_window_secs = 60;
    let app = test_app(config);

    for _ in 0..2 {
        let _ = app.clone().oneshot(
            Request::builder()
                .method("GET")
                .uri("/.well-known/openid-configuration")
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();
    }

    let response = app.oneshot(
        Request::builder()
            .method("GET")
            .uri("/.well-known/openid-configuration")
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    let retry_after = response.headers().get("retry-after");
    assert!(retry_after.is_some());
}

#[tokio::test]
async fn test_rate_limit_login_tier_stricter() {
    let mut config = test_config();
    config.rate_limit_login_max_requests = 2;
    config.rate_limit_login_window_secs = 60;
    config.rate_limit_max_requests = 100;
    let app = test_app(config);

    for _ in 0..2 {
        let _ = app.clone().oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(json!({"username": "a", "password": "b"}).to_string()))
                .unwrap(),
        ).await.unwrap();
    }

    let response = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .header("content-type", "application/json")
            .body(Body::from(json!({"username": "a", "password": "b"}).to_string()))
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

    let other_response = app.oneshot(
        Request::builder()
            .method("GET")
            .uri("/.well-known/openid-configuration")
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    assert_eq!(other_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_rate_limit_token_tier_stricter() {
    let mut config = test_config();
    config.rate_limit_token_max_requests = 2;
    config.rate_limit_token_window_secs = 60;
    config.rate_limit_max_requests = 100;
    let app = test_app(config);

    for _ in 0..2 {
        let _ = app.clone().oneshot(
            Request::builder()
                .method("POST")
                .uri("/token")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("grant_type=authorization_code&code=test"))
                .unwrap(),
        ).await.unwrap();
    }

    let response = app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/token")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from("grant_type=authorization_code&code=test"))
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn test_ip_from_x_forwarded_for() {
    let mut config = test_config();
    config.rate_limit_max_requests = 2;
    config.rate_limit_window_secs = 60;
    // 配置可信代理以允许 X-Forwarded-For 头被信任
    config.trusted_proxy_ips = vec!["127.0.0.1".to_string()];
    let app = test_app_with_remote_addr(config, Some("127.0.0.1:12345".to_string()));

    for _ in 0..2 {
        let _ = app.clone().oneshot(
            Request::builder()
                .method("GET")
                .uri("/.well-known/openid-configuration")
                .header("x-forwarded-for", "1.2.3.4")
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();
    }

    let response = app.clone().oneshot(
        Request::builder()
            .method("GET")
            .uri("/.well-known/openid-configuration")
            .header("x-forwarded-for", "1.2.3.4")
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

    let other_ip_response = app.oneshot(
        Request::builder()
            .method("GET")
            .uri("/.well-known/openid-configuration")
            .header("x-forwarded-for", "5.6.7.8")
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    assert_eq!(other_ip_response.status(), StatusCode::OK);
}
