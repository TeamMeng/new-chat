mod config;
mod error;
mod notif;
mod sse;

use crate::{config::AppConfig, error::AppError, notif::AppEvent, sse::sse_handler};
use anyhow::Result;
use axum::{
    Router,
    middleware::from_fn_with_state,
    response::{Html, IntoResponse},
    routing::get,
};
use chat_core::{
    DecodingKey, User,
    middlewares::{TokenVerify, verify_token},
};
use dashmap::DashMap;
use std::{ops::Deref, sync::Arc};
use tokio::sync::broadcast;

pub use notif::setup_pg_listener;

const INDEX_HTML: &str = include_str!("../index.html");

pub type UserMap = Arc<DashMap<u64, broadcast::Sender<Arc<AppEvent>>>>;

#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

pub struct AppStateInner {
    pub config: AppConfig,
    pub users: UserMap,
    dk: DecodingKey,
}

pub fn get_router() -> Result<(AppState, Router), AppError> {
    let config = AppConfig::load()?;
    let state = AppState::new(config)?;

    let app = Router::new()
        .route("/", get(index_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        .route("/events", get(sse_handler))
        .with_state(state.clone());

    Ok((state, app))
}

async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TokenVerify for AppState {
    type Error = AppError;

    fn verify(&self, token: &str) -> Result<User, Self::Error> {
        Ok(self.dk.verify(token)?)
    }
}

impl AppState {
    pub fn new(config: AppConfig) -> Result<Self, AppError> {
        let dk = DecodingKey::load(&config.auth.pk)?;
        let users = Arc::new(DashMap::new());
        Ok(Self(Arc::new(AppStateInner { config, users, dk })))
    }
}
