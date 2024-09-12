#![forbid(unsafe_code)]
#![warn(clippy::dbg_macro, clippy::use_debug, clippy::todo)]

use std::{path::Path, sync::Arc, time::Duration};

use anyhow::{ensure, Context};
use key_rwlock::KeyRwLock;
use poem::{listener::TcpListener, middleware::Tracing, EndpointExt, Route, Server};
use poem_ext::panic_handler::PanicHandler;
use poem_openapi::OpenApiService;
use sandkasten::{
    api::get_api,
    config::{self, Config},
    environments,
    metrics::{self, Metrics},
    program::prune::prune_programs,
    VERSION,
};
use tokio::fs;
use tracing::{error, info, trace};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting Sandkasten v{VERSION}");

    info!("Loading config");
    let config = config::load().context("Failed to load config")?;
    trace!("Loaded config: {config:#?}");
    ensure!(config.base_resource_usage_runs >= 1);
    ensure!(config.base_resource_usage_permits >= 1);
    ensure!(config.base_resource_usage_permits <= config.max_concurrent_jobs as _);

    info!("Creating directories for jobs and programs");
    create_dir_if_not_exists(&config.programs_dir).await?;
    create_dir_if_not_exists(&config.jobs_dir).await?;

    info!("Pruning jobs directory");
    for dir in std::fs::read_dir(&config.jobs_dir).context("Failed to read jobs directory")? {
        let path = dir.context("Failed to read jobs directory entry")?.path();
        if path.is_dir() {
            std::fs::remove_dir_all(&path)
                .with_context(|| format!("Failed to remove directory {}", path.display()))?;
        } else {
            std::fs::remove_file(&path)
                .with_context(|| format!("Failed to remove file {}", path.display()))?;
        }
    }

    let config = Arc::new(Config {
        programs_dir: config.programs_dir.canonicalize().unwrap(),
        jobs_dir: config.jobs_dir.canonicalize().unwrap(),
        ..config
    });

    info!("Loading environments");
    let environments = Arc::new(
        environments::load(&config.environments_path).context("Failed to load environments")?,
    );
    info!("Loaded {} environments", environments.len());

    let program_lock = Arc::new(KeyRwLock::new());
    let job_lock = Arc::new(KeyRwLock::new());

    let metrics = Arc::new(Metrics::new().context("Failed to initialize Prometheus metrics")?);

    tokio::spawn(prune_old_programs_loop(
        Arc::clone(&config),
        Arc::clone(&program_lock),
    ));

    let api_service = OpenApiService::new(
        get_api(
            Arc::clone(&config),
            Arc::clone(&environments),
            program_lock,
            job_lock,
        ),
        "Sandkasten",
        VERSION,
    )
    .summary("Run untrusted code in an isolated environment")
    .description(include_str!("api_description.md"))
    .external_document("/openapi.json")
    .server(&config.server);
    let mut route = Route::new()
        .nest("/openapi.json", api_service.spec_endpoint())
        .nest("/docs", api_service.swagger_ui())
        .nest("/redoc", api_service.redoc());
    if config.enable_metrics {
        route = route.nest("/metrics", poem::get(metrics::endpoint));
    }
    let app = route
        .nest("/", api_service)
        .data(metrics)
        .with(Tracing)
        .with(PanicHandler::middleware());

    info!("Listening on {}:{}", config.host, config.port);
    Server::new(TcpListener::bind((config.host.as_str(), config.port)))
        .run(app)
        .await
        .context("Failed to start server")?;

    Ok(())
}

/// Periodically delete all programs that are not in use anymore.
async fn prune_old_programs_loop(config: Arc<Config>, program_lock: Arc<KeyRwLock<Uuid>>) {
    let mut interval = tokio::time::interval(Duration::from_secs(config.prune_programs_interval));
    loop {
        interval.tick().await;
        if let Err(err) = prune_programs(&config, Arc::clone(&program_lock)).await {
            error!("Pruning old programs failed: {err:#}");
        }
    }
}

/// Create a directory if it does not exist yet. Return an error if the file
/// exists, but is not a directory.
async fn create_dir_if_not_exists(path: &Path) -> Result<(), anyhow::Error> {
    if fs::try_exists(path)
        .await
        .with_context(|| format!("Failed to check existence of {}", path.display()))?
    {
        let metadata = fs::metadata(path)
            .await
            .with_context(|| format!("Failed to read metadata of {}", path.display()))?;
        anyhow::ensure!(metadata.is_dir(), "{} is not a directory", path.display());
    } else {
        fs::create_dir_all(path)
            .await
            .with_context(|| format!("Failed to create directory {}", path.display()))?;
    }

    Ok(())
}
