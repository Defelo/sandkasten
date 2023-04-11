#![forbid(unsafe_code)]
#![warn(clippy::dbg_macro, clippy::use_debug, clippy::todo)]

use std::time::Duration;

use poem::{listener::TcpListener, middleware::Tracing, EndpointExt, Route, Server};
use poem_ext::panic_handler::PanicHandler;
use poem_openapi::OpenApiService;
use tracing::{error, info};

use crate::{api::Api, program::prune_programs};

mod api;
mod config;
mod environments;
mod program;
mod sandbox;
mod schemas;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    info!("Loading config");
    let config = config::load()?;

    info!("Loading environments");
    let environments = environments::load()?;

    tokio::spawn({
        let config = config.clone();
        async move {
            let mut interval =
                tokio::time::interval(Duration::from_secs(config.prune_programs_interval));
            loop {
                interval.tick().await;
                if let Err(err) = prune_programs(&config).await {
                    error!("pruning old programs failed: {err}");
                }
            }
        }
    });

    let api_service = OpenApiService::new(
        Api {
            config: config.clone(),
            environments,
            compile_lock: Default::default(),
        },
        "Sandkasten",
        env!("CARGO_PKG_VERSION"),
    )
    .external_document("/openapi.json")
    .server(config.server);
    let app = Route::new()
        .nest("/openapi.json", api_service.spec_endpoint())
        .nest("/docs", api_service.swagger_ui())
        .nest("/redoc", api_service.redoc())
        .nest("/", api_service)
        .with(Tracing)
        .with(PanicHandler::middleware());

    info!("Listening on {}:{}", config.host, config.port);
    Server::new(TcpListener::bind((config.host, config.port)))
        .run(app)
        .await?;

    Ok(())
}
