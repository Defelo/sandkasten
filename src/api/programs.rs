use std::{collections::HashSet, sync::Arc};

use key_rwlock::KeyRwLock;
use once_cell::unsync::Lazy;
use poem_ext::{response, shield_mw::shield};
use poem_openapi::{param::Path, payload::Json, OpenApi};
use regex::Regex;
use sandkasten_client::schemas::programs::{
    BuildRequest, BuildResult, BuildRunRequest, BuildRunResult, EnvVar, File, LimitExceeded,
    MainFile, RunRequest, RunResult,
};
use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::{
    config::Config,
    environments::Environments,
    program::{build_program, run_program, BuildProgramError, RunProgramError},
};

use super::Tags;

pub struct ProgramsApi {
    pub config: Arc<Config>,
    pub environments: Arc<Environments>,
    pub program_lock: Arc<KeyRwLock<Uuid>>,
    pub job_lock: Arc<KeyRwLock<Uuid>>,
    pub request_semaphore: Arc<Semaphore>,
}

#[OpenApi(tag = "Tags::Programs")]
impl ProgramsApi {
    /// Build and immediately run a program.
    #[oai(path = "/run", method = "post", transform = "shield")]
    async fn run(&self, data: Json<BuildRunRequest>) -> BuildRun::Response {
        if !check_mainfile(&data.0.build.main_file)
            || !check_files(&data.0.build.files)
            || !check_files(&data.0.run.files)
        {
            return BuildRun::invalid_file_names();
        }
        if !check_env_vars(&data.0.build.env_vars) || !check_env_vars(&data.0.run.env_vars) {
            return BuildRun::invalid_env_vars();
        }
        let _guard = self.request_semaphore.acquire().await?;
        let (
            BuildResult {
                program_id,
                ttl,
                compile_result,
            },
            read_guard,
        ) = match build_program(
            Arc::clone(&self.config),
            Arc::clone(&self.environments),
            data.0.build,
            Arc::clone(&self.program_lock),
            Arc::clone(&self.job_lock),
        )
        .await
        {
            Ok(result) => result,
            Err(BuildProgramError::EnvironmentNotFound(_)) => {
                return BuildRun::environment_not_found()
            }
            Err(BuildProgramError::CompilationFailed(result)) => {
                return BuildRun::compile_error(result)
            }
            Err(BuildProgramError::ConflictingFilenames) => return BuildRun::invalid_file_names(),
            Err(BuildProgramError::LimitsExceeded(lim)) => {
                return BuildRun::compile_limits_exceeded(lim)
            }
            Err(err) => return Err(err.into()),
        };

        match run_program(
            Arc::clone(&self.config),
            Arc::clone(&self.environments),
            program_id,
            data.0.run,
            &read_guard,
            Arc::clone(&self.job_lock),
        )
        .await
        {
            Ok(run_result) => BuildRun::ok(BuildRunResult {
                program_id,
                ttl,
                build: compile_result,
                run: run_result,
            }),
            Err(RunProgramError::LimitsExceeded(lim)) => BuildRun::run_limits_exceeded(lim),
            Err(err) => Err(err.into()),
        }
    }

    /// Upload and compile a program.
    #[oai(path = "/programs", method = "post", transform = "shield")]
    async fn build_program(&self, data: Json<BuildRequest>) -> Build::Response {
        if !check_mainfile(&data.0.main_file) || !check_files(&data.0.files) {
            return Build::invalid_file_names();
        }
        if !check_env_vars(&data.0.env_vars) {
            return Build::invalid_env_vars();
        }
        let _guard = self.request_semaphore.acquire().await?;
        match build_program(
            Arc::clone(&self.config),
            Arc::clone(&self.environments),
            data.0,
            Arc::clone(&self.program_lock),
            Arc::clone(&self.job_lock),
        )
        .await
        {
            Ok((result, _)) => Build::ok(result),
            Err(BuildProgramError::EnvironmentNotFound(_)) => Build::environment_not_found(),
            Err(BuildProgramError::CompilationFailed(result)) => Build::compile_error(result),
            Err(BuildProgramError::ConflictingFilenames) => Build::invalid_file_names(),
            Err(BuildProgramError::LimitsExceeded(lim)) => Build::compile_limits_exceeded(lim),
            Err(err) => Err(err.into()),
        }
    }

    /// Run a program that has previously been built.
    #[oai(
        path = "/programs/:program_id/run",
        method = "post",
        transform = "shield"
    )]
    async fn run_program(&self, program_id: Path<Uuid>, data: Json<RunRequest>) -> Run::Response {
        if !check_files(&data.0.files) {
            return Run::invalid_file_names();
        }
        if !check_env_vars(&data.0.env_vars) {
            return Run::invalid_env_vars();
        }
        let _guard = self.request_semaphore.acquire().await?;
        match run_program(
            Arc::clone(&self.config),
            Arc::clone(&self.environments),
            program_id.0,
            data.0,
            &self.program_lock.read(program_id.0).await,
            Arc::clone(&self.job_lock),
        )
        .await
        {
            Ok(result) => Run::ok(result),
            Err(RunProgramError::ProgramNotFound) => Run::program_not_found(),
            Err(RunProgramError::LimitsExceeded(lim)) => Run::run_limits_exceeded(lim),
            Err(err) => Err(err.into()),
        }
    }
}

response!(BuildRun = {
    /// Code has been executed successfully.
    Ok(200) => BuildRunResult,
    /// Environment does not exist.
    EnvironmentNotFound(404, error),
    /// Code could not be compiled.
    CompileError(400, error) => RunResult,
    /// File names are not unique.
    InvalidFileNames(400, error),
    /// Environment variable names are not valid.
    InvalidEnvVars(400, error),
    /// The specified compile limits are too high.
    CompileLimitsExceeded(400, error) => Vec<LimitExceeded>,
    /// The specified run limits are too high.
    RunLimitsExceeded(400, error) => Vec<LimitExceeded>,
});

response!(Build = {
    /// Program has been built successfully.
    Ok(201) => BuildResult,
    /// Environment does not exist.
    EnvironmentNotFound(404, error),
    /// Code could not be compiled.
    CompileError(400, error) => RunResult,
    /// File names are not unique.
    InvalidFileNames(400, error),
    /// Environment variable names are not valid.
    InvalidEnvVars(400, error),
    /// The specified compile limits are too high.
    CompileLimitsExceeded(400, error) => Vec<LimitExceeded>,
});

response!(Run = {
    /// Code has been executed successfully.
    Ok(200) => RunResult,
    /// File names are not unique.
    InvalidFileNames(400, error),
    /// Environment variable names are not valid.
    InvalidEnvVars(400, error),
    /// Program does not exist.
    ProgramNotFound(404, error),
    /// The specified run limits are too high.
    RunLimitsExceeded(400, error) => Vec<LimitExceeded>,
});

fn check_filename(name: &str) -> bool {
    let invalid_names = Lazy::new(|| Regex::new(r"^\.*$").unwrap());
    !invalid_names.is_match(name)
}

fn check_files(files: &[File]) -> bool {
    files.iter().all(|f| check_filename(&f.name))
        && files.iter().map(|f| &f.name).collect::<HashSet<_>>().len() == files.len()
}

fn check_mainfile(main_file: &MainFile) -> bool {
    main_file
        .name
        .as_ref()
        .map_or(true, |name| check_filename(name))
}

fn check_env_vars(env_vars: &[EnvVar]) -> bool {
    env_vars.iter().all(|e| e.name != "_")
}
