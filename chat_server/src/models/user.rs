use super::User;
use crate::{
    AppError,
    models::{ChatUser, Workspace},
};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
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

#[allow(dead_code)]
impl User {
    /// Find a user by email
    pub async fn find_by_email(email: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let ret = sqlx::query_as(
            "
            SELECT id, ws_id, fullname, email, created_at FROM users WHERE email = $1
            ",
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;
        Ok(ret)
    }

    /// create a new user
    pub async fn create(input: &CreateUser, pool: &PgPool) -> Result<Self, AppError> {
        // check if email exists
        if Self::find_by_email(&input.email, pool).await?.is_some() {
            return Err(AppError::EmailAleardyExists(input.email.clone()));
        }
        // check if workspace exists, if not create one
        let ws = match Workspace::find_by_name(&input.workspace, pool).await? {
            Some(ws) => ws,
            None => Workspace::create(&input.workspace, 0, pool).await?,
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
        .fetch_one(pool)
        .await?;

        if ws.owner_id == 0 {
            ws.update_owner(user.id as _, pool).await?;
        }

        Ok(user)
    }

    /// Verify email and password
    pub async fn verify(input: &SigninUser, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let user: Option<User> = sqlx::query_as(
            "
            SELECT id, ws_id, fullname, email, password_hash, created_at FROM users WHERE email = $1
            ",
        )
        .bind(&input.email)
        .fetch_optional(pool)
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
}

impl ChatUser {
    // pub async fn fetch_all(user: &User) {}
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
    use crate::test_util::get_test_pool;
    use anyhow::Result;

    #[tokio::test]
    async fn create_user_should_work() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;

        let fullname = "TeamMeng";
        let workspace = "none";
        let email = "TeamMeng@123.com";
        let password = "123456";

        // create user success
        let input = CreateUser::new(fullname, workspace, email, password);

        let user = User::create(&input, &pool).await?;

        assert_eq!(fullname, &user.fullname);
        assert_eq!(email, &user.email);
        assert!(user.id > 0);

        // failed to create user
        let ret = User::create(&input, &pool).await;
        assert!(ret.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn find_user_by_email_should_work() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;
        let email = "Test@123.com";

        let user = User::find_by_email(email, &pool).await?;

        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.fullname, "TeamTest");
        assert_eq!(user.email, email);

        let input = SigninUser::new(email, "123456");

        let user = User::verify(&input, &pool).await?;
        assert!(user.is_some());

        // failed to find user by email
        let ret = User::find_by_email("TeamMeng@123.com", &pool).await?;
        assert!(ret.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn create_duplicate_user_should_fail() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;

        let fullname = "TeamTest";
        let workspace = "acme";
        let email = "Test@123.com";
        let password = "123456";

        let input = CreateUser::new(fullname, workspace, email, password);

        let ret = User::create(&input, &pool).await;

        if let Err(AppError::EmailAleardyExists(email)) = ret {
            assert_eq!(email, input.email);
        }

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
