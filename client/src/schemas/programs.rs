//! Schemas for programs endpoints.

#[cfg(feature = "poem-openapi")]
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The request data for building and running a program.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct BuildRunRequest {
    /// The data for the build step.
    pub build: BuildRequest,
    /// The data for the run step.
    #[cfg_attr(feature = "poem-openapi", oai(default))]
    pub run: RunRequest,
}

/// The request data for building a program.
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
    /// A list of environment variables to set during the build step.
    #[cfg_attr(feature = "poem-openapi", oai(default, validator(max_items = 16)))]
    pub env_vars: Vec<EnvVar>,
    /// Limits to set on the compilation process.
    #[cfg_attr(feature = "poem-openapi", oai(default))]
    pub compile_limits: LimitsOpt,
}

/// The request data for running a program.
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
    /// A list of environment variables to set during the run step.
    #[cfg_attr(feature = "poem-openapi", oai(default, validator(max_items = 16)))]
    pub env_vars: Vec<EnvVar>,
    /// Limits to set on the process.
    #[cfg_attr(feature = "poem-openapi", oai(default))]
    pub run_limits: LimitsOpt,
}

/// A file that is put in the working directory of the build/run process.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct File {
    /// The name of the file.
    #[cfg_attr(
        feature = "poem-openapi",
        oai(validator(pattern = r"^[a-zA-Z0-9._-]{1,32}$"))
    )]
    pub name: String,
    /// The content of the file.
    #[cfg_attr(feature = "poem-openapi", oai(validator(max_length = 65536)))]
    pub content: String,
}

/// An environment variable that is set for the build/run process.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct EnvVar {
    /// The name of the environment variable.
    #[cfg_attr(
        feature = "poem-openapi",
        oai(validator(pattern = r"^[a-zA-Z0-9_]{1,64}$"))
    )]
    pub name: String,
    /// The value of the environment variable.
    #[cfg_attr(feature = "poem-openapi", oai(validator(pattern = "^[^\0]{0,256}$")))]
    pub value: String,
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

/// The error responses that may be returned when building and running a program.
#[derive(Debug, Deserialize)]
#[serde(tag = "error", content = "details", rename_all = "snake_case")]
pub enum BuildRunError {
    /// Environment does not exist.
    EnvironmentNotFound,
    /// Code could not be compiled.
    CompileError(RunResult),
    /// File names are not unique.
    InvalidFileNames,
    /// Environment variable names are not valid.
    InvalidEnvVars,
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

/// The error responses that may be returned when building a program.
#[derive(Debug, Deserialize)]
#[serde(tag = "error", content = "details", rename_all = "snake_case")]
pub enum BuildError {
    /// Environment does not exist.
    EnvironmentNotFound,
    /// Code could not be compiled.
    CompileError(RunResult),
    /// File names are not unique.
    InvalidFileNames,
    /// Environment variable names are not valid.
    InvalidEnvVars,
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

/// The error responses that may be returned when running a program.
#[derive(Debug, Deserialize)]
#[serde(tag = "error", content = "details", rename_all = "snake_case")]
pub enum RunError {
    /// File names are not unique.
    InvalidFileNames,
    /// Environment variable names are not valid.
    InvalidEnvVars,
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

/// Information about a build/run limit that has been exceeded.
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct LimitExceeded {
    /// The name of the limit.
    pub name: String,
    /// The maximum value that is allowed for this limit.
    pub max_value: u64,
}

macro_rules! limits {
    ($( $(#[doc = $doc:literal])* $(#[validator($validator:meta)])* $name:ident : $type:ty , )*) => {
        /// The resource limits of a process.
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
        #[cfg_attr(feature = "poem-openapi", derive(Object))]
        pub struct Limits {
            $(
                $(#[doc = $doc])*
                pub $name : $type,
            )*
        }

        /// The resource limits of a process. Omit a value to use the default limit.
        #[derive(Debug, Default, Serialize)]
        #[cfg_attr(feature = "poem-openapi", derive(Object))]
        pub struct LimitsOpt {
            $(
                $(#[doc = $doc])*
                $(
                    #[cfg_attr(feature = "poem-openapi", oai(validator($validator)))]
                )*
                pub $name : Option<$type>,
            )*
        }

        impl LimitsOpt {
            /// Try to convert to [`Limits`] by replacing empty limits with default values and
            /// returning an error if a limit has been exceeded.
            pub fn check(
                &self,
                max_limits: &Limits,
            ) -> Result<Limits, Vec<LimitExceeded>> {
                let mut errors = Vec::new();
                let out = Limits {
                    $(
                        $name : {
                            let val = self.$name.unwrap_or(max_limits.$name);
                            if val > max_limits.$name {
                                errors.push(LimitExceeded {
                                    name: stringify!($name).into(),
                                    max_value: max_limits.$name as _,
                                });
                            }
                            val
                        },
                    )*
                };
                if errors.is_empty() {
                    Ok(out)
                } else {
                    Err(errors)
                }
            }
        }
    };
}

limits! {
    /// The maximum number of cpus the process is allowed to use.
    #[validator(minimum(value = "1"))]
    cpus: u64,
    /// The number of **seconds** the process is allowed to run.
    #[validator(minimum(value = "1"))]
    time: u64,
    /// The amount of memory the process is allowed to use (in **MB**).
    #[validator(minimum(value = "1"))]
    memory: u64,
    /// The size of the tmpfs mounted at /tmp (in **MB**).
    tmpfs: u64,
    /// The maximum size of a file the process is allowed to create (in **MB**).
    #[validator(minimum(value = "1"))]
    filesize: u64,
    /// The maximum number of file descripters the process can open at the same time.
    #[validator(minimum(value = "1"))]
    file_descriptors: u64,
    /// The maximum number of processes that can run concurrently in the sandbox.
    #[validator(minimum(value = "1"))]
    processes: u64,
    /// The maximum number of bytes that are read from stdout.
    stdout_max_size: u64,
    /// The maximum number of bytes that are read from stderr.
    stderr_max_size: u64,
    /// Whether the process is allowed to access the network.
    network: bool,
}
