mod sse;

use crate::sse::sse_handler;
use anyhow::Result;
use axum::{
    Router,
    response::{Html, IntoResponse},
    routing::get,
};
use chat_core::{Chat, Message};
use futures_util::StreamExt;
use sqlx::postgres::PgListener;
use tracing::info;

const INDEX_HTML: &str = include_str!("../index.html");

pub enum Event {
    NewChat(Chat),
    AddToChat(Chat),
    RemoveFromChat(Chat),
    NewMessage(Message),
}

pub fn get_router() -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/events", get(sse_handler))
}

pub async fn setup_pg_listener() -> Result<()> {
    let mut listener = PgListener::connect("postgres://postgres:postgres@localhost/chat").await?;
    listener.listen("chat_updated").await?;
    listener.listen("chat_message_created").await?;

    let mut stream = listener.into_stream();

    tokio::spawn(async move {
        while let Some(notif) = stream.next().await {
            info!("Received notification: {:?}", notif);
        }
    });

    Ok(())
}

async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}
