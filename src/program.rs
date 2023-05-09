use std::{
    path::Path,
    sync::Arc,
    time::{self, UNIX_EPOCH},
};

use key_rwlock::KeyRwLock;
use sandkasten_client::schemas::programs::{
    BuildRequest, BuildResult, LimitExceeded, Limits, RunRequest, RunResult,
};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::{fs, sync::OwnedRwLockReadGuard};
use tracing::{debug, error};
use uuid::Uuid;

use crate::{
    config::Config,
    environments::{Environment, Environments},
    sandbox::{with_tempdir, Mount, MountType, RunConfig, RunError},
};

/// Store (and compile) the uploaded program into a directory in the local fs. Return a unique
/// identifier for the program.
pub async fn build_program(
    config: Arc<Config>,
    environments: Arc<Environments>,
    data: BuildRequest,
    program_lock: Arc<KeyRwLock<Uuid>>,
    job_lock: Arc<KeyRwLock<Uuid>>,
) -> Result<(BuildResult, OwnedRwLockReadGuard<()>), BuildProgramError> {
    let env = environments.environments.get(&data.environment).ok_or(
        BuildProgramError::EnvironmentNotFound(data.environment.clone()),
    )?;

    let hash = Sha256::new()
        .chain_update(postcard::to_stdvec(&(
            &env.name,
            &env.version,
            &env.compile_script,
            &data.files,
            &data.env_vars,
        ))?)
        .finalize();
    let id = Uuid::from_u128(
        hash.into_iter()
            .take(16)
            .fold(0, |acc, x| (acc << 8) | x as u128),
    );
    let path = config.programs_dir.join(id.to_string());

    let _guard = program_lock.read(id).await;
    if let Some(cached) = get_cached_program(id, &path, &config, env).await? {
        return Ok((cached, _guard));
    }
    drop(_guard);

    let _guard = program_lock.write(id).await;
    if let Some(cached) = get_cached_program(id, &path, &config, env).await? {
        return Ok((cached, _guard.downgrade()));
    }

    match store_program(&config, &environments, data, env, &path, &job_lock).await {
        Ok(result) => {
            if let Some(result) = &result {
                let serialized = postcard::to_stdvec(result)?;
                fs::write(path.join("compile_result"), serialized).await?;
            }
            fs::write(path.join("ok"), []).await?;
            fs::write(
                path.join("last_run"),
                time::SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    .to_string(),
            )
            .await?;
            Ok((
                BuildResult {
                    program_id: id,
                    ttl: config.program_ttl,
                    compile_result: result,
                },
                _guard.downgrade(),
            ))
        }
        Err(err) => {
            if let Err(err) = fs::remove_dir_all(&path).await {
                error!("could not remove program directory {path:?}: {err}");
            }
            Err(err)
        }
    }
}

async fn get_cached_program(
    program_id: Uuid,
    path: &Path,
    config: &Config,
    env: &Environment,
) -> Result<Option<BuildResult>, BuildProgramError> {
    if !fs::try_exists(path.join("ok")).await? {
        return Ok(None);
    }

    let compile_result = if env.compile_script.is_some() {
        let serialized = fs::read(path.join("compile_result")).await?;
        Some(postcard::from_bytes(&serialized)?)
    } else {
        None
    };
    Ok(Some(BuildResult {
        program_id,
        ttl: config.program_ttl,
        compile_result,
    }))
}

/// Run a given program and return its output.
pub async fn run_program(
    config: Arc<Config>,
    environments: Arc<Environments>,
    program_id: Uuid,
    data: RunRequest,
    _program_guard: OwnedRwLockReadGuard<()>,
    job_lock: Arc<KeyRwLock<Uuid>>,
) -> Result<RunResult, RunProgramError> {
    let limits = data
        .run_limits
        .check(&config.run_limits)
        .map_err(RunProgramError::LimitsExceeded)?;

    let path = config.programs_dir.join(program_id.to_string());
    if !fs::try_exists(&path).await? {
        return Err(RunProgramError::ProgramNotFound);
    }

    std::fs::write(
        path.join("last_run"),
        time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string(),
    )?;

    let run_script = fs::read_to_string(path.join("run_script")).await?;
    let main_file = fs::read_to_string(path.join("main_file")).await?;

    let job_id = Uuid::new_v4();
    let _guard = job_lock.write(job_id).await;
    Ok(with_tempdir(
        config.jobs_dir.join(job_id.to_string()),
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

pub async fn prune_programs(
    config: &Config,
    program_lock: Arc<KeyRwLock<Uuid>>,
) -> Result<(), std::io::Error> {
    async fn prune(dir: fs::DirEntry, prune_until: u64) -> bool {
        if !fs::try_exists(dir.path()).await.unwrap_or(false) {
            return false;
        }
        match fs::read_to_string(dir.path().join("last_run"))
            .await
            .ok()
            .and_then(|lr| lr.parse::<u64>().ok())
        {
            Some(last_run) if last_run > prune_until => return false,
            _ => {}
        }
        let result = fs::remove_dir_all(dir.path()).await;
        match result {
            Ok(_) => true,
            Err(err) => {
                error!(
                    "could not delete old program at {}: {err}",
                    dir.path().display()
                );
                false
            }
        }
    }

    debug!("pruning programs (ttl={})", config.program_ttl);
    let mut it = fs::read_dir(&config.programs_dir).await?;
    let mut pruned = 0;
    let prune_until = time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - config.program_ttl;
    while let Some(dir) = it.next_entry().await? {
        let Ok(program_id)  = dir.file_name().to_string_lossy().parse::<Uuid>() else {
            pruned += prune(dir, prune_until).await as usize;
            continue;
        };
        if let Ok(_guard) = program_lock.try_write(program_id).await {
            pruned += prune(dir, prune_until).await as usize;
            continue;
        }
        tokio::spawn({
            let program_lock = Arc::clone(&program_lock);
            async move {
                let _guard = program_lock.write(program_id).await;
                if prune(dir, prune_until).await {
                    debug!("successfully removed one old program");
                }
            }
        });
    }
    debug!("successfully removed {pruned} old program(s)");
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
    job_lock: &KeyRwLock<Uuid>,
) -> Result<Option<RunResult>, BuildProgramError> {
    let limits = data
        .compile_limits
        .check(&config.compile_limits)
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
        let job_id = Uuid::new_v4();
        let _guard = job_lock.write(job_id).await;
        let result = with_tempdir(
            config.jobs_dir.join(job_id.to_string()),
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
        envvars: &data
            .env_vars
            .iter()
            .map(|e| (e.name.as_str(), e.value.as_str()))
            .collect::<Vec<_>>(),
        cwd: "/box",
        stdin: None,
        mounts: &[
            Mount {
                dest: "/nix/store",
                typ: MountType::ReadOnly { src: "/nix/store" },
            },
            Mount {
                dest: "/program",
                typ: MountType::ReadWrite {
                    src: &path.join("files").display().to_string(),
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
        args: &std::iter::once(main_file)
            .chain(data.args.iter().map(|f| f.as_str()))
            .collect::<Vec<_>>(),
        envvars: &data
            .env_vars
            .iter()
            .map(|e| (e.name.as_str(), e.value.as_str()))
            .collect::<Vec<_>>(),
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
