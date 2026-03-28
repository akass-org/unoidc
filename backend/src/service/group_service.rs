// Group Service
//
// 用户组业务逻辑层

use sqlx::PgPool;
use uuid::Uuid;

use crate::model::{CreateGroup, Group, UpdateGroup};
use crate::repo::{GroupRepo, UserRepo};

pub struct GroupService;

impl GroupService {
    /// 创建新组
    pub async fn create_group(pool: &PgPool, input: CreateGroup) -> Result<Group, anyhow::Error> {
        // 验证组名
        if input.name.is_empty() || input.name.len() > 64 {
            return Err(anyhow::anyhow!("Group name must be 1-64 characters"));
        }

        // 检查组名是否已存在
        if GroupRepo::find_by_name(pool, &input.name).await?.is_some() {
            return Err(anyhow::anyhow!("Group name already exists"));
        }

        let group = GroupRepo::create(pool, input)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create group: {}", e))?;

        Ok(group)
    }

    /// 根据 ID 获取组
    pub async fn get_group(pool: &PgPool, id: Uuid) -> Result<Group, anyhow::Error> {
        GroupRepo::find_by_id(pool, id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Group not found"))
    }

    /// 根据名称获取组
    pub async fn get_group_by_name(pool: &PgPool, name: &str) -> Result<Group, anyhow::Error> {
        GroupRepo::find_by_name(pool, name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Group not found"))
    }

    /// 获取所有组
    pub async fn list_groups(pool: &PgPool) -> Result<Vec<Group>, anyhow::Error> {
        GroupRepo::find_all(pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list groups: {}", e))
    }

    /// 更新组
    pub async fn update_group(
        pool: &PgPool,
        id: Uuid,
        input: UpdateGroup,
    ) -> Result<Group, anyhow::Error> {
        GroupRepo::update(pool, id, input)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to update group: {}", e))
    }

    /// 删除组
    pub async fn delete_group(pool: &PgPool, id: Uuid) -> Result<(), anyhow::Error> {
        GroupRepo::delete(pool, id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete group: {}", e))
    }

    /// 添加用户到组
    pub async fn add_user_to_group(
        pool: &PgPool,
        user_id: Uuid,
        group_id: Uuid,
    ) -> Result<(), anyhow::Error> {
        // 验证用户存在
        UserRepo::find_by_id(pool, user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        // 验证组存在
        GroupRepo::find_by_id(pool, group_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Group not found"))?;

        GroupRepo::add_user_to_group(pool, user_id, group_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to add user to group: {}", e))
    }

    /// 从组中移除用户
    pub async fn remove_user_from_group(
        pool: &PgPool,
        user_id: Uuid,
        group_id: Uuid,
    ) -> Result<(), anyhow::Error> {
        GroupRepo::remove_user_from_group(pool, user_id, group_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to remove user from group: {}", e))
    }

    /// 获取用户所属的组
    pub async fn get_user_groups(pool: &PgPool, user_id: Uuid) -> Result<Vec<Group>, anyhow::Error> {
        GroupRepo::find_user_groups(pool, user_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get user groups: {}", e))
    }

    /// 获取组中的所有用户 ID
    pub async fn get_group_user_ids(pool: &PgPool, group_id: Uuid) -> Result<Vec<Uuid>, anyhow::Error> {
        GroupRepo::find_group_user_ids(pool, group_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get group users: {}", e))
    }
}
