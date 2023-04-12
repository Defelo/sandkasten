#![forbid(unsafe_code)]
#![warn(clippy::dbg_macro, clippy::use_debug, clippy::todo)]

use std::{sync::Arc, time::Duration};

use poem::{listener::TcpListener, middleware::Tracing, EndpointExt, Route, Server};
use poem_ext::panic_handler::PanicHandler;
use poem_openapi::OpenApiService;
use tokio::fs;
use tracing::{error, info};

use sandkasten::{api::get_api, config, environments, program::prune_programs};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    info!("Loading config");
    let config = Arc::new(config::load()?);

    info!("Loading environments");
    let environments = Arc::new(environments::load()?);

    if !fs::try_exists(&config.programs_dir).await? {
        fs::create_dir_all(&config.programs_dir).await?;
    }

    tokio::spawn({
        let config = Arc::clone(&config);
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
        get_api(Arc::clone(&config), Arc::clone(&environments)),
        "Sandkasten",
        env!("CARGO_PKG_VERSION"),
    )
    .external_document("/openapi.json")
    .server(&config.server);
    let app = Route::new()
        .nest("/openapi.json", api_service.spec_endpoint())
        .nest("/docs", api_service.swagger_ui())
        .nest("/redoc", api_service.redoc())
        .nest("/", api_service)
        .with(Tracing)
        .with(PanicHandler::middleware());

    info!("Listening on {}:{}", config.host, config.port);
    Server::new(TcpListener::bind((config.host.as_str(), config.port)))
        .run(app)
        .await?;

    Ok(())
}
