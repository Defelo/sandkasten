#![forbid(unsafe_code)]
#![warn(clippy::dbg_macro, clippy::use_debug, clippy::todo)]

use std::{sync::Arc, time::Duration};

use poem::{listener::TcpListener, middleware::Tracing, EndpointExt, Route, Server};
use poem_ext::panic_handler::PanicHandler;
use poem_openapi::OpenApiService;
use tokio::fs;
use tracing::{error, info};

use sandkasten::{
    api::get_api,
    config::{self, Config},
    environments,
    program::prune_programs,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    info!("Loading config");
    let config = config::load()?;
    if !fs::try_exists(&config.programs_dir).await? {
        fs::create_dir_all(&config.programs_dir).await?;
    }
    if fs::try_exists(&config.jobs_dir).await? {
        fs::remove_dir_all(&config.jobs_dir).await?;
    }
    fs::create_dir_all(&config.jobs_dir).await?;

    let config = Arc::new(Config {
        programs_dir: config.programs_dir.canonicalize().unwrap(),
        jobs_dir: config.jobs_dir.canonicalize().unwrap(),
        ..config
    });

    info!("Loading environments");
    let environments = Arc::new(environments::load()?);

    let program_lock = Default::default();
    let job_lock = Default::default();

    tokio::spawn({
        let config = Arc::clone(&config);
        let program_lock = Arc::clone(&program_lock);
        async move {
            let mut interval =
                tokio::time::interval(Duration::from_secs(config.prune_programs_interval));
            loop {
                interval.tick().await;
                if let Err(err) = prune_programs(&config, Arc::clone(&program_lock)).await {
                    error!("pruning old programs failed: {err}");
                }
            }
        }
    });

    let api_service = OpenApiService::new(
        get_api(
            Arc::clone(&config),
            Arc::clone(&environments),
            program_lock,
            job_lock,
        ),
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
