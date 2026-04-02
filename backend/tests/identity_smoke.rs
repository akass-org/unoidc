// 身份模块冒烟测试
//
// 测试 user/group/client/consent 的 CRUD 操作

use backend::*;
use sqlx::PgPool;

#[tokio::test]
#[ignore] // 需要数据库连接,默认跳过
async fn test_user_crud() {
    // 连接数据库
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // 运行迁移
    backend::db::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    // 创建用户
    let user = model::CreateUser {
        username: format!("test_user_{}", uuid::Uuid::new_v4()),
        email: format!("test_{}@example.com", uuid::Uuid::new_v4()),
        password_hash: "test_hash".to_string(),
        display_name: Some("Test User".to_string()),
        given_name: None,
        family_name: None,
    };

    let created = repo::UserRepo::create(&pool, user.clone())
        .await
        .expect("Failed to create user");

    assert_eq!(created.username, user.username);
    assert_eq!(created.email, user.email);

    // 查找用户
    let found = repo::UserRepo::find_by_id(&pool, created.id)
        .await
        .expect("Failed to find user")
        .expect("User not found");

    assert_eq!(found.id, created.id);

    // 清理
    repo::UserRepo::delete(&pool, created.id)
        .await
        .expect("Failed to delete user");
}

#[tokio::test]
#[ignore]
async fn test_group_crud() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    backend::db::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    // 创建组
    let group = model::CreateGroup {
        name: format!("test_group_{}", uuid::Uuid::new_v4()),
        description: Some("Test Group".to_string()),
    };

    let created = repo::GroupRepo::create(&pool, group.clone())
        .await
        .expect("Failed to create group");

    assert_eq!(created.name, group.name);

    // 查找组
    let found = repo::GroupRepo::find_by_id(&pool, created.id)
        .await
        .expect("Failed to find group")
        .expect("Group not found");

    assert_eq!(found.id, created.id);

    // 清理
    repo::GroupRepo::delete(&pool, created.id)
        .await
        .expect("Failed to delete group");
}

#[tokio::test]
#[ignore]
async fn test_client_crud() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    backend::db::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    // 创建客户端
    let client = model::CreateClient {
        client_id: format!("test_client_{}", uuid::Uuid::new_v4()),
        client_secret_hash: Some("test_hash".to_string()),
        is_public: false,
        name: "Test Client".to_string(),
        description: Some("Test Description".to_string()),
        app_url: Some("https://example.com".to_string()),
        redirect_uris: vec!["https://example.com/callback".to_string()],
        post_logout_redirect_uris: None,
        grant_types: vec!["authorization_code".to_string()],
        response_types: vec!["code".to_string()],
        token_endpoint_auth_method: "client_secret_basic".to_string(),
    };

    let created = repo::ClientRepo::create(&pool, client.clone())
        .await
        .expect("Failed to create client");

    assert_eq!(created.client_id, client.client_id);
    assert_eq!(created.name, client.name);

    // 查找客户端
    let found = repo::ClientRepo::find_by_id(&pool, created.id)
        .await
        .expect("Failed to find client")
        .expect("Client not found");

    assert_eq!(found.id, created.id);

    // 清理
    repo::ClientRepo::delete(&pool, created.id)
        .await
        .expect("Failed to delete client");
}

#[tokio::test]
#[ignore]
async fn test_user_service() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    backend::db::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    // 注册用户
    let username = format!("testuser_{}", uuid::Uuid::new_v4());
    let email = format!("test_{}@example.com", uuid::Uuid::new_v4());
    let password = "test_password_123".to_string();
        let display_name = "Test User"; // Added display name variable

        let user = service::UserService::register(
            &pool,
            username.clone(),
            email.clone(),
            password.clone(),
            Some(display_name.to_string()),
        )
        .await
        .expect("Failed to register user");

    assert_eq!(user.username, username);
    assert_eq!(user.email, email);

    // 清理
    repo::UserRepo::delete(&pool, user.id)
        .await
        .expect("Failed to delete user");
}
