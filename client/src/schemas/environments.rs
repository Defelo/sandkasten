//! Schemas for environments endpoints.

use std::collections::HashMap;

#[cfg(feature = "poem-openapi")]
use poem_openapi::{types::Example, NewType, Object};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::programs::ResourceUsage;

/// A package that can build and run programs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct Environment {
    /// The display name of the environment (e.g. `Rust` or `C++`).
    pub name: String,
    /// The version of the environment.
    pub version: String,
    /// The default name of the main file that is used if no filename is
    /// specified.
    pub default_main_file_name: String,
    /// Additional metadata specific to the environment.
    pub meta: Value,
}

/// A map of environments where the key represents the id of the environment.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "poem-openapi", derive(NewType))]
#[cfg_attr(
    feature = "poem-openapi",
    oai(
        from_parameter = false,
        from_multipart = false,
        to_header = false,
        example = true
    )
)]
pub struct ListEnvironmentsResponse(pub HashMap<String, Environment>);

/// The base resource usage of an environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct BaseResourceUsage {
    /// The base resource usage of the build step.
    pub build: Option<ResourceUsage>,
    /// The minimum base resource usage of the run step.
    pub run_min: ResourceUsage,
    /// The average base resource usage of the run step.
    pub run_avg: ResourceUsage,
    /// The maximum base resource usage of the run step.
    pub run_max: ResourceUsage,
}

/// The error responses that may be returned when calculating the base resource
/// usage.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "error", content = "details", rename_all = "snake_case")]
pub enum GetBaseResourceUsageError {
    /// Environment does not exist.
    EnvironmentNotFound,
}

#[cfg(feature = "poem-openapi")]
impl Example for ListEnvironmentsResponse {
    fn example() -> Self {
        Self(HashMap::from([
            (
                "rust".into(),
                Environment {
                    name: "Rust".into(),
                    version: "1.64.0".into(),
                    default_main_file_name: "code.rs".into(),
                    meta: json!({
                        "homepage": "https://www.rust-lang.org/"
                    }),
                },
            ),
            (
                "python".into(),
                Environment {
                    name: "Python".into(),
                    version: "3.11.1".into(),
                    default_main_file_name: "code.py".into(),
                    meta: json!({
                        "packages": ["numpy", "pandas"]
                    }),
                },
            ),
        ]))
    }
}
