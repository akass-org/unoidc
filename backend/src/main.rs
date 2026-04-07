use backend::{
    build_app_with_state, AppState, config::Config, db, metrics,
    middleware::LogRedactionLayer,
};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志（LogRedactionLayer 替代 fmt::layer()，自带格式化和脱敏）
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=debug,tower_http=debug".into()),
        )
        .with(LogRedactionLayer)
        .init();

    // 加载配置
    let config = Config::from_env()?;
    tracing::info!("Configuration loaded");

    // 连接数据库
    let db = db::connect(&config.database_url).await?;
    tracing::info!("Database connected");

    // 运行迁移
    db::run_migrations(&db).await?;
    tracing::info!("Migrations completed");

    // 初始化 metrics
    metrics::init();
    tracing::info!("Metrics initialized");

    // 初始化活跃会话指标（M-05: 启动时统计实际会话数）
    match backend::repo::SessionRepo::count_active(&db).await {
        Ok(count) => {
            metrics::SESSION_ACTIVE_TOTAL.set(count as f64);
            tracing::info!("Initialized session_active_total to {}", count);
        }
        Err(e) => {
            tracing::warn!("Failed to count active sessions on startup: {}", e);
            // 继续使用默认值 0，不阻止启动
        }
    }

    // 构建应用（注入配置和数据库连接）
    let email_service = if !config.smtp.host.is_empty() {
        Some(backend::service::EmailService::new(
            config.smtp.host.clone(),
            config.smtp.port,
            config.smtp.username.clone(),
            config.smtp.password.clone(),
            config.smtp.from_address.clone(),
            config.smtp.tls,
        ))
    } else {
        tracing::info!("SMTP not configured, email features will be disabled");
        None
    };

    let state = AppState::new(config, db, email_service);
    let app = build_app_with_state(state);

    // 启动服务器
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("Server listening on {}", listener.local_addr()?);

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;

    Ok(())
}
