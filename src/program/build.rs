use std::{
    ffi::OsStr,
    os::unix::prelude::OsStrExt,
    path::Path,
    sync::Arc,
    time::{self, UNIX_EPOCH},
};

use key_rwlock::KeyRwLock;
use sandkasten_client::schemas::programs::{
    BuildRequest, BuildResult, LimitExceeded, Limits, RunResult,
};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::{fs, sync::OwnedRwLockReadGuard};
use tracing::error;
use uuid::Uuid;

use super::{mounts_from_closure, with_tempdir};
use crate::{
    config::Config,
    environments::{Environment, Environments},
    sandbox::{Mount, MountType, RunConfig, RunError},
};

/// Build and store the uploaded program into a directory in the local fs.
/// Return a unique identifier for the program.
pub async fn build_program(
    config: Arc<Config>,
    environments: Arc<Environments>,
    data: BuildRequest,
    program_lock: Arc<KeyRwLock<Uuid>>,
    job_lock: Arc<KeyRwLock<Uuid>>,
) -> Result<(BuildResult, OwnedRwLockReadGuard<()>), BuildProgramError> {
    let env = environments
        .get(&data.environment)
        .ok_or(BuildProgramError::EnvironmentNotFound(
            data.environment.clone(),
        ))?;

    // compute the program id by hashing the request data
    let hash = Sha256::new()
        .chain_update(postcard::to_stdvec(&(
            &env.name,
            &env.version,
            &env.compile_script,
            &env.run_script,
            &env.closure,
            &env.sandkasten_version,
            &data.main_file,
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

    // check if the program has already been built before
    let _guard = program_lock.read(id).await;
    if let Some(cached) = get_cached_program(id, &path, &config, env).await? {
        return Ok((cached, _guard));
    }
    drop(_guard);

    // acquire the write lock and start building the program
    let _guard = program_lock.write(id).await;
    if let Some(cached) = get_cached_program(id, &path, &config, env).await? {
        return Ok((cached, _guard.downgrade()));
    }

    match store_in_directory(&config, data, env, &path, &job_lock).await {
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
                    cached: false,
                    compile_result: result,
                },
                _guard.downgrade(),
            ))
        }
        Err(err) => {
            if fs::try_exists(&path).await? {
                if let Err(err) = fs::remove_dir_all(&path).await {
                    error!("Failed to remove program directory {path:?}: {err:#}");
                }
            }
            Err(err)
        }
    }
}

/// Try to get a program that has been built previoulsy by id.
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
        cached: true,
        compile_result,
    }))
}

/// Build a program from a given [`BuildRequest`] and store the result at the
/// given `path`.
async fn store_in_directory(
    config: &Config,
    build_request: BuildRequest,
    environment: &Environment,
    program_directory: &Path,
    job_lock: &KeyRwLock<Uuid>,
) -> Result<Option<RunResult>, BuildProgramError> {
    // check if limits have been exceeded and use default values from config for
    // empty fields
    let compile_limits = build_request
        .compile_limits
        .check(&config.compile_limits)
        .map_err(BuildProgramError::LimitsExceeded)?;

    // write metadata that is used later for running the program
    fs::create_dir_all(program_directory.join("files")).await?;
    fs::write(
        program_directory.join("run_script"),
        &environment.run_script,
    )
    .await?;
    fs::write(
        program_directory.join("closure"),
        &environment.closure.as_os_str().as_bytes(),
    )
    .await?;

    let main_file_name = build_request
        .main_file
        .name
        .as_ref()
        .unwrap_or(&environment.default_main_file_name);
    if build_request
        .files
        .iter()
        .any(|f| f.name == *main_file_name)
    {
        return Err(BuildProgramError::ConflictingFilenames);
    }
    fs::write(program_directory.join("main_file"), main_file_name).await?;

    if let Some(compile_script) = &environment.compile_script {
        // if the environment has a compile script, run it and write the output
        // to the program directory
        Ok(Some(
            compile_program(CompileProgram {
                config,
                job_lock,
                build_request: &build_request,
                environment,
                compile_script,
                program_directory,
                compile_limits,
                main_file_name,
            })
            .await?,
        ))
    } else {
        // copy files to program dir
        fs::write(
            program_directory.join("files").join(main_file_name),
            build_request.main_file.content,
        )
        .await?;
        for file in build_request.files {
            fs::write(
                program_directory.join("files").join(file.name),
                file.content,
            )
            .await?;
        }
        Ok(None)
    }
}

// Run the compile script of an environment to build a program
async fn compile_program(
    CompileProgram {
        config,
        job_lock,
        build_request,
        environment,
        compile_script,
        program_directory,
        compile_limits,
        main_file_name,
    }: CompileProgram<'_>,
) -> Result<RunResult, BuildProgramError> {
    let job_id = Uuid::new_v4();
    let _guard = job_lock.write(job_id).await;

    // collect command line arguments and environment variables from build request
    let args = std::iter::once(main_file_name)
        .chain(build_request.files.iter().map(|f| f.name.as_str()))
        .collect::<Vec<_>>();
    let envvars = build_request
        .env_vars
        .iter()
        .map(|e| (e.name.as_str(), e.value.as_str()))
        .collect::<Vec<_>>();

    with_tempdir(config.jobs_dir.join(job_id.to_string()), |tmpdir| async {
        let tmpdir = { tmpdir }; // move tmpdir into async block

        // create working directory for compile script and copy files from build request
        // into it
        fs::create_dir_all(tmpdir.join("box")).await?;
        fs::write(
            tmpdir.join("box").join(main_file_name),
            &build_request.main_file.content,
        )
        .await?;
        for file in &build_request.files {
            fs::write(tmpdir.join("box").join(&file.name), &file.content).await?;
        }

        let mut mounts = vec![
            Mount {
                dest: OsStr::new("/program").into(),
                typ: MountType::ReadWrite {
                    src: program_directory.join("files").into_os_string().into(),
                },
            },
            Mount {
                dest: OsStr::new("/box").into(),
                typ: MountType::ReadOnly {
                    src: tmpdir.join("box").into_os_string().into(),
                },
            },
            Mount {
                dest: OsStr::new("/tmp").into(),
                typ: MountType::Temp {
                    size: compile_limits.tmpfs,
                },
            },
        ];
        mounts.extend(mounts_from_closure(&environment.closure).await?);

        // run the compile script
        RunConfig {
            nsjail: &config.nsjail_path,
            time: &config.time_path,
            use_cgroup: config.use_cgroup,
            tmpdir: &tmpdir,
            program: compile_script,
            args: &args,
            envvars: &envvars,
            cwd: "/box",
            stdin: None,
            mounts: &mounts,
            limits: compile_limits,
        }
        .run()
        .await
        .map_err(Into::into)
    })
    .await?
    .and_then(|result| {
        if result.status == 0 {
            Ok(result)
        } else {
            Err(BuildProgramError::CompilationFailed(result))
        }
    })
}

struct CompileProgram<'a> {
    config: &'a Config,
    job_lock: &'a KeyRwLock<Uuid>,
    build_request: &'a BuildRequest,
    environment: &'a Environment,
    compile_script: &'a str,
    program_directory: &'a Path,
    compile_limits: Limits,
    main_file_name: &'a str,
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
    #[error("conflicting filenames")]
    ConflictingFilenames,
    #[error("limits exceeded: {0:?}")]
    LimitsExceeded(Vec<LimitExceeded>),
}
