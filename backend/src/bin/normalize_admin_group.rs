use backend::{db, repo::GroupRepo};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = db::connect(&database_url).await?;

    let admin = GroupRepo::find_by_name(&pool, "admin").await?;
    let legacy = GroupRepo::find_by_name(&pool, "admins").await?;

    match (admin, legacy) {
        (Some(admin_group), Some(legacy_group)) => {
            // 保留 admin，清理 legacy admins 及其关联，避免权限判定与展示出现双重语义。
            sqlx::query("DELETE FROM user_groups WHERE group_id = $1")
                .bind(legacy_group.id)
                .execute(&pool)
                .await?;

            sqlx::query("DELETE FROM client_groups WHERE group_id = $1")
                .bind(legacy_group.id)
                .execute(&pool)
                .await?;

            GroupRepo::delete(&pool, legacy_group.id).await?;

            println!(
                "Normalized admin groups: kept '{}' ({}) and removed legacy 'admins' ({})",
                admin_group.name, admin_group.id, legacy_group.id
            );
        }
        (None, Some(legacy_group)) => {
            // 老环境只有 admins：直接更名为 admin，保持权限连续性。
            sqlx::query("UPDATE groups SET name = 'admin' WHERE id = $1")
                .bind(legacy_group.id)
                .execute(&pool)
                .await?;

            println!(
                "Renamed legacy group 'admins' to 'admin' ({})",
                legacy_group.id
            );
        }
        (Some(admin_group), None) => {
            println!("Admin group already normalized: {}", admin_group.id);
        }
        (None, None) => {
            anyhow::bail!("Neither 'admin' nor 'admins' group exists");
        }
    }

    Ok(())
}
