use crate::{
    AppError,
    models::{Chat, ChatType},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
}

#[allow(dead_code)]
impl Chat {
    pub async fn create(input: &CreateChat, ws_id: u64, pool: &PgPool) -> Result<Self, AppError> {
        let chat = sqlx::query_as(
            "
            INSERT INTO chats (ws_id ,name, type, members)
            VALUES ($1, $2, $3)
            RETURNING id, name, members, created_at
            ",
        )
        .bind(ws_id as i64)
        .bind(&input.name)
        .bind(ChatType::Group)
        .bind(&input.members)
        .fetch_one(pool)
        .await?;

        Ok(chat)
    }
}
