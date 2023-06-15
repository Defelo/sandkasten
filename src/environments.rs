use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Context;
use sandkasten_client::schemas::programs;
use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, warn};

use crate::VERSION;

pub type Environments = HashMap<String, Environment>;

#[derive(Debug, Deserialize)]
pub struct Environment {
    pub name: String,
    pub version: String,
    pub meta: Value,
    pub default_main_file_name: String,
    pub compile_script: Option<String>,
    pub run_script: String,
    pub closure: PathBuf,
    pub test: Test,
    pub sandkasten_version: String,
}

#[derive(Debug, Deserialize)]
pub struct Test {
    pub main_file: programs::MainFile,
    pub files: Vec<programs::File>,
}

/// Load [`Environments`] from a list of paths.
pub fn load(paths: &[PathBuf]) -> Result<Environments, anyhow::Error> {
    let mut out = HashMap::new();
    for path in paths {
        load_directory(&mut out, path)
            .with_context(|| format!("Failed to load directory {}", path.display()))?;
    }
    Ok(out)
}

fn load_directory(out: &mut Environments, path: &Path) -> Result<(), anyhow::Error> {
    debug!("Loading environments in {}", path.display());
    for file in std::fs::read_dir(path)
        .with_context(|| format!("Failed to read directory {}", path.display()))?
    {
        let file = file.with_context(|| format!("Failed to read file in {}", path.display()))?;
        load_file(out, &file.path())
            .with_context(|| format!("Failed to load file {}", file.path().display()))?;
    }
    Ok(())
}

fn load_file(out: &mut Environments, path: &Path) -> Result<(), anyhow::Error> {
    let name = path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Failed to get file name of path {}", path.display()))?;
    let name = name
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Failed to convert file name {:?} to utf-8 string", name))?
        .trim_end_matches(".json")
        .into();

    if out.contains_key(&name) {
        warn!("Skipping environment {name} as it has already been defined previously.");
        return Ok(());
    }

    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let environment: Environment = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {}", path.display()))?;

    if environment.sandkasten_version != VERSION {
        warn!(
            "Package {name} was built for a different version of Sandkasten ({})",
            environment.sandkasten_version
        );
    }

    debug!("Loaded environment {name} from {}", path.display());
    out.insert(name, environment);

    Ok(())
}
