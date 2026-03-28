// Group Repository
//
// 用户组数据访问层

use sqlx::PgPool;
use uuid::Uuid;

use crate::model::{CreateGroup, Group, UpdateGroup};

pub struct GroupRepo;

impl GroupRepo {
    /// 根据ID查找组
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Group>, sqlx::Error> {
        sqlx::query_as::<_, Group>(
            r#"
            SELECT * FROM groups WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// 根据名称查找组
    pub async fn find_by_name(pool: &PgPool, name: &str) -> Result<Option<Group>, sqlx::Error> {
        sqlx::query_as::<_, Group>(
            r#"
            SELECT * FROM groups WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(pool)
        .await
    }

    /// 获取所有组
    pub async fn find_all(pool: &PgPool) -> Result<Vec<Group>, sqlx::Error> {
        sqlx::query_as::<_, Group>(
            r#"
            SELECT * FROM groups ORDER BY created_at DESC
            "#,
        )
        .fetch_all(pool)
        .await
    }

    /// 创建组
    pub async fn create(pool: &PgPool, input: CreateGroup) -> Result<Group, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = time::OffsetDateTime::now_utc();

        sqlx::query_as::<_, Group>(
            r#"
            INSERT INTO groups (id, name, description, created_at)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(now)
        .fetch_one(pool)
        .await
    }

    /// 更新组
    pub async fn update(pool: &PgPool, id: Uuid, input: UpdateGroup) -> Result<Group, sqlx::Error> {
        // 动态构建 UPDATE 语句
        let group = Self::find_by_id(pool, id).await?.ok_or(sqlx::Error::RowNotFound)?;

        let name = input.name.unwrap_or(group.name);
        let description = input.description.or(group.description);

        sqlx::query_as::<_, Group>(
            r#"
            UPDATE groups
            SET name = $2, description = $3
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .fetch_one(pool)
        .await
    }

    /// 删除组
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM groups WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 添加用户到组
    pub async fn add_user_to_group(pool: &PgPool, user_id: Uuid, group_id: Uuid) -> Result<(), sqlx::Error> {
        let now = time::OffsetDateTime::now_utc();

        sqlx::query(
            r#"
            INSERT INTO user_groups (user_id, group_id, created_at)
            VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(user_id)
        .bind(group_id)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 从组中移除用户
    pub async fn remove_user_from_group(pool: &PgPool, user_id: Uuid, group_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM user_groups
            WHERE user_id = $1 AND group_id = $2
            "#,
        )
        .bind(user_id)
        .bind(group_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 获取用户所属的所有组
    pub async fn find_user_groups(pool: &PgPool, user_id: Uuid) -> Result<Vec<Group>, sqlx::Error> {
        sqlx::query_as::<_, Group>(
            r#"
            SELECT g.*
            FROM groups g
            INNER JOIN user_groups ug ON g.id = ug.group_id
            WHERE ug.user_id = $1
            ORDER BY g.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    /// 获取组中的所有用户ID
    pub async fn find_group_user_ids(pool: &PgPool, group_id: Uuid) -> Result<Vec<Uuid>, sqlx::Error> {
        let results: Vec<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT user_id FROM user_groups WHERE group_id = $1
            "#,
        )
        .bind(group_id)
        .fetch_all(pool)
        .await?;

        Ok(results.into_iter().map(|r| r.0).collect())
    }
}
