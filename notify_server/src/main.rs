use anyhow::Result;
use notify_server::{AppConfig, get_router, setup_pg_listener};
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let config = AppConfig::load()?;

    let addr = format!("0.0.0.0:{}", &config.server.port);
    info!("Listening on: {}", addr);
    let listener = TcpListener::bind(addr).await?;

    let (app, state) = get_router(config);

    setup_pg_listener(state).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
