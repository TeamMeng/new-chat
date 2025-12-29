use sqlx::PgPool;

use crate::{
    AppError, AppState,
    models::{ChatUser, Workspace},
};

impl AppState {
    pub async fn create_workspace(&self, name: &str, user_id: u64) -> Result<Workspace, AppError> {
        let ws = sqlx::query_as(
            "
            INSERT INTO workspaces (name, owner_id)
            VALUES ($1, $2)
            RETURNING id, name, owner_id, created_at
            ",
        )
        .bind(name)
        .bind(user_id as i64)
        .fetch_one(&self.pool)
        .await?;
        Ok(ws)
    }

    pub async fn find_workspace_by_name(&self, name: &str) -> Result<Option<Workspace>, AppError> {
        let ws = sqlx::query_as(
            "
            SELECT id, name, owner_id, created_at
            FROM workspaces
            WHERE name = $1
            ",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        Ok(ws)
    }

    #[allow(dead_code)]
    pub async fn find_workspace_by_id(&self, id: u64) -> Result<Option<Workspace>, AppError> {
        let ws = sqlx::query_as(
            "
            SELECT id, name, owner_id, created_at
            FROM workspaces
            WHERE id = $1
            ",
        )
        .bind(id as i64)
        .fetch_optional(&self.pool)
        .await?;
        Ok(ws)
    }

    #[allow(dead_code)]
    pub async fn fetch_chat_users(&self, id: u64) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            "
            SELECT id, fullname, email
            FROM users
            WHERE ws_id = $1 order by id
            ",
        )
        .bind(id as i64)
        .fetch_all(&self.pool)
        .await?;
        Ok(users)
    }
}

impl Workspace {
    pub async fn update_owner(&self, owner_id: u64, pool: &PgPool) -> Result<Workspace, AppError> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CreateUser;
    use anyhow::Result;

    #[tokio::test]
    async fn workspace_create_should_work_and_set_owner() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let ws = state.create_workspace("test", 0).await?;
        let input = CreateUser::new("TeamMeng", &ws.name, "TeamMeng@123.com", "123456");
        let user = state.create_user(&input).await?;

        assert_eq!(ws.name, "test");
        assert_eq!(user.ws_id, ws.id);

        let ws = ws.update_owner(user.id as _, &state.pool).await?;

        assert_eq!(ws.owner_id, user.id);

        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_find_by_name() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let ws = state.find_workspace_by_name("acme").await?;

        assert!(ws.is_some());
        assert_eq!(ws.unwrap().name, "acme");

        Ok(())
    }

    #[tokio::test]
    async fn workspace_find_by_email_should_fail() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let ws = state.find_workspace_by_name("test").await?;

        assert!(ws.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_all_chat_users() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let users = state.fetch_all_chat_users(1).await?;

        assert_eq!(users.len(), 5);

        Ok(())
    }
}
