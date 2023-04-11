use std::{env, path::PathBuf};

use config::{ConfigError, File};
use serde::Deserialize;

use crate::sandbox::Limits;

pub fn load() -> Result<Config, ConfigError> {
    config::Config::builder()
        .add_source(File::with_name(
            &env::var("CONFIG_PATH").unwrap_or("config.toml".to_owned()),
        ))
        .build()?
        .try_deserialize()
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub server: String,

    pub programs_dir: PathBuf,
    pub jobs_dir: PathBuf,

    pub program_ttl: u64,             // in seconds
    pub prune_programs_interval: u64, // in seconds

    pub compile_limits: Limits,
    pub run_limits: Limits,
}
