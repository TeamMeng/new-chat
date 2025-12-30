use crate::{AppError, AppState};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chat_core::{ChatUser, User};
use serde::{Deserialize, Serialize};
use std::mem;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateUser {
    pub fullname: String,
    pub email: String,
    pub workspace: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SigninUser {
    pub email: String,
    pub password: String,
}

impl AppState {
    /// Find a user by email
    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let ret = sqlx::query_as(
            "
            SELECT id, ws_id, fullname, email, created_at
            FROM users
            WHERE email = $1
            ",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;
        Ok(ret)
    }

    /// Find a user by id
    #[allow(dead_code)]
    pub async fn find_user_by_id(&self, id: i64) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as(
            "
            SELECT id, ws_id, fullname, email, created_at
            FROM users
            WHERE id = $1
            ",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// create a new user
    pub async fn create_user(&self, input: &CreateUser) -> Result<User, AppError> {
        // check if email exists
        if self.find_user_by_email(&input.email).await?.is_some() {
            return Err(AppError::EmailAleardyExists(input.email.clone()));
        }
        // check if workspace exists, if not create one
        let ws = match self.find_workspace_by_name(&input.workspace).await? {
            Some(ws) => ws,
            None => self.create_workspace(&input.workspace, 0).await?,
        };

        let password_hash = hash_password(&input.password)?;

        let user: User = sqlx::query_as(
            "
            INSERT INTO users (ws_id, fullname, email, password_hash)
            VALUES ($1, $2, $3, $4)
            RETURNING id, ws_id, fullname, email, created_at
            ",
        )
        .bind(ws.id)
        .bind(&input.fullname)
        .bind(&input.email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;

        if ws.owner_id == 0 {
            self.update_workspace_owner(user.id as _, ws.id as _)
                .await?;
        }

        Ok(user)
    }

    /// Verify email and password
    pub async fn verify_user(&self, input: &SigninUser) -> Result<Option<User>, AppError> {
        let user: Option<User> = sqlx::query_as(
            "
            SELECT id, ws_id, fullname, email, password_hash, created_at
            FROM users
            WHERE email = $1
            ",
        )
        .bind(&input.email)
        .fetch_optional(&self.pool)
        .await?;

        match user {
            Some(mut user) => {
                let password_hash = mem::take(&mut user.password_hash);
                let is_valid =
                    verify_password(&input.password, &password_hash.unwrap_or_default())?;
                if is_valid { Ok(Some(user)) } else { Ok(None) }
            }
            None => Ok(None),
        }
    }

    pub async fn fetch_chat_user_by_ids(&self, ids: &[i64]) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            "
            SELECT id, fullname, email
            FROM users
            WHERE id = ANY($1)
            ",
        )
        .bind(ids)
        .fetch_all(&self.pool)
        .await?;
        Ok(users)
    }

    pub async fn fetch_all_chat_users(&self, ws_id: u64) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            "
            SELECT id, fullname, email
            FROM users
            WHERE ws_id = $1
            ",
        )
        .bind(ws_id as i64)
        .fetch_all(&self.pool)
        .await?;
        Ok(users)
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}

fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let argon2 = Argon2::default();
    let password_hash = PasswordHash::new(password_hash)?;

    let is_valid = argon2
        .verify_password(password.as_bytes(), &password_hash)
        .is_ok();

    Ok(is_valid)
}

#[cfg(test)]
impl CreateUser {
    pub fn new(fullname: &str, workspace: &str, email: &str, password: &str) -> Self {
        Self {
            fullname: fullname.to_string(),
            workspace: workspace.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        }
    }
}

#[cfg(test)]
impl SigninUser {
    pub fn new(email: &str, password: &str) -> Self {
        Self {
            email: email.into(),
            password: password.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn create_user_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let fullname = "TeamMeng";
        let workspace = "none";
        let email = "TeamMeng@123.com";
        let password = "123456";

        // create user success
        let input = CreateUser::new(fullname, workspace, email, password);
        let user = state.create_user(&input).await?;

        assert_eq!(fullname, &user.fullname);
        assert_eq!(email, &user.email);
        assert!(user.id > 0);

        // failed to create user
        let ret = state.create_user(&input).await;

        assert!(ret.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn find_user_by_email_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let email = "Test@123.com";
        let user = state.find_user_by_email(email).await?;

        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.fullname, "TeamTest");
        assert_eq!(user.email, email);

        let input = SigninUser::new(email, "123456");
        let user = state.verify_user(&input).await?;

        assert!(user.is_some());

        // failed to find user by email
        let ret = state.find_user_by_email("TeamMeng@123.com").await?;
        assert!(ret.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn create_duplicate_user_should_fail() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let fullname = "TeamTest";
        let workspace = "acme";
        let email = "Test@123.com";
        let password = "123456";
        let input = CreateUser::new(fullname, workspace, email, password);
        let ret = state.create_user(&input).await;

        if let Err(AppError::EmailAleardyExists(email)) = ret {
            assert_eq!(email, input.email);
        }

        Ok(())
    }

    #[tokio::test]
    async fn find_user_by_id_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;

        // find id success
        let user = state
            .find_user_by_id(1)
            .await?
            .expect("user 1 should exists");

        assert_eq!(user.id, 1);
        assert_eq!(user.fullname, "TeamTest");
        assert_eq!(user.email, "Test@123.com");

        // find id fail
        let user = state.find_user_by_id(6).await?;

        assert!(user.is_none());

        Ok(())
    }

    #[test]
    fn hash_and_verify_password_should_work() -> Result<()> {
        let password = "123456";

        let password_hash = hash_password(password)?;

        let is_valid = verify_password(password, &password_hash)?;
        assert!(is_valid);

        Ok(())
    }
}
