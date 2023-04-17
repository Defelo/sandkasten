use std::{future::Future, path::Path, process::Stdio, string::FromUtf8Error};

use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
};
use tracing::error;

use crate::schemas::{
    self,
    programs::{ResourceUsage, RunResult},
};

#[derive(Debug)]
pub struct RunConfig<'a> {
    pub nsjail: &'a str,
    pub time: &'a str,
    pub tmpdir: &'a Path,
    pub program: &'a str,
    pub args: &'a [&'a str],
    pub envvars: &'a [(&'a str, &'a str)],
    pub cwd: &'a str,
    pub stdin: Option<&'a str>,
    pub mounts: &'a [Mount<'a>],
    pub limits: Limits,
}

#[derive(Debug)]
pub struct Mount<'a> {
    pub dest: &'a str,
    pub typ: MountType<'a>,
}

#[derive(Debug, Copy, Clone)]
pub enum MountType<'a> {
    ReadOnly {
        src: &'a str,
    },
    ReadWrite {
        src: &'a str,
    },
    Temp {
        /// in MB
        size: u64,
    },
}

#[derive(Debug, Clone, Object, Serialize, Deserialize, PartialEq, Eq)]
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
}

impl RunConfig<'_> {
    pub async fn run(&self) -> Result<RunResult, RunError> {
        let time_path = self.tmpdir.join("time");
        fs::write(&time_path, Vec::new()).await?;

        let mut cmd = tokio::process::Command::new(self.time);
        cmd.arg("-q")
            .args(["-f", "%e %M %x"])
            .args(["-o", &time_path.display().to_string()])
            .arg("--")
            .arg(self.nsjail)
            .arg("-Q")
            .args(["--user", "65534"])
            .args(["--group", "65534"])
            .args(["--hostname", "box"])
            .args(["--cwd", self.cwd])
            .args(["--max_cpus", &self.limits.cpus.to_string()])
            .args(["--time_limit", &self.limits.time.to_string()])
            .args(["--rlimit_as", &self.limits.memory.to_string()])
            .args(["--rlimit_fsize", &self.limits.filesize.to_string()])
            .args(["--rlimit_nofile", &self.limits.file_descriptors.to_string()])
            .args(["--rlimit_nproc", &self.limits.processes.to_string()]);

        for &(name, value) in self.envvars {
            cmd.arg("-E").arg(format!("{name}={value}"));
        }

        for &Mount { dest, typ } in self.mounts {
            match typ {
                MountType::ReadOnly { src } => {
                    cmd.arg("-R").arg(format!("{src}:{dest}"));
                }
                MountType::ReadWrite { src } => {
                    cmd.arg("-B").arg(format!("{src}:{dest}"));
                }
                MountType::Temp { size } => {
                    if size > 0 {
                        cmd.arg("-m").arg(format!("none:{dest}:tmpfs:size={size}M"));
                    }
                }
            };
        }
        cmd.arg("-R").arg("/dev/null");
        cmd.arg("-R").arg("/dev/urandom");
        cmd.arg("-s").arg("/proc/self/fd:/dev/fd");
        cmd.arg("-s").arg("/dev/null:/etc/passwd");

        let mut child = cmd
            .arg("--")
            .arg(self.program)
            .args(self.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()?;

        if let Some(stdin) = &self.stdin {
            let mut handle = child.stdin.take().unwrap();
            handle.write_all(stdin.as_bytes()).await?;
            drop(handle);
        }

        let stdout_reader = BufReader::new(child.stdout.take().unwrap());
        let stderr_reader = BufReader::new(child.stderr.take().unwrap());

        child.wait().await?;

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        stdout_reader
            .take(self.limits.stdout_max_size)
            .read_to_end(&mut stdout)
            .await?;
        stderr_reader
            .take(self.limits.stderr_max_size)
            .read_to_end(&mut stderr)
            .await?;
        let stdout = String::from_utf8_lossy(&stdout).into_owned();
        let stderr = String::from_utf8_lossy(&stderr).into_owned();

        let time_file = fs::read_to_string(time_path).await?;
        let mut tf = time_file.split_whitespace();
        let (time, memory, status) = (|| {
            Some((
                (tf.next()?.parse::<f32>().ok()? * 1000.0) as _,
                tf.next()?.parse().ok()?,
                tf.next()?.parse().ok()?,
            ))
        })()
        .ok_or(RunError::InvalidTimeFile)?;

        Ok(RunResult {
            status,
            stdout,
            stderr,
            resource_usage: ResourceUsage { time, memory },
            limits: self.limits.clone(),
        })
    }
}

#[derive(Debug, Error)]
pub enum RunError {
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("string is not valid utf-8: {0}")]
    StringConversionError(#[from] FromUtf8Error),
    #[error("time file has not been created correctly")]
    InvalidTimeFile,
}

impl Limits {
    pub fn from(
        config_limits: &Self,
        limits: &schemas::programs::LimitsOpt,
    ) -> Result<Self, Vec<LimitExceeded>> {
        let mut errors = Vec::new();
        let mut get = |name, mx, val: Option<_>| {
            let val = val.unwrap_or(mx);
            if val > mx {
                errors.push(LimitExceeded {
                    name,
                    max_value: mx,
                });
            }
            val
        };
        let out = Self {
            cpus: get("cpus", config_limits.cpus, limits.cpus),
            time: get("time", config_limits.time, limits.time),
            memory: get("memory", config_limits.memory, limits.memory),
            tmpfs: get("tmpfs", config_limits.tmpfs, limits.tmpfs),
            filesize: get("filesize", config_limits.filesize, limits.filesize),
            file_descriptors: get(
                "file_descriptors",
                config_limits.file_descriptors,
                limits.file_descriptors,
            ),
            processes: get("processes", config_limits.processes, limits.processes),
            stdout_max_size: get(
                "stdout_max_size",
                config_limits.stdout_max_size,
                limits.stdout_max_size,
            ),
            stderr_max_size: get(
                "stderr_max_size",
                config_limits.stderr_max_size,
                limits.stderr_max_size,
            ),
        };
        if errors.is_empty() {
            Ok(out)
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Object)]
#[oai(read_only_all)]
pub struct LimitExceeded {
    pub name: &'static str,
    pub max_value: u64,
}

pub async fn with_tempdir<P, A>(
    path: P,
    closure: impl FnOnce(P) -> A,
) -> Result<A::Output, std::io::Error>
where
    P: AsRef<Path> + Clone + std::fmt::Debug,
    A: Future,
{
    fs::create_dir_all(&path).await?;
    let out = closure(path.clone()).await;
    if let Err(err) = fs::remove_dir_all(&path).await {
        error!("could not remove tempdir {path:?}: {err}");
    }
    Ok(out)
}
