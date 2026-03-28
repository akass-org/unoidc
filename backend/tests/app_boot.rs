#[tokio::test]
async fn app_builds_router() {
    use backend::{build_app_with_state, AppState, config::Config};
    use sqlx::postgres::PgPoolOptions;

    // 使用测试配置构建应用（不需要真实数据库连接）
    let config = Config::default();
    // 使用一个空的 PgPool 进行路由结构测试
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy(&config.database_url)
        .expect("Failed to create pool");

    let state = AppState::new(config, pool);
    let _app = build_app_with_state(state);
}
