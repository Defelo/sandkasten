use std::{future::Future, path::Path, process::Stdio, string::FromUtf8Error};

use sandkasten_client::schemas::programs::{Limits, ResourceUsage, RunResult};
use thiserror::Error;
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
};
use tracing::error;

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

        if self.limits.network {
            cmd.arg("-N").args(["-R", "/etc/resolv.conf"]);
        }

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
