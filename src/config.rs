use std::{env, path::PathBuf};

use anyhow::Context;
use config::{Environment, File};
use sandkasten_client::schemas::programs::Limits;
use serde::{Deserialize, Deserializer};
use tracing::info;
use url::Url;

pub fn load() -> Result<Config, anyhow::Error> {
    let path = env::var("CONFIG_PATH").unwrap_or("config.toml".to_owned());
    info!("Loading config from {path}");
    let conf: Config = config::Config::builder()
        .add_source(File::with_name(&path))
        .add_source(Environment::default().separator("__"))
        .build()
        .context("Failed to load config")?
        .try_deserialize()
        .context("Failed to parse config")?;

    Ok(Config {
        nsjail_path: conf.nsjail_path.canonicalize().with_context(|| {
            format!(
                "Failed to resolve `nsjail_path` {}",
                conf.nsjail_path.display()
            )
        })?,
        time_path: conf.time_path.canonicalize().with_context(|| {
            format!("Failed to resolve `time_path` {}", conf.time_path.display())
        })?,
        ..conf
    })
}

#[derive(Debug, Deserialize)]
pub struct Config {
    /// The host to listen on.
    pub host: String,
    /// The port to listen on.
    pub port: u16,
    /// The path prefix added by a reverse proxy. Set to `"/"` if you don't have
    /// a reverse proxy.
    pub server: String,

    /// The url of the redis server (see https://docs.rs/redis/latest/redis/#connection-parameters).
    pub redis_url: Url,
    /// The default time to live for cache entries in seconds.
    pub cache_ttl: u64,

    /// The directory where programs are stored.
    pub programs_dir: PathBuf,
    /// The directory where files for jobs are stored.
    pub jobs_dir: PathBuf,

    /// The time to live for programs in seconds.
    pub program_ttl: u64,
    /// The number of seconds to wait between deleting old programs.
    pub prune_programs_interval: u64,

    /// The maximum number of jobs that can be run at the same time.
    pub max_concurrent_jobs: usize,

    /// The maximum allowed limits for compile steps.
    pub compile_limits: Limits,
    /// The maximum allowed limits for run steps.
    pub run_limits: Limits,

    /// The number of times the program is run when measuring the base resource
    /// usage of an environment.
    pub base_resource_usage_runs: usize,

    /// Whether to use cgroup to set resource limits where possible. It is
    /// strongly recommended to set this to true in production environments!
    pub use_cgroup: bool,
    /// The path to the nsjail binary. This binary must have the setuid bit set
    /// and it must be owned by root OR sandkasten itself must be run as root.
    pub nsjail_path: PathBuf,
    /// The path to the time binary.
    pub time_path: PathBuf,

    /// A list of paths to load environments from. If specified as an
    /// environment variable, separate the paths using a `:`
    /// (e.g. `"/foo/path1:/bar/path2:/baz/path3"`).
    #[serde(deserialize_with = "path")]
    pub environments_path: Vec<PathBuf>,
}

fn path<'de, D>(deserializer: D) -> Result<Vec<PathBuf>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Path {
        String(String),
        List(Vec<PathBuf>),
    }

    Ok(match Path::deserialize(deserializer)? {
        Path::String(x) => x.trim().split(':').map(Into::into).collect(),
        Path::List(x) => x,
    })
}
