//! Schemas for configuration endpoints.

#[cfg(feature = "poem-openapi")]
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use super::programs::Limits;

/// The public configuration of Sandkasten.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct PublicConfig {
    /// The time to live for programs in seconds.
    pub program_ttl: u64,

    /// The maximum number of jobs that can run at the same time.
    pub max_concurrent_jobs: usize,

    /// The maximum allowed limits for compile steps.
    pub compile_limits: Limits,
    /// The maximum allowed limits for run steps.
    pub run_limits: Limits,

    /// The number of times the program is run when measuring the base resource
    /// usage of an environment.
    pub base_resource_usage_runs: usize,
}
