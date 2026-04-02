use backend::{
    crypto::password,
    db,
    model::CreateUser,
    repo::{GroupRepo, UserRepo},
};
use sqlx::Row;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <username> <email> <password>", args[0]);
        std::process::exit(1);
    }

    let username = &args[1];
    let email = &args[2];
    let plain_password = &args[3];

    if plain_password.len() < 8 || plain_password.len() > 128 {
        anyhow::bail!("password length must be between 8 and 128");
    }

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = db::connect(&database_url).await?;

    // Ensure user exists
    let user = if let Some(user) = UserRepo::find_by_username(&pool, username).await? {
        let new_hash = password::hash_password(plain_password)?;
        sqlx::query(
            r#"
            UPDATE users
            SET
                email = $2,
                password_hash = $3,
                display_name = COALESCE(display_name, 'aka'),
                enabled = TRUE,
                failed_login_attempts = 0,
                locked_until = NULL,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            "#,
        )
        .bind(user.id)
        .bind(email)
        .bind(new_hash)
        .execute(&pool)
        .await?;

        UserRepo::find_by_id(&pool, user.id).await?.expect("user exists after update")
    } else {
        let password_hash = password::hash_password(plain_password)?;
        let created = UserRepo::create(
            &pool,
            CreateUser {
                username: username.to_string(),
                email: email.to_string(),
                password_hash,
                display_name: Some("aka".to_string()),
                given_name: None,
                family_name: None,
            },
        )
        .await?;
        created
    };

    // Ensure admin group exists
    let admin_group_id = if let Some(group) = GroupRepo::find_by_name(&pool, "admin").await? {
        group.id
    } else {
        let group = GroupRepo::create(
            &pool,
            backend::model::CreateGroup {
                name: "admin".to_string(),
                description: Some("Administrators".to_string()),
            },
        )
        .await?;
        group.id
    };

    // Bind user to admin group
    GroupRepo::add_user_to_group(&pool, user.id, admin_group_id).await?;

    println!("Admin user ensured: {} ({})", user.username, user.id);

    // Print group memberships for visibility
    let rows = sqlx::query(
        r#"
        SELECT g.name
        FROM groups g
        JOIN user_groups ug ON ug.group_id = g.id
        WHERE ug.user_id = $1
        ORDER BY g.name
        "#,
    )
    .bind(user.id)
    .fetch_all(&pool)
    .await?;

    let groups: Vec<String> = rows
        .into_iter()
        .map(|r| r.try_get::<String, _>("name"))
        .collect::<Result<_, _>>()?;

    println!("Groups: {}", groups.join(", "));
    Ok(())
}
