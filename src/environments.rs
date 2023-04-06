use std::{collections::HashMap, env};

use config::{ConfigError, File};
use serde::Deserialize;

pub fn load() -> Result<Environments, ConfigError> {
    config::Config::builder()
        .add_source(File::with_name(
            &env::var("ENVIRONMENTS_CONFIG_PATH").unwrap_or("environments.json".to_owned()),
        ))
        .build()?
        .try_deserialize()
}

#[derive(Debug, Deserialize)]
pub struct Environments {
    pub environments: HashMap<String, Environment>,
    pub nsjail_path: String,
}

#[derive(Debug, Deserialize)]
pub struct Environment {
    pub name: String,
    pub version: String,
    pub compile_script: Option<String>,
    pub run_script: String,
}
