use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Environment {
    pub name: String,
    #[allow(dead_code)]
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct BuildRunRequest {
    pub build: BuildRequest,
    pub run: RunRequest,
}

#[derive(Debug, Deserialize)]
pub struct BuildRunResponse {
    pub program_id: String,
    pub build: Option<RunResponse>,
    pub run: RunResponse,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct RunResponse {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
    pub resource_usage: ResourceUsage,
    pub limits: Limits,
}

#[derive(Debug, Serialize)]
pub struct BuildRequest {
    pub environment: String,
    pub files: Vec<File>,
    pub compile_limits: LimitsOpt,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "error", content = "details", rename_all = "snake_case")]
pub enum BuildError {
    CompileError(RunResponse),
}

#[derive(Debug, Serialize, Default)]
pub struct RunRequest {
    pub stdin: Option<String>,
    pub args: Vec<String>,
    pub files: Vec<File>,
    pub run_limits: LimitsOpt,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub name: String,
    pub content: String,
}

#[derive(Debug, Serialize, Default)]
pub struct LimitsOpt {
    pub cpus: Option<u64>,
    pub file_descriptors: Option<u64>,
    pub filesize: Option<u64>,
    pub memory: Option<u64>,
    pub tmpfs: Option<u64>,
    pub processes: Option<u64>,
    pub time: Option<u64>,
    pub stdout_max_size: Option<u64>,
    pub stderr_max_size: Option<u64>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Limits {
    pub cpus: u64,
    pub file_descriptors: u64,
    pub filesize: u64,
    pub memory: u64,
    pub tmpfs: u64,
    pub processes: u64,
    pub time: u64,
    pub stdout_max_size: u64,
    pub stderr_max_size: u64,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct ResourceUsage {
    pub time: u64,
    pub memory: u64,
}

pub fn url(path: impl std::fmt::Display) -> String {
    format!("http://127.0.0.1:8000{path}")
}
