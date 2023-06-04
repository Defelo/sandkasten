use std::{env, path::PathBuf};

use config::{Environment, File};
use sandkasten_client::schemas::programs::Limits;
use serde::{Deserialize, Deserializer};
use url::Url;

pub fn load() -> Result<Config, anyhow::Error> {
    let conf: Config = config::Config::builder()
        .add_source(File::with_name(
            &env::var("CONFIG_PATH").unwrap_or("config.toml".to_owned()),
        ))
        .add_source(Environment::default().separator("__"))
        .build()?
        .try_deserialize()?;

    Ok(Config {
        nsjail_path: conf.nsjail_path.canonicalize()?,
        time_path: conf.time_path.canonicalize()?,
        ..conf
    })
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub server: String,

    pub redis_url: Url,
    pub cache_ttl: u64, // in seconds

    pub programs_dir: PathBuf,
    pub jobs_dir: PathBuf,

    pub program_ttl: u64,             // in seconds
    pub prune_programs_interval: u64, // in seconds

    pub max_concurrent_jobs: usize,

    pub compile_limits: Limits,
    pub run_limits: Limits,

    pub base_resource_usage_runs: usize,

    pub use_cgroup: bool,
    pub nsjail_path: PathBuf,
    pub time_path: PathBuf,

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
