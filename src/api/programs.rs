use std::{collections::HashSet, sync::Arc};

use key_lock::KeyLock;
use once_cell::unsync::Lazy;
use poem_ext::response;
use poem_openapi::{param::Path, payload::Json, OpenApi};
use regex::Regex;
use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::{
    config::Config,
    environments::Environments,
    program::{build_program, run_program, BuildProgramError, RunProgramError},
    sandbox::LimitExceeded,
    schemas::programs::{
        BuildRequest, BuildResult, BuildRunRequest, BuildRunResult, File, RunRequest, RunResult,
    },
};

use super::Tags;

pub struct ProgramsApi {
    pub config: Arc<Config>,
    pub environments: Arc<Environments>,
    pub compile_lock: KeyLock<Uuid>,
    pub job_semaphore: Semaphore,
}

#[OpenApi(tag = "Tags::Programs")]
impl ProgramsApi {
    /// Build and immediately run a program.
    #[oai(path = "/run", method = "post")]
    async fn run(&self, data: Json<BuildRunRequest>) -> BuildRun::Response {
        if !check_files(&data.0.build.files) || !check_files(&data.0.run.files) {
            return BuildRun::invalid_file_names();
        }
        let _guard = self.job_semaphore.acquire().await?;
        let BuildResult {
            program_id,
            compile_result,
        } = match build_program(
            &self.config,
            &self.environments,
            data.0.build,
            &self.compile_lock,
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
            Err(BuildProgramError::LimitsExceeded(lim)) => {
                return BuildRun::compile_limits_exceeded(lim)
            }
            Err(err) => return Err(err.into()),
        };

        match run_program(&self.config, &self.environments, program_id, data.0.run).await {
            Ok(run_result) => BuildRun::ok(BuildRunResult {
                program_id,
                build: compile_result,
                run: run_result,
            }),
            Err(RunProgramError::LimitsExceeded(lim)) => BuildRun::run_limits_exceeded(lim),
            Err(err) => Err(err.into()),
        }
    }

    /// Upload and compile a program.
    #[oai(path = "/programs", method = "post")]
    async fn build_program(&self, data: Json<BuildRequest>) -> Build::Response {
        if !check_files(&data.0.files) {
            return Build::invalid_file_names();
        }
        let _guard = self.job_semaphore.acquire().await?;
        match build_program(&self.config, &self.environments, data.0, &self.compile_lock).await {
            Ok(result) => Build::ok(result),
            Err(BuildProgramError::EnvironmentNotFound(_)) => Build::environment_not_found(),
            Err(BuildProgramError::CompilationFailed(result)) => Build::compile_error(result),
            Err(BuildProgramError::LimitsExceeded(lim)) => Build::compile_limits_exceeded(lim),
            Err(err) => Err(err.into()),
        }
    }

    /// Run a program that has previously been built.
    #[oai(path = "/programs/:program_id/run", method = "post")]
    async fn run_program(&self, program_id: Path<Uuid>, data: Json<RunRequest>) -> Run::Response {
        if !check_files(&data.0.files) {
            return Run::invalid_file_names();
        }
        let _guard = self.job_semaphore.acquire().await?;
        match run_program(&self.config, &self.environments, program_id.0, data.0).await {
            Ok(result) => Run::ok(result),
            Err(RunProgramError::ProgramNotFound) => Run::not_found(),
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
    /// The specified compile limits are too high.
    CompileLimitsExceeded(400, error) => Vec<LimitExceeded>,
});

response!(Run = {
    /// Code has been executed successfully.
    Ok(200) => RunResult,
    /// File names are not unique.
    InvalidFileNames(400, error),
    /// Program does not exist.
    NotFound(404, error),
    /// The specified run limits are too high.
    RunLimitsExceeded(400, error) => Vec<LimitExceeded>,
});

response!(DeleteProgram = {
    /// Program has been deleted.
    Ok(200),
    /// Program does not exist.
    NotFound(404, error),
});

fn check_files(files: &[File]) -> bool {
    let invalid_names = Lazy::new(|| Regex::new(r"^\.*$").unwrap());
    files.iter().all(|f| !invalid_names.is_match(&f.name))
        && files.iter().map(|f| &f.name).collect::<HashSet<_>>().len() == files.len()
}
