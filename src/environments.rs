use std::{collections::HashMap, env, path::PathBuf};

use config::{ConfigError, File};
use sandkasten_client::schemas::programs;
use serde::Deserialize;

pub fn load() -> Result<Environments, ConfigError> {
    config::Config::builder()
        .add_source(File::with_name(
            &env::var("ENVIRONMENTS_CONFIG_PATH").unwrap_or("environments.json".to_owned()),
        ))
        .build()?
        .try_deserialize()
        .map(|x: Environments| Environments {
            nsjail_path: x.nsjail_path.canonicalize().unwrap(),
            time_path: x.time_path.canonicalize().unwrap(),
            ..x
        })
}

#[derive(Debug, Deserialize)]
pub struct Environments {
    pub environments: HashMap<String, Environment>,
    pub nsjail_path: PathBuf,
    pub time_path: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct Environment {
    pub name: String,
    pub version: String,
    pub default_main_file_name: String,
    pub compile_script: Option<String>,
    pub run_script: String,
    pub closure: String,
    pub test: Test,
}

#[derive(Debug, Deserialize)]
pub struct Test {
    pub main_file: programs::MainFile,
    pub files: Vec<programs::File>,
}
