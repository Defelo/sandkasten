use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::sandbox::Limits;

#[derive(Debug, Object)]
pub struct RunRequest {
    pub build: BuildProgramRequest,
    pub run: RunProgramRequest,
}

#[derive(Debug, Object)]
pub struct BuildProgramRequest {
    /// The environment to use for building and running the program.
    pub environment: String,
    /// A list of source files. The first file is usually used as entrypoint to the program.
    #[oai(validator(min_items = 1))]
    pub files: Vec<File>,
    /// Limits to set on the compilation process.
    #[oai(default)]
    pub compile_limits: LimitsOpt,
}

#[derive(Debug, Object)]
pub struct RunProgramRequest {
    /// The stdin input the process reads.
    pub stdin: Option<String>,
    /// A list of command line arguments that are passed to the process.
    #[oai(default)]
    pub args: Vec<String>,
    /// A list of additional files that are put in the working directory of the process.
    #[oai(default)]
    pub files: Vec<File>,
    /// Limits to set on the process.
    #[oai(default)]
    pub run_limits: LimitsOpt,
}

#[derive(Debug, Object, Serialize)]
pub struct File {
    pub name: String,
    pub content: String,
}

#[derive(Debug, Object, Default)]
pub struct LimitsOpt {
    /// The maximum number of cpus the process is allowed to use.
    pub cpus: Option<u64>,
    /// The number of **seconds** the process is allowed to run.
    pub time: Option<u64>,
    /// The amount of memory the process is allowed to use (in **MB**).
    pub memory: Option<u64>,
    /// The maximum size of a file the process is allowed to create (in **MB**).
    pub filesize: Option<u64>,
    /// The maximum number of file descripters the process can open at the same time.
    pub file_descriptors: Option<u64>,
    /// The maximum number of processes that can run concurrently in the sandbox.
    pub processes: Option<u64>,
}

/// The results of building and running a program.
#[derive(Debug, Object)]
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
#[derive(Debug, Object, Serialize, Deserialize)]
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
#[derive(Debug, Object, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// The number of **milliseconds** the process ran.
    pub time: u64,
    /// The amount of memory the process used (in **KB**)
    pub memory: u64,
}
