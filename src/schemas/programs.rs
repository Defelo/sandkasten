use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::sandbox::Limits;

#[derive(Debug, Object, Serialize)]
pub struct BuildRunRequest {
    pub build: BuildRequest,
    pub run: RunRequest,
}

#[derive(Debug, Object, Serialize)]
pub struct BuildRequest {
    /// The environment to use for building and running the program.
    pub environment: String,
    /// A list of source files. The first file is usually used as entrypoint to the program.
    #[oai(validator(min_items = 1, max_items = 10))]
    pub files: Vec<File>,
    /// Limits to set on the compilation process.
    #[oai(default)]
    pub compile_limits: LimitsOpt,
}

#[derive(Debug, Object, Serialize, Default)]
pub struct RunRequest {
    /// The stdin input the process reads.
    #[oai(validator(max_length = 65536))]
    pub stdin: Option<String>,
    /// A list of command line arguments that are passed to the process.
    #[oai(default, validator(max_items = 100, pattern = "^[^\0]{0,4096}$"))]
    pub args: Vec<String>,
    /// A list of additional files that are put in the working directory of the process.
    #[oai(default, validator(max_items = 10))]
    pub files: Vec<File>,
    /// Limits to set on the process.
    #[oai(default)]
    pub run_limits: LimitsOpt,
}

#[derive(Debug, Object, Serialize, Deserialize)]
pub struct File {
    #[oai(validator(pattern = r"^[a-zA-Z0-9._-]{1,32}$"))]
    pub name: String,
    #[oai(validator(max_length = 65536))]
    pub content: String,
}

#[derive(Debug, Object, Default, Serialize)]
pub struct LimitsOpt {
    /// The maximum number of cpus the process is allowed to use.
    pub cpus: Option<u64>,
    /// The number of **seconds** the process is allowed to run.
    pub time: Option<u64>,
    /// The amount of memory the process is allowed to use (in **MB**).
    pub memory: Option<u64>,
    /// The size of the tmpfs mounted at /tmp (in **MB**).
    pub tmpfs: Option<u64>,
    /// The maximum size of a file the process is allowed to create (in **MB**).
    pub filesize: Option<u64>,
    /// The maximum number of file descripters the process can open at the same time.
    pub file_descriptors: Option<u64>,
    /// The maximum number of processes that can run concurrently in the sandbox.
    pub processes: Option<u64>,
    /// The maximum number of bytes that are read from stdout.
    pub stdout_max_size: Option<u64>,
    /// The maximum number of bytes that are read from stderr.
    pub stderr_max_size: Option<u64>,
}

/// The results of building and running a program.
#[derive(Debug, Object, Deserialize)]
pub struct BuildRunResult {
    /// A unique identifier of the program that was built.
    pub program_id: Uuid,
    /// The results of compiling the program. Empty iff programs don't need to be compiled in this
    /// environment.
    pub build: Option<RunResult>,
    /// The results of running the program.
    pub run: RunResult,
}

/// The results of building a program.
#[derive(Debug, Object)]
pub struct BuildResult {
    /// A unique identifier of the program that was built.
    pub program_id: Uuid,
    /// The results of compiling the program. Empty iff programs don't need to be compiled in this
    /// environment.
    pub compile_result: Option<RunResult>,
}

/// The results of running (or compiling) a program.
#[derive(Debug, Object, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunResult {
    /// The exit code of the processes.
    pub status: i32,
    /// The stdout output the process produced.
    pub stdout: String,
    /// The stderr output the process produced.
    pub stderr: String,
    /// The amount of resources the process used.
    pub resource_usage: ResourceUsage,
    /// The limits that applied to the process.
    pub limits: Limits,
}

/// The amount of resources a process used.
#[derive(Debug, Object, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceUsage {
    /// The number of **milliseconds** the process ran.
    pub time: u64,
    /// The amount of memory the process used (in **KB**)
    pub memory: u64,
}
