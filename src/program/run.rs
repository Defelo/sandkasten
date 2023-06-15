use std::{
    ffi::OsStr,
    path::PathBuf,
    sync::Arc,
    time::{self, UNIX_EPOCH},
};

use key_rwlock::KeyRwLock;
use sandkasten_client::schemas::programs::{LimitExceeded, RunRequest, RunResult};
use thiserror::Error;
use tokio::{fs, sync::OwnedRwLockReadGuard};
use uuid::Uuid;

use super::{mounts_from_closure, with_tempdir};
use crate::{
    config::Config,
    sandbox::{Mount, MountType, RunConfig, RunError},
};

/// Run a given program and return its output.
pub async fn run_program(
    config: Arc<Config>,
    program_id: Uuid,
    run_request: RunRequest,
    _program_guard: &OwnedRwLockReadGuard<()>,
    job_lock: Arc<KeyRwLock<Uuid>>,
) -> Result<RunResult, RunProgramError> {
    // check if limits have been exceeded and use default values from config for
    // empty fields
    let run_limits = run_request
        .run_limits
        .check(&config.run_limits)
        .map_err(RunProgramError::LimitsExceeded)?;

    // check that the program's directory exists
    let path = config.programs_dir.join(program_id.to_string());
    if !fs::try_exists(&path).await? {
        return Err(RunProgramError::ProgramNotFound);
    }

    // update the program's last run timestamp
    std::fs::write(
        path.join("last_run"),
        time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string(),
    )?;

    // read environment metadata from program directory
    let run_script = fs::read_to_string(path.join("run_script")).await?;
    let main_file = fs::read_to_string(path.join("main_file")).await?;
    let closure = PathBuf::from(fs::read_to_string(path.join("closure")).await?);

    // collect command line arguments and environment variables from run request
    let args = std::iter::once(main_file.as_str())
        .chain(run_request.args.iter().map(|f| f.as_str()))
        .collect::<Vec<_>>();
    let envvars = run_request
        .env_vars
        .iter()
        .map(|e| (e.name.as_str(), e.value.as_str()))
        .collect::<Vec<_>>();

    let job_id = Uuid::new_v4();
    let _guard = job_lock.write(job_id).await;
    with_tempdir(config.jobs_dir.join(job_id.to_string()), |tmpdir| async {
        let tmpdir = tmpdir; // move tmpdir into async block

        // create working directory and copy files from run request into it
        fs::create_dir_all(tmpdir.join("box")).await?;
        for file in &run_request.files {
            fs::write(tmpdir.join("box").join(&file.name), &file.content).await?;
        }

        let mut mounts = vec![
            Mount {
                dest: OsStr::new("/program").into(),
                typ: MountType::ReadOnly {
                    src: path.join("files").into_os_string().into(),
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
                    size: run_limits.tmpfs,
                },
            },
        ];
        mounts.extend(mounts_from_closure(&closure).await?);

        // run the program
        RunConfig {
            nsjail: &config.nsjail_path,
            time: &config.time_path,
            use_cgroup: config.use_cgroup,
            tmpdir: &tmpdir,
            program: &run_script,
            args: &args,
            envvars: &envvars,
            cwd: "/box",
            stdin: run_request.stdin.as_deref(),
            mounts: &mounts,
            limits: run_limits,
        }
        .run()
        .await
    })
    .await?
    .map_err(Into::into)
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
