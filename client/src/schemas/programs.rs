#[cfg(feature = "poem-openapi")]
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct BuildRunRequest {
    pub build: BuildRequest,
    #[cfg_attr(feature = "poem-openapi", oai(default))]
    pub run: RunRequest,
}

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct BuildRequest {
    /// The environment to use for building and running the program.
    pub environment: String,
    /// A list of source files. The first file is usually used as entrypoint to the program.
    #[cfg_attr(
        feature = "poem-openapi",
        oai(validator(min_items = 1, max_items = 10))
    )]
    pub files: Vec<File>,
    /// Limits to set on the compilation process.
    #[cfg_attr(feature = "poem-openapi", oai(default))]
    pub compile_limits: LimitsOpt,
}

#[derive(Debug, Serialize, Default)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct RunRequest {
    /// The stdin input the process reads.
    #[cfg_attr(feature = "poem-openapi", oai(default, validator(max_length = 65536)))]
    pub stdin: Option<String>,
    /// A list of command line arguments that are passed to the process.
    #[cfg_attr(
        feature = "poem-openapi",
        oai(default, validator(max_items = 100, pattern = "^[^\0]{0,4096}$"))
    )]
    pub args: Vec<String>,
    /// A list of additional files that are put in the working directory of the process.
    #[cfg_attr(feature = "poem-openapi", oai(default, validator(max_items = 10)))]
    pub files: Vec<File>,
    /// Limits to set on the process.
    #[cfg_attr(feature = "poem-openapi", oai(default))]
    pub run_limits: LimitsOpt,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct File {
    #[cfg_attr(
        feature = "poem-openapi",
        oai(validator(pattern = r"^[a-zA-Z0-9._-]{1,32}$"))
    )]
    pub name: String,
    #[cfg_attr(feature = "poem-openapi", oai(validator(max_length = 65536)))]
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct Limits {
    /// The maximum number of cpus the process is allowed to use.
    pub cpus: u64,
    /// The number of **seconds** the process is allowed to run.
    pub time: u64,
    /// The amount of memory the process is allowed to use (in **MB**).
    pub memory: u64,
    /// The size of the tmpfs mounted at /tmp (in **MB**).
    pub tmpfs: u64,
    /// The maximum size of a file the process is allowed to create (in **MB**).
    pub filesize: u64,
    /// The maximum number of file descripters the process can open at the same time.
    pub file_descriptors: u64,
    /// The maximum number of processes that can run concurrently in the sandbox.
    pub processes: u64,
    /// The maximum number of bytes that are read from stdout.
    pub stdout_max_size: u64,
    /// The maximum number of bytes that are read from stderr.
    pub stderr_max_size: u64,
    /// Whether the process is allowed to access the network.
    pub network: bool,
}

#[derive(Debug, Default, Serialize)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct LimitsOpt {
    /// The maximum number of cpus the process is allowed to use.
    #[cfg_attr(feature = "poem-openapi", oai(validator(minimum(value = "1"))))]
    pub cpus: Option<u64>,
    /// The number of **seconds** the process is allowed to run.
    #[cfg_attr(feature = "poem-openapi", oai(validator(minimum(value = "1"))))]
    pub time: Option<u64>,
    /// The amount of memory the process is allowed to use (in **MB**).
    #[cfg_attr(feature = "poem-openapi", oai(validator(minimum(value = "1"))))]
    pub memory: Option<u64>,
    /// The size of the tmpfs mounted at /tmp (in **MB**).
    pub tmpfs: Option<u64>,
    /// The maximum size of a file the process is allowed to create (in **MB**).
    #[cfg_attr(feature = "poem-openapi", oai(validator(minimum(value = "1"))))]
    pub filesize: Option<u64>,
    /// The maximum number of file descripters the process can open at the same time.
    #[cfg_attr(feature = "poem-openapi", oai(validator(minimum(value = "1"))))]
    pub file_descriptors: Option<u64>,
    /// The maximum number of processes that can run concurrently in the sandbox.
    #[cfg_attr(feature = "poem-openapi", oai(validator(minimum(value = "1"))))]
    pub processes: Option<u64>,
    /// The maximum number of bytes that are read from stdout.
    pub stdout_max_size: Option<u64>,
    /// The maximum number of bytes that are read from stderr.
    pub stderr_max_size: Option<u64>,
    /// Whether the process is allowed to access the network.
    pub network: Option<bool>,
}

/// The results of building and running a program.
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct BuildRunResult {
    /// A unique identifier of the program that was built.
    pub program_id: Uuid,
    /// The number of seconds after the last execution of the program before it is removed.
    pub ttl: u64,
    /// The results of compiling the program. Empty iff programs don't need to be compiled in this
    /// environment.
    pub build: Option<RunResult>,
    /// The results of running the program.
    pub run: RunResult,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "error", content = "details", rename_all = "snake_case")]
pub enum BuildRunError {
    /// Environment does not exist.
    EnvironmentNotFound,
    /// Code could not be compiled.
    CompileError(RunResult),
    /// File names are not unique.
    InvalidFileNames,
    /// The specified compile limits are too high.
    CompileLimitsExceeded(Vec<LimitExceeded>),
    /// The specified run limits are too high.
    RunLimitsExceeded(Vec<LimitExceeded>),
}

/// The results of building a program.
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct BuildResult {
    /// A unique identifier of the program that was built.
    pub program_id: Uuid,
    /// The number of seconds after the last execution of the program before it is removed.
    pub ttl: u64,
    /// The results of compiling the program. Empty iff programs don't need to be compiled in this
    /// environment.
    pub compile_result: Option<RunResult>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "error", content = "details", rename_all = "snake_case")]
pub enum BuildError {
    /// Environment does not exist.
    EnvironmentNotFound,
    /// Code could not be compiled.
    CompileError(RunResult),
    /// File names are not unique.
    InvalidFileNames,
    /// The specified compile limits are too high.
    CompileLimitsExceeded(Vec<LimitExceeded>),
}

/// The results of running (or compiling) a program.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
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

#[derive(Debug, Deserialize)]
#[serde(tag = "error", content = "details", rename_all = "snake_case")]
pub enum RunError {
    /// File names are not unique.
    InvalidFileNames,
    /// Program does not exist.
    ProgramNotFound,
    /// The specified run limits are too high.
    RunLimitsExceeded(Vec<LimitExceeded>),
}

/// The amount of resources a process used.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct ResourceUsage {
    /// The number of **milliseconds** the process ran.
    pub time: u64,
    /// The amount of memory the process used (in **KB**)
    pub memory: u64,
}

#[derive(Debug, Deserialize)]
pub struct LimitExceeded {
    pub name: String,
    pub max_value: u64,
}
