use std::{
    path::Path,
    time::{self, UNIX_EPOCH},
};

use key_lock::KeyLock;
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::fs;
use tracing::{debug, error};
use uuid::Uuid;

use crate::{
    config::Config,
    environments::{Environment, Environments},
    sandbox::{with_tempdir, Limits, Mount, MountType, RunConfig, RunError},
    schemas::{BuildProgramRequest, BuildResult, RunProgramRequest, RunResult},
};

/// Store (and compile) the uploaded program into a directory in the local fs. Return a unique
/// identifier for the program.
pub async fn build_program(
    config: &Config,
    environments: &Environments,
    data: BuildProgramRequest,
    compile_lock: &KeyLock<Uuid>,
) -> Result<BuildResult, BuildProgramError> {
    let env = environments.environments.get(&data.environment).ok_or(
        BuildProgramError::EnvironmentNotFound(data.environment.clone()),
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
    if fs::try_exists(path.join("compile_result")).await? {
        drop(_guard);
        let compile_result = if env.compile_script.is_some() {
            let serialized = fs::read(path.join("compile_result")).await?;
            Some(postcard::from_bytes(&serialized)?)
        } else {
            None
        };
        return Ok(BuildResult {
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
            Ok(BuildResult {
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
) -> Result<RunResult, RunProgramError> {
    let path = config.programs_dir.join(program_id.to_string());
    if !fs::try_exists(&path).await? {
        return Err(RunProgramError::ProgramNotFound);
    }

    fs::write(
        path.join("last_run"),
        time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string(),
    )
    .await?;

    let run_script = fs::read_to_string(path.join("run_script")).await?;
    let main_file = fs::read_to_string(path.join("main_file")).await?;

    Ok(with_tempdir(
        config.jobs_dir.join(Uuid::new_v4().to_string()),
        |tmpdir| async move {
            fs::create_dir_all(tmpdir.join("box")).await?;
            for file in &data.files {
                fs::write(tmpdir.join("box").join(&file.name), &file.content).await?;
            }
            execute_program(
                &environments.nsjail_path,
                &environments.time_path,
                &run_script,
                &main_file,
                &data,
                &path.join("files"),
                &tmpdir,
            )
            .await
        },
    )
    .await??)
}

/// Delete a program's directly and all its contents.
pub async fn delete_program(config: &Config, program_id: Uuid) -> Result<(), DeleteProgramError> {
    let path = config.programs_dir.join(program_id.to_string());
    if !fs::try_exists(&path).await? {
        return Err(DeleteProgramError::ProgramNotFound);
    }
    fs::remove_dir_all(path).await?;
    Ok(())
}

pub async fn prune_programs(config: &Config) -> Result<(), std::io::Error> {
    debug!("pruning programs (ttl={})", config.program_ttl);
    let mut it = fs::read_dir(&config.programs_dir).await?;
    let now = time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut pruned = 0;
    while let Some(dir) = it.next_entry().await? {
        if fs::metadata(dir.path())
            .await?
            .created()?
            .elapsed()
            .unwrap()
            .as_secs()
            < config.program_ttl
        {
            continue;
        }
        match fs::read_to_string(dir.path().join("last_run"))
            .await
            .ok()
            .and_then(|lr| lr.parse::<u64>().ok())
        {
            Some(last_run) if now < last_run + config.program_ttl => {}
            _ => {
                if let Err(err) = fs::remove_dir_all(dir.path()).await {
                    error!(
                        "could not delete old program at {}: {err}",
                        dir.path().display()
                    );
                } else {
                    pruned += 1;
                }
            }
        }
    }
    debug!("successfully removed {pruned} old programs");
    Ok(())
}

#[derive(Debug, Error)]
pub enum BuildProgramError {
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
    #[error("no main file")]
    NoMainFile,
}

#[derive(Debug, Error)]
pub enum RunProgramError {
    #[error("program does not exist")]
    ProgramNotFound,
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("run error: {0}")]
    RunError(#[from] RunError),
}

#[derive(Debug, Error)]
pub enum DeleteProgramError {
    #[error("program does not exist")]
    ProgramNotFound,
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
}

async fn store_program(
    config: &Config,
    environments: &Environments,
    data: BuildProgramRequest,
    env: &Environment,
    path: &Path,
) -> Result<Option<RunResult>, BuildProgramError> {
    fs::create_dir_all(path.join("files")).await?;
    fs::write(path.join("run_script"), &env.run_script).await?;
    fs::write(
        path.join("main_file"),
        &data
            .files
            .first()
            .ok_or(BuildProgramError::NoMainFile)?
            .name,
    )
    .await?;

    let compile_result = if let Some(compile_script) = &env.compile_script {
        // run compile script and copy output to program dir
        let result = with_tempdir(
            config.jobs_dir.join(Uuid::new_v4().to_string()),
            |tmpdir| async move {
                fs::create_dir_all(tmpdir.join("box")).await?;
                for file in &data.files {
                    fs::write(tmpdir.join("box").join(&file.name), &file.content).await?;
                }
                compile_program(
                    &environments.nsjail_path,
                    &environments.time_path,
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
            return Err(BuildProgramError::CompilationFailed(result));
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
    time: &str,
    compile_script: &str,
    data: &BuildProgramRequest,
    path: &Path,
    tmpdir: &Path,
) -> Result<RunResult, RunError> {
    RunConfig {
        nsjail,
        time,
        tmpdir,
        program: compile_script,
        args: &data
            .files
            .iter()
            .map(|f| f.name.as_str())
            .collect::<Vec<_>>(),
        envvars: &[],
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
                    src: &tmpdir.join("box").display().to_string(),
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
            time: data.compile_limits.time.unwrap_or(10),
            memory: data.compile_limits.memory.unwrap_or(1024),
            filesize: 32,
            file_descriptors: 100,
            processes: 100,
        },
    }
    .run()
    .await
}

async fn execute_program(
    nsjail: &str,
    time: &str,
    run_script: &str,
    main_file: &str,
    data: &RunProgramRequest,
    program_path: &Path,
    tmpdir: &Path,
) -> Result<RunResult, RunError> {
    RunConfig {
        nsjail,
        time,
        tmpdir,
        program: run_script,
        args: &data.args.iter().map(|f| f.as_str()).collect::<Vec<_>>(),
        envvars: &[("MAIN", main_file)],
        cwd: "/box",
        stdin: data.stdin.as_deref(),
        mounts: &[
            Mount {
                dest: "/nix/store",
                typ: MountType::ReadOnly { src: "/nix/store" },
            },
            Mount {
                dest: "/program",
                typ: MountType::ReadOnly {
                    src: &program_path.display().to_string(),
                },
            },
            Mount {
                dest: "/box",
                typ: MountType::ReadWrite {
                    src: &tmpdir.join("box").display().to_string(),
                },
            },
            Mount {
                dest: "/tmp",
                typ: MountType::Temp { size: 1024 },
            },
        ],
        limits: Limits {
            cpus: 1,
            time: data.run_limits.time.unwrap_or(10),
            memory: data.run_limits.memory.unwrap_or(1024),
            filesize: 32,
            file_descriptors: 100,
            processes: 100,
        },
    }
    .run()
    .await
}
