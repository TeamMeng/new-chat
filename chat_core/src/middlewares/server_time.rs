use crate::middlewares::REQUEST_ID_HEADER;
use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use tokio::time::Instant;
use tracing::warn;

const REQUEST_SERVER_TIME_HEADER: &str = "x-server-time";

pub async fn server_time(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let mut res = next.run(req).await;

    let elapsed = format!("{}us", start.elapsed().as_micros());

    let v = match HeaderValue::from_str(&elapsed) {
        Ok(v) => {
            res.headers_mut()
                .insert(REQUEST_SERVER_TIME_HEADER, v.clone());
            Some(v)
        }
        Err(e) => {
            warn!(
                "Parse elapsed time failed: {} for request {:?}",
                e,
                res.headers().get(REQUEST_ID_HEADER)
            );
            None
        }
    };

    let Some(v) = v else {
        return res;
    };
    res.headers_mut().insert(REQUEST_SERVER_TIME_HEADER, v);
    res
}
