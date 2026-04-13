use backend::{config::Config, db, service::KeyService, AppState};
use std::sync::Arc;

pub async fn get_test_db() -> Arc<AppState> {
    if std::env::var("DATABASE_URL").is_err() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR must be set");
        let env_path = std::path::Path::new(&manifest_dir).join(".env");
        dotenvy::from_path(&env_path).ok();
    }
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for tests");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .idle_timeout(std::time::Duration::from_secs(30))
        .connect(&database_url)
        .await
        .unwrap();
    db::run_migrations(&pool).await.unwrap();

    backend::metrics::init();

    let config = Config::default();

    // Tests can run against a reused dev database. If the active key was encrypted
    // with a different PRIVATE_KEY_ENCRYPTION_KEY in the past, key decryption fails
    // and many tests become flaky. Reset JWKs and recreate an active key in that case.
    if KeyService::get_active_key(&pool, &config.private_key_encryption_key)
        .await
        .is_err()
    {
        sqlx::query("DELETE FROM jwks")
            .execute(&pool)
            .await
            .unwrap();

        KeyService::get_active_key(&pool, &config.private_key_encryption_key)
            .await
            .unwrap();
    }

    Arc::new(AppState { config, db: pool, email_service: None })
}
