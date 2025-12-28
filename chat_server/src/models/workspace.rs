use crate::{
    AppError,
    models::{ChatUser, Workspace},
};
use sqlx::PgPool;

impl Workspace {
    pub async fn create(name: &str, user_id: u64, pool: &PgPool) -> Result<Self, AppError> {
        let ws = sqlx::query_as(
            "
            INSERT INTO workspaces (name, owner_id)
            VALUES ($1, $2)
            RETURNING id, name, owner_id, created_at
            ",
        )
        .bind(name)
        .bind(user_id as i64)
        .fetch_one(pool)
        .await?;
        Ok(ws)
    }

    pub async fn find_by_name(name: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let ws = sqlx::query_as(
            "
            SELECT id, name, owner_id, created_at
            FROM workspaces
            WHERE name = $1
            ",
        )
        .bind(name)
        .fetch_optional(pool)
        .await?;
        Ok(ws)
    }

    #[allow(unused)]
    pub async fn find_by_id(id: u64, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let ws = sqlx::query_as(
            "
            SELECT id, name, owner_id, created_at
            FROM workspaces
            WHERE id = $1
            ",
        )
        .bind(id as i64)
        .fetch_optional(pool)
        .await?;
        Ok(ws)
    }

    pub async fn update_owner(&self, owner_id: u64, pool: &PgPool) -> Result<Self, AppError> {
        let ws = sqlx::query_as(
            "
            UPDATE workspaces
            SET owner_id = $1
            WHERE id = $2 AND EXISTS (
                SELECT 1 FROM users WHERE id = $1 AND ws_id = $2
            )
            RETURNING id, name, owner_id, created_at
            ",
        )
        .bind(owner_id as i64)
        .bind(self.id)
        .fetch_one(pool)
        .await?;

        Ok(ws)
    }

    #[allow(unused)]
    pub async fn fetch_all_chat_users(
        ws_id: u64,
        pool: &PgPool,
    ) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            "
            SELECT id, fullname, email
            FROM users
            WHERE ws_id = $1 ORDER BY users.id
            ",
        )
        .bind(ws_id as i64)
        .fetch_all(pool)
        .await?;
        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::{CreateUser, User},
        test_util::get_test_pool,
    };
    use anyhow::Result;

    #[tokio::test]
    async fn workspace_create_should_work_and_set_owner() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;
        let ws = Workspace::create("test", 0, &pool).await?;

        let input = CreateUser::new("TeamMeng", &ws.name, "TeamMeng@123.com", "123456");
        let user = User::create(&input, &pool).await?;
        assert_eq!(ws.name, "test");
        assert_eq!(user.ws_id, ws.id);

        let ws = ws.update_owner(user.id as _, &pool).await?;
        assert_eq!(ws.owner_id, user.id);

        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_find_by_name() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;

        let ws = Workspace::find_by_name("acme", &pool).await?;
        assert!(ws.is_some());
        assert_eq!(ws.unwrap().name, "acme");

        Ok(())
    }

    #[tokio::test]
    async fn workspace_find_by_email_should_fail() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;

        let ws = Workspace::find_by_name("test", &pool).await?;
        assert!(ws.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_all_chat_users() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;

        let users = Workspace::fetch_all_chat_users(1, &pool).await?;
        assert_eq!(users.len(), 5);

        Ok(())
    }
}
