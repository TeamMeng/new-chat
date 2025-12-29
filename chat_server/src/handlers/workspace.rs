use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::{AppError, AppState, models::User};

pub(crate) async fn list_chat_users_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let users = state.fetch_all_chat_users(user.ws_id as _).await?;
    Ok((StatusCode::OK, Json(users)).into_response())
}
