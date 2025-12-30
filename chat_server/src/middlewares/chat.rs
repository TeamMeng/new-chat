use crate::{AppError, AppState, models::User};
use axum::{
    extract::{FromRequestParts, Path, Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};

pub async fn verify_chat(State(state): State<AppState>, req: Request, next: Next) -> Response {
    // verify if user_id is a member of chat_id
    let (mut parts, body) = req.into_parts();
    let Path(chat_id) = Path::<u64>::from_request_parts(&mut parts, &state)
        .await
        .unwrap();

    let user = parts.extensions.get::<User>().unwrap();

    if !state
        .is_chat_member(chat_id, user.id as _)
        .await
        .unwrap_or_default()
    {
        let err = AppError::CreateMessageError(format!(
            "user {} are not a member of chat {}",
            user.id, chat_id
        ));
        return err.into_response();
    }

    let req = Request::from_parts(parts, body);
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middlewares::verify_token;
    use anyhow::Result;
    use axum::{
        Router, body::Body, http::StatusCode, middleware::from_fn_with_state, routing::get,
    };
    use tower::ServiceExt;

    async fn handler(_req: Request) -> impl IntoResponse {
        (StatusCode::OK, "ok")
    }

    #[tokio::test]
    async fn verify_chat_middleware_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let user = state
            .find_user_by_id(1)
            .await?
            .expect("user id: 1 should exists");
        let token = state.ek.sign(user)?;

        let app = Router::new()
            .route("/chat/{id}/messages", get(handler))
            .layer(from_fn_with_state(state.clone(), verify_chat))
            .layer(from_fn_with_state(state.clone(), verify_token))
            .with_state(state);

        // user in chat
        let req = Request::builder()
            .uri("/chat/1/messages")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())?;
        let res = app.clone().oneshot(req).await?;
        assert_eq!(StatusCode::OK, res.status());

        // user not in chat
        let req = Request::builder()
            .uri("/chat/5/messages")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())?;
        let res = app.oneshot(req).await?;
        assert_eq!(StatusCode::BAD_REQUEST, res.status());

        Ok(())
    }
}
