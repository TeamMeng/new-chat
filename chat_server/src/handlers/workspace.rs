use crate::{AppError, AppState, error::ErrorOutput};
use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};
use chat_core::{ChatUser, User};

#[utoipa::path(
    get,
    path = "/api/users",
    responses(
        (status = 200, description = "List of chats", body = Vec<ChatUser>),
        (status = 400, description = "Invalid input", body = ErrorOutput),
    ),
    security(
        ("token"=[])
    )
)]
pub(crate) async fn list_chat_users_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let users = state.fetch_all_chat_users(user.ws_id as _).await?;
    Ok((StatusCode::OK, Json(users)).into_response())
}
