use crate::middlewares::REQUEST_ID_HEADER;
use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use tracing::warn;
use uuid::Uuid;

pub async fn set_request_id(req: Request, next: Next) -> Response {
    // if x-request-id exists, do nothing, otherwise generate a new one
    let mut res = next.run(req).await;
    let id = match res.headers().get(REQUEST_ID_HEADER) {
        Some(v) => Some(v.clone()),
        None => {
            let request_id = Uuid::now_v7().to_string();
            match HeaderValue::from_str(&request_id) {
                Ok(v) => {
                    res.headers_mut().insert(REQUEST_ID_HEADER, v.clone());
                    Some(v)
                }
                Err(e) => {
                    warn!("parse generate request id failed: {}", e);
                    None
                }
            }
        }
    };

    let Some(id) = id else {
        return res;
    };
    res.headers_mut().insert(REQUEST_ID_HEADER, id);
    res
}
