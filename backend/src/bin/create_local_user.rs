use std::env;

use backend::{crypto::password, db, model::CreateUser, repo::UserRepo};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let args: Vec<String> = env::args().collect();
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

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = db::connect(&database_url).await?;

    if UserRepo::find_by_username(&pool, username).await?.is_some() {
        anyhow::bail!("user '{}' already exists", username);
    }

    let password_hash = password::hash_password(plain_password)?;

    let user = UserRepo::create(
        &pool,
        CreateUser {
            username: username.to_string(),
            email: email.to_string(),
            password_hash: Some(password_hash),
            display_name: None,
            given_name: None,
            family_name: None,
        },
    )
    .await?;

    println!("Created user '{}' with id {}", user.username, user.id);
    Ok(())
}
