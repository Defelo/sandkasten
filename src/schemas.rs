use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Object)]
pub struct RunRequest {
    pub create: CreateProgramRequest,
    pub run: RunProgramRequest,
}

#[derive(Debug, Clone, Object)]
pub struct CreateProgramRequest {
    pub environment: String,
    #[oai(validator(min_items = 1))]
    pub files: Vec<File>,
    #[oai(default)]
    pub compile_limits: Limits,
}

#[derive(Debug, Clone, Object)]
pub struct RunProgramRequest {
    pub stdin: Option<String>,
    #[oai(default)]
    pub args: Vec<String>,
    #[oai(default)]
    pub files: Vec<File>,
    #[oai(default)]
    pub run_limits: Limits,
}

#[derive(Debug, Clone, Object, Serialize)]
pub struct File {
    pub name: String,
    pub content: String,
}

#[derive(Debug, Clone, Object, Default)]
pub struct Limits {
    pub timeout: Option<u64>,
    pub memory_limit: Option<u64>,
}

#[derive(Debug, Clone, Object)]
pub struct CreateResult {
    pub program_id: Uuid,
    pub compile_result: Option<RunResult>,
}

#[derive(Debug, Clone, Object, Serialize, Deserialize)]
pub struct RunResult {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, Object)]
pub struct Environment {
    pub name: String,
    pub version: String,
}
