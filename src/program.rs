use std::path::Path;

use key_lock::KeyLock;
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::fs;
use tracing::error;
use uuid::Uuid;

use crate::{
    config::Config,
    environments::{Environment, Environments},
    sandbox::{with_tempdir, Limits, Mount, MountType, RunConfig, RunError},
    schemas::{CreateProgramRequest, CreateResult, RunProgramRequest, RunResult},
};

/// Store (and compile) the uploaded program into a directory in the local fs. Return a unique
/// identifier for the program.
pub async fn create_program(
    config: &Config,
    environments: &Environments,
    data: CreateProgramRequest,
    compile_lock: &KeyLock<Uuid>,
) -> Result<CreateResult, CreateProgramError> {
    let env = environments.environments.get(&data.environment).ok_or(
        CreateProgramError::EnvironmentNotFound(data.environment.clone()),
    )?;

    let hash = Sha256::new()
        .chain_update(postcard::to_stdvec(&(
            &env.name,
            &env.version,
            &env.compile_script,
            &data.files,
        ))?)
        .finalize();

    let id = Uuid::from_u128(
        hash.into_iter()
            .take(16)
            .fold(0, |acc, x| (acc << 8) | x as u128),
    );
    let _guard = compile_lock.lock(id).await;

    let path = config.programs_dir.join(id.to_string());
    if fs::try_exists(path.join("environment")).await? {
        drop(_guard);
        let compile_result = if env.compile_script.is_some() {
            let serialized = fs::read(path.join("compile_result")).await?;
            Some(postcard::from_bytes(&serialized)?)
        } else {
            None
        };
        return Ok(CreateResult {
            program_id: id,
            compile_result,
        });
    }

    match store_program(config, environments, data, env, &path).await {
        Ok(result) => {
            if let Some(result) = &result {
                let serialized = postcard::to_stdvec(result)?;
                fs::write(path.join("compile_result"), serialized).await?;
            }
            Ok(CreateResult {
                program_id: id,
                compile_result: result,
            })
        }
        Err(err) => {
            if let Err(err) = fs::remove_dir_all(&path).await {
                error!("could not remove program directory {path:?}: {err}");
            }
            Err(err)
        }
    }
}

/// Run a given program and return its output.
pub async fn run_program(
    config: &Config,
    environments: &Environments,
    program_id: Uuid,
    data: RunProgramRequest,
) -> anyhow::Result<RunResult> {
    todo!()
}

/// Delete a program's directly and all its contents.
pub async fn delete_program(
    config: &Config,
    environments: &Environments,
    program_id: Uuid,
) -> anyhow::Result<()> {
    todo!()
}

// TODO Delete all programs that have not been run recently.

#[derive(Debug, Error)]
pub enum CreateProgramError {
    #[error("could not find environment {0}")]
    EnvironmentNotFound(String),
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("run error: {0}")]
    RunError(#[from] RunError),
    #[error("compilation failed (exit code {})", .0.status)]
    CompilationFailed(RunResult),
    #[error("postcard error: {0}")]
    PostcardError(#[from] postcard::Error),
}

async fn store_program(
    config: &Config,
    environments: &Environments,
    data: CreateProgramRequest,
    env: &Environment,
    path: &Path,
) -> Result<Option<RunResult>, CreateProgramError> {
    fs::create_dir_all(path.join("files")).await?;
    // TODO write all information for `run_program`
    fs::write(path.join("environment"), &data.environment).await?;

    let compile_result = if let Some(compile_script) = &env.compile_script {
        // run compile script and copy output to program dir
        let result = with_tempdir(
            config.jobs_dir.join(Uuid::new_v4().to_string()),
            |tmpdir| async move {
                for file in &data.files {
                    fs::write(tmpdir.join(&file.name), &file.content).await?;
                }
                compile_program(
                    &environments.nsjail_path,
                    compile_script,
                    &data,
                    path,
                    &tmpdir,
                )
                .await
            },
        )
        .await??;
        if result.status != 0 {
            return Err(CreateProgramError::CompilationFailed(result));
        }
        Some(result)
    } else {
        // copy files to program dir
        for file in data.files {
            fs::write(path.join("files").join(file.name), file.content).await?;
        }
        None
    };

    Ok(compile_result)
}

async fn compile_program(
    nsjail: &str,
    compile_script: &str,
    data: &CreateProgramRequest,
    path: &Path,
    tmpdir: &Path,
) -> Result<RunResult, RunError> {
    RunConfig {
        nsjail,
        program: compile_script,
        args: &data
            .files
            .iter()
            .map(|f| f.name.as_str())
            .collect::<Vec<_>>(),
        cwd: "/box",
        stdin: None,
        mounts: &[
            Mount {
                dest: "/nix/store",
                typ: MountType::ReadOnly { src: "/nix/store" },
            },
            Mount {
                dest: "/box",
                typ: MountType::ReadOnly {
                    src: &tmpdir.display().to_string(),
                },
            },
            Mount {
                dest: "/out",
                typ: MountType::ReadWrite {
                    src: &path.join("files").display().to_string(),
                },
            },
            Mount {
                dest: "/tmp",
                typ: MountType::Temp { size: 1024 },
            },
        ],
        limits: Limits {
            cpus: 1,
            time: data.compile_limits.timeout.unwrap_or(10),
            memory: data.compile_limits.memory_limit.unwrap_or(1024),
            filesize: 32,
            file_descriptors: 100,
            processes: 100,
        },
    }
    .run()
    .await
}
