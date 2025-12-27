mod user;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
pub use user::{CreateUser, SigninUser};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: i64,
    pub fullname: String,
    pub email: String,
    #[sqlx(default)]
    #[serde(skip)]
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
impl User {
    pub fn new(fullname: &str, email: &str) -> User {
        Self {
            id: i64::default(),
            fullname: fullname.to_string(),
            email: email.to_string(),
            password_hash: None,
            created_at: DateTime::default(),
        }
    }
}
