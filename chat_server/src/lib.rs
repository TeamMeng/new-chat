mod config;
mod error;
mod handlers;
mod middlewares;
mod models;
mod utils;

use crate::{
    handlers::*,
    middlewares::{set_layers, verify_token},
    utils::{DecodingKey, EncodingKey},
};
use anyhow::Context;
use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, patch, post},
};
use sqlx::PgPool;
use std::{fmt, ops::Deref, sync::Arc};

pub use config::AppConfig;
pub use error::AppError;

#[derive(Clone, Debug)]
pub(crate) struct AppState {
    inner: Arc<AppStateInner>,
}

#[allow(unused)]
pub(crate) struct AppStateInner {
    pub(crate) config: AppConfig,
    pub(crate) ek: EncodingKey,
    pub(crate) dk: DecodingKey,
    pub(crate) pool: PgPool,
}

pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
    let state = AppState::try_new(config).await?;

    let api = Router::new()
        .route(
            "/chat",
            get(list_chat_handler)
                .post(create_chat_handler)
                .patch(update_chat_handler)
                .delete(delete_chat_handler),
        )
        .route(
            "/chat/{id}",
            patch(update_chat_handler)
                .get(list_chat_handler)
                .post(send_message_handler),
        )
        .route("/chat/{id}/messages", get(list_messages_handler))
        .layer(from_fn_with_state(state.clone(), verify_token))
        // routes doesn't need token verification
        .route("/signup", post(signup_handler))
        .route("/signin", post(signin_handler));

    let app = Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state);

    Ok(set_layers(app))
}

// when use state.config == state.inner.config
impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
        let pool = PgPool::connect(&config.server.db_url).await?;
        let ek = EncodingKey::load(&config.auth.sk).context("load pk failed")?;
        let dk = DecodingKey::load(&config.auth.pk).context("load sk failed")?;
        Ok(Self {
            inner: Arc::new(AppStateInner {
                config,
                ek,
                dk,
                pool,
            }),
        })
    }
}

impl fmt::Debug for AppStateInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppStateInner")
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
impl AppState {
    pub async fn new_for_test() -> Result<(sqlx_db_tester::TestPg, Self), AppError> {
        use sqlx_db_tester::TestPg;
        use std::path::Path;

        let config = AppConfig::load()?;
        let dk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
        let ek = EncodingKey::load(&config.auth.sk).context("load sk failed")?;
        let post = config.server.db_url.rfind('/').expect("invalid db_url");
        let server_url = &config.server.db_url[..post];
        let tdb = TestPg::new(server_url.to_string(), Path::new("../migrations"));
        let pool = tdb.get_pool().await;
        let state = Self {
            inner: Arc::new(AppStateInner {
                config,
                ek,
                dk,
                pool,
            }),
        };
        Ok((tdb, state))
    }
}
