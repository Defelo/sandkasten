use std::{collections::HashMap, path::PathBuf};

use sandkasten_client::schemas::programs;
use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, error, warn};

pub type Environments = HashMap<String, Environment>;

#[derive(Debug, Deserialize)]
pub struct Environment {
    pub name: String,
    pub version: String,
    pub meta: Value,
    pub default_main_file_name: String,
    pub compile_script: Option<String>,
    pub run_script: String,
    pub closure: String,
    pub test: Test,
    pub sandkasten_version: String,
}

#[derive(Debug, Deserialize)]
pub struct Test {
    pub main_file: programs::MainFile,
    pub files: Vec<programs::File>,
}

pub fn load(paths: &[PathBuf]) -> Result<Environments, anyhow::Error> {
    let version = env!("CARGO_PKG_VERSION");
    let mut out = HashMap::new();
    for path in paths {
        debug!("Loading environments in {}", path.display());
        for file in match std::fs::read_dir(path) {
            Ok(x) => x,
            Err(err) => {
                error!("Could not open {} directory: {err}", path.display());
                continue;
            }
        } {
            let file = match file {
                Ok(x) => x,
                Err(err) => {
                    error!("Could not read file in {}: {err}", path.display());
                    continue;
                }
            };
            let path = file.path();
            let name = path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .trim_end_matches(".json")
                .into();
            if out.contains_key(&name) {
                warn!("Skipping environment {name} as it has already been defined previously.");
                continue;
            }
            let content = match std::fs::read_to_string(&path) {
                Ok(x) => x,
                Err(err) => {
                    error!("Could not read file {}: {err}", path.display());
                    continue;
                }
            };
            let environment: Environment = match serde_json::from_str(&content) {
                Ok(x) => x,
                Err(err) => {
                    error!(
                        "Could not parse content of {} as environment: {err}",
                        path.display()
                    );
                    continue;
                }
            };
            if environment.sandkasten_version != version {
                warn!(
                    "Package {name} was built for a different version of Sandkasten ({})",
                    environment.sandkasten_version
                );
            }
            debug!("Loaded environment {name} from {}", path.display());
            out.insert(name, environment);
        }
    }
    Ok(out)
}
