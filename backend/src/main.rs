use backend::{build_app, config::Config, db, metrics, middleware::LogRedactionLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志（带脱敏功能）
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(LogRedactionLayer)  // 添加日志脱敏层
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

    // 构建应用
    let app = build_app().await;

    // 启动服务器
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("Server listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}
