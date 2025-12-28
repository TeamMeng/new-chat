use crate::{
    AppError, AppState,
    models::{Chat, CreateChat, User},
};
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

pub(crate) async fn list_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chats = Chat::fetch_all(user.ws_id as _, &state.pool).await?;
    Ok((StatusCode::OK, Json(chats)).into_response())
}

pub(crate) async fn get_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::get_by_id(id, &state.pool).await?;
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

pub(crate) async fn create_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(input): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::create(&input, user.ws_id as _, &state.pool).await?;
    Ok((StatusCode::CREATED, Json(chat)).into_response())
}
