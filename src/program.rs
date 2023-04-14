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
    sandbox::{with_tempdir, LimitExceeded, Limits, Mount, MountType, RunConfig, RunError},
    schemas::programs::{BuildRequest, BuildResult, RunRequest, RunResult},
};

/// Store (and compile) the uploaded program into a directory in the local fs. Return a unique
/// identifier for the program.
pub async fn build_program(
    config: &Config,
    environments: &Environments,
    data: BuildRequest,
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
    data: RunRequest,
) -> Result<RunResult, RunProgramError> {
    let path = config.programs_dir.join(program_id.to_string());
    if !fs::try_exists(&path).await? {
        return Err(RunProgramError::ProgramNotFound);
    }

    let limits = Limits::from(&config.run_limits, &data.run_limits)
        .map_err(RunProgramError::LimitsExceeded)?;

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
            execute_program(ExecuteProgram {
                nsjail: &environments.nsjail_path,
                time: &environments.time_path,
                run_script: &run_script,
                main_file: &main_file,
                data: &data,
                program_path: &path.join("files"),
                tmpdir: &tmpdir,
                limits,
            })
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
    #[error("limits exceeded: {0:?}")]
    LimitsExceeded(Vec<LimitExceeded>),
}

#[derive(Debug, Error)]
pub enum RunProgramError {
    #[error("program does not exist")]
    ProgramNotFound,
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("run error: {0}")]
    RunError(#[from] RunError),
    #[error("limits exceeded: {0:?}")]
    LimitsExceeded(Vec<LimitExceeded>),
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
    data: BuildRequest,
    env: &Environment,
    path: &Path,
) -> Result<Option<RunResult>, BuildProgramError> {
    let limits = Limits::from(&config.compile_limits, &data.compile_limits)
        .map_err(BuildProgramError::LimitsExceeded)?;

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
                compile_program(CompileProgram {
                    nsjail: &environments.nsjail_path,
                    time: &environments.time_path,
                    compile_script,
                    data: &data,
                    path,
                    tmpdir: &tmpdir,
                    limits,
                })
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
    CompileProgram {
        nsjail,
        time,
        compile_script,
        data,
        path,
        tmpdir,
        limits,
    }: CompileProgram<'_>,
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
                typ: MountType::Temp { size: limits.tmpfs },
            },
        ],
        limits,
    }
    .run()
    .await
}

struct CompileProgram<'a> {
    nsjail: &'a str,
    time: &'a str,
    compile_script: &'a str,
    data: &'a BuildRequest,
    path: &'a Path,
    tmpdir: &'a Path,
    limits: Limits,
}

async fn execute_program(
    ExecuteProgram {
        nsjail,
        time,
        run_script,
        main_file,
        data,
        program_path,
        tmpdir,
        limits,
    }: ExecuteProgram<'_>,
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
                typ: MountType::ReadOnly {
                    src: &tmpdir.join("box").display().to_string(),
                },
            },
            Mount {
                dest: "/tmp",
                typ: MountType::Temp { size: limits.tmpfs },
            },
        ],
        limits,
    }
    .run()
    .await
}

struct ExecuteProgram<'a> {
    nsjail: &'a str,
    time: &'a str,
    run_script: &'a str,
    main_file: &'a str,
    data: &'a RunRequest,
    program_path: &'a Path,
    tmpdir: &'a Path,
    limits: Limits,
}
