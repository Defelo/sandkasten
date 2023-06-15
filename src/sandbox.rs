use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    path::Path,
    process::Stdio,
    string::FromUtf8Error,
};

use sandkasten_client::schemas::programs::{Limits, ResourceUsage, RunResult};
use thiserror::Error;
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
};
use tracing::error;

#[derive(Debug)]
pub struct RunConfig<'a> {
    pub nsjail: &'a Path,
    pub time: &'a Path,
    pub tmpdir: &'a Path,
    pub program: &'a str,
    pub args: &'a [&'a str],
    pub envvars: &'a [(&'a str, &'a str)],
    pub cwd: &'a str,
    pub stdin: Option<&'a str>,
    pub mounts: &'a [Mount<'a>],
    pub limits: Limits,
    pub use_cgroup: bool,
}

#[derive(Debug)]
pub struct Mount<'a> {
    pub dest: Cow<'a, OsStr>,
    pub typ: MountType<'a>,
}

#[derive(Debug, Clone)]
pub enum MountType<'a> {
    ReadOnly {
        src: Cow<'a, OsStr>,
    },
    ReadWrite {
        src: Cow<'a, OsStr>,
    },
    Temp {
        /// Size of tmpfs in MB
        size: u64,
    },
}

impl RunConfig<'_> {
    pub async fn run(&self) -> Result<RunResult, RunError> {
        // create an empty file which will be used by time to report the resource usage
        // of the program
        let time_path = self.tmpdir.join("time");
        fs::write(&time_path, Vec::new()).await?;

        // construct the time/nsjail command
        let mut cmd = tokio::process::Command::new(self.time);
        cmd.arg("--quiet")
            .args(["--format", "%e %M %x"]) // elapsed time in seconds, max memory usage, exit code
            .arg("--output")
            .arg(&time_path)
            .arg("--")
            .arg(self.nsjail)
            .arg("--really_quiet") // log fatal messages only
            .args(["--user", "65534"]) // user: nobody
            .args(["--group", "65534"]) // group: nobody
            .args(["--hostname", "box"])
            .args(["--cwd", self.cwd]) // current working directory
            // resource limits:
            .args(["--max_cpus", &self.limits.cpus.to_string()])
            .args(["--time_limit", &self.limits.time.to_string()]) // in seconds
            .args(["--rlimit_fsize", &self.limits.filesize.to_string()]) // in MB
            .args(["--rlimit_nofile", &self.limits.file_descriptors.to_string()]);

        if self.use_cgroup {
            cmd.args(["--detect_cgroupv2"])
                .args([
                    "--cgroup_mem_max", // in bytes
                    &(self.limits.memory * 1000 * 1000).to_string(),
                ])
                .args(["--cgroup_mem_swap_max", "0"])
                .args(["--cgroup_pids_max", &self.limits.processes.to_string()]);
        } else {
            cmd.args(["--rlimit_as", &self.limits.memory.to_string()]) // in MB
                .args(["--rlimit_nproc", &self.limits.processes.to_string()]);
        }

        // environment variables:
        for &(name, value) in self.envvars {
            cmd.arg("-E").arg(format!("{name}={value}"));
        }

        // mounts:
        for Mount { dest, typ } in self.mounts {
            match typ {
                MountType::ReadOnly { src } => {
                    let mut arg = src.clone();
                    if src != dest {
                        let arg = arg.to_mut();
                        arg.reserve_exact(1 + dest.len());
                        arg.push(OsString::from(":"));
                        arg.push(dest);
                    }
                    cmd.arg("-R").arg(arg);
                }
                MountType::ReadWrite { src } => {
                    let mut arg = src.clone();
                    if src != dest {
                        let arg = arg.to_mut();
                        arg.reserve_exact(1 + dest.len());
                        arg.push(OsString::from(":"));
                        arg.push(dest);
                    }
                    cmd.arg("-B").arg(arg);
                }
                &MountType::Temp { size } => {
                    if size > 0 {
                        let mut arg = OsString::from("none:");
                        arg.push(dest);
                        arg.push(format!(":tmpfs:size={size}M"));
                        cmd.arg("-m").arg(arg);
                    }
                }
            };
        }
        // files that are needed by some programs
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

        // pass stdin to process
        if let Some(stdin) = &self.stdin {
            let mut handle = child.stdin.take().unwrap();
            handle.write_all(stdin.as_bytes()).await?;
            drop(handle);
        }

        let stdout_reader = BufReader::new(child.stdout.take().unwrap());
        let stderr_reader = BufReader::new(child.stderr.take().unwrap());

        child.wait().await?;

        // read stdout and stderr from process
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

        // read resource usage and status
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
