use crate::{AppError, AppState, error::ErrorOutput, models::CreateChat};
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chat_core::{Chat, User};

/// List all chats in the workspace of the user
#[utoipa::path(
    get,
    path = "/api/chats",
    responses(
        (status = 200, description = "List of chats", body = Vec<Chat>)
    ),
    security(
        ("token"=[])
    )
)]
pub(crate) async fn list_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chats = state.fetch_all_chats(user.id as _, user.ws_id as _).await?;
    Ok((StatusCode::OK, Json(chats)).into_response())
}

/// Get the chat info by id
#[utoipa::path(
    get,
    path = "/api/chats/{id}",
    params(
        ("id" = u64, Path, description = "Chat id")
    ),
    responses(
        (status = 200, description = "Get found", body = Chat),
        (status = 400, description = "Get not found", body = ErrorOutput)
    ),
    security(
        ("token"=[])
    )
)]
pub(crate) async fn get_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.get_chat_by_id(id).await?;
    match chat {
        Some(chat) => Ok((StatusCode::OK, Json(chat)).into_response()),
        None => Ok((
            StatusCode::NOT_FOUND,
            AppError::NotFound(format!("chat id: {} not found", id)),
        )
            .into_response()),
    }
}

pub(crate) async fn update_chat_handler() -> impl IntoResponse {
    "update chat"
}

pub(crate) async fn delete_chat_handler() -> impl IntoResponse {
    "delete chat"
}

/// Create a new chat in the workspace of the user
#[utoipa::path(
    post,
    path = "/api/chats/",
    responses(
        (status = 201, description = "Chat created", body = Chat)
    ),
    security(
        ("token"=[])
    )
)]
pub(crate) async fn create_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(input): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state
        .create_chat(&input, user.id as _, user.ws_id as _)
        .await?;
    Ok((StatusCode::CREATED, Json(chat)).into_response())
}
