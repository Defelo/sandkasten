use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[tokio::test]
#[ignore]
async fn test_oai_spec() {
    reqwest::get(url("/openapi.json"))
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

#[tokio::test]
#[ignore]
async fn test_environments() {
    let environments: HashMap<String, Environment> = reqwest::get(url("/environments"))
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(environments.get("python").unwrap().name, "Python");
    assert_eq!(environments.get("rust").unwrap().name, "Rust");
}

#[tokio::test]
#[ignore]
async fn test_build_run_python() {
    let response: BuildRunResponse = reqwest::Client::new()
        .post(url("/run"))
        .json(&BuildRunRequest {
            build: BuildRequest {
                environment: "python".into(),
                files: vec![
                    File {
                        name: "test.py".into(),
                        content: "from foo import add, mul\nimport sys\nimport time\nprint(add(6, 7))\nprint(mul(6, 7), file=sys.stderr)\ntime.sleep(0.456)\nexit(42)".into(),
                    },
                    File {
                        name: "foo.py".into(),
                        content: "def add(a, b):\n  return a + b\ndef mul(a, b):\n  return a * b".into(),
                    },
                ],
                compile_limits: LimitsOpt::default(),
            },
            run: RunRequest {
                stdin: None,
                args: Vec::new(),
                files: Vec::new(),
                run_limits: LimitsOpt::default(),
            },
        })
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(response.build.is_none());
    assert_eq!(response.run.status, 42);
    assert_eq!(response.run.stdout, "13\n");
    assert_eq!(response.run.stderr, "42\n");
    assert!(response.run.resource_usage.time >= 456 && response.run.resource_usage.time <= 800);
    assert!(
        response.run.resource_usage.memory >= 1000 && response.run.resource_usage.memory <= 20000
    );
}

#[tokio::test]
#[ignore]
async fn test_build_run_rust_compilation_error() {
    let response = reqwest::Client::new()
        .post(url("/run"))
        .json(&BuildRunRequest {
            build: BuildRequest {
                environment: "rust".into(),
                files: vec![File {
                    name: "test.rs".into(),
                    content: "fn main() { fn_not_found(); }".into(),
                }],
                compile_limits: LimitsOpt::default(),
            },
            run: RunRequest {
                stdin: None,
                args: Vec::new(),
                files: Vec::new(),
                run_limits: LimitsOpt::default(),
            },
        })
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 400);
    let BuildError::CompileError(response) = response.json().await.unwrap();
    assert_eq!(response.status, 1);
    assert!(response.stdout.is_empty());
    assert!(!response.stderr.is_empty());
}

#[tokio::test]
#[ignore]
async fn test_build_run_rust_ok() {
    let response: BuildRunResponse = reqwest::Client::new()
        .post(url("/run"))
        .json(&BuildRunRequest {
            build: BuildRequest {
                environment: "rust".into(),
                files: vec![
                File {
                    name: "test.rs".into(),
                    content: "mod foo; fn main() { let test = (); println!(\"foo bar\"); foo::asdf(); }".into(),
                }, File {
                    name: "foo.rs".into(),
                    content: "pub fn asdf() { eprintln!(\"test {}\", 7 * 191); }".into()
                }],
                compile_limits: LimitsOpt::default(),
            },
            run: RunRequest {
                stdin: None,
                args: Vec::new(),
                files: Vec::new(),
                run_limits: LimitsOpt::default(),
            },
        })
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    let build = response.build.unwrap();
    assert_eq!(build.status, 0);
    assert!(build.stdout.is_empty());
    assert!(!build.stderr.is_empty());
    assert!(build.resource_usage.time >= 1 && build.resource_usage.time <= 2000);
    assert!(build.resource_usage.memory >= 100 && response.run.resource_usage.memory <= 10000);
    assert_eq!(response.run.status, 0);
    assert_eq!(response.run.stdout, "foo bar\n");
    assert_eq!(response.run.stderr, "test 1337\n");
    assert!(response.run.resource_usage.time <= 100);
    assert!(
        response.run.resource_usage.memory >= 100 && response.run.resource_usage.memory <= 10000
    );
}

#[tokio::test]
#[ignore]
async fn test_build_cached() {
    let request = BuildRunRequest {
        build: BuildRequest {
            environment: "rust".into(),
            files: vec![File {
                name: "test.rs".into(),
                content: "fn main() { println!(\"test\"); }".into(),
            }],
            compile_limits: LimitsOpt::default(),
        },
        run: RunRequest {
            stdin: None,
            args: Vec::new(),
            files: Vec::new(),
            run_limits: LimitsOpt::default(),
        },
    };

    let BuildRunResponse {
        program_id,
        build,
        run,
    }: BuildRunResponse = reqwest::Client::new()
        .post(url("/run"))
        .json(&request)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    let build = build.unwrap();
    assert_eq!(run.status, 0);
    assert_eq!(run.stdout, "test\n");

    let response: BuildRunResponse = reqwest::Client::new()
        .post(url("/run"))
        .json(&request)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(response.program_id, program_id);
    assert_eq!(response.build.unwrap(), build);
    assert_eq!(response.run.status, 0);
    assert_eq!(response.run.stdout, "test\n");

    let response: RunResponse = reqwest::Client::new()
        .post(url(format!("/programs/{program_id}/run")))
        .json(&RunRequest::default())
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(response.status, 0);
    assert_eq!(response.stdout, "test\n");
}

#[tokio::test]
#[ignore]
async fn test_no_internet() {
    let response: BuildRunResponse = reqwest::Client::new()
        .post(url("/run"))
        .json(&BuildRunRequest {
            build: BuildRequest {
                environment: "python".into(),
                files: vec![File {
                    name: "test.py".into(),
                    content: "from http.client import *; c=HTTPConnection('1.1.1.1'); c.request('GET', 'http://1.1.1.1')".into(),
                }],
                compile_limits: Default::default(),
            },
            run: Default::default(),
        })
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(response.run.status, 1);
    assert_eq!(
        response.run.stderr.trim().lines().last().unwrap(),
        "OSError: [Errno 101] Network is unreachable"
    );
}

#[tokio::test]
#[ignore]
async fn test_forkbomb() {
    let response: BuildRunResponse = reqwest::Client::new()
        .post(url("/run"))
        .json(&BuildRunRequest {
            build: BuildRequest {
                environment: "python".into(),
                files: vec![File {
                    name: "test.py".into(),
                    content: "import os\nwhile True: os.fork()".into(),
                }],
                compile_limits: Default::default(),
            },
            run: Default::default(),
        })
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(response.run.status, 1);
    assert_eq!(
        response.run.stderr.trim().lines().last().unwrap(),
        "BlockingIOError: [Errno 11] Resource temporarily unavailable"
    );
    assert!(response.run.resource_usage.time < 1000);
}

#[tokio::test]
#[ignore]
async fn test_stdoutbomb() {
    let response: BuildRunResponse = reqwest::Client::new()
        .post(url("/run"))
        .json(&BuildRunRequest {
            build: BuildRequest {
                environment: "rust".into(),
                files: vec![File {
                    name: "test.rs".into(),
                    content: "fn main() { loop { println!(\"spam\"); eprintln!(\"maps\"); } }"
                        .into(),
                }],
                compile_limits: Default::default(),
            },
            run: RunRequest {
                run_limits: LimitsOpt {
                    time: Some(1),
                    stdout_max_size: Some(2048),
                    stderr_max_size: Some(1024),
                    ..Default::default()
                },
                ..Default::default()
            },
        })
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(response.run.status, 137);
    assert_eq!(response.run.stdout.len(), 2048);
    assert_eq!(response.run.stderr.len(), 1024);
}

#[derive(Debug, Deserialize)]
struct Environment {
    name: String,
    #[allow(dead_code)]
    version: String,
}

#[derive(Debug, Serialize)]
struct BuildRunRequest {
    build: BuildRequest,
    run: RunRequest,
}

#[derive(Debug, Deserialize)]
struct BuildRunResponse {
    program_id: String,
    build: Option<RunResponse>,
    run: RunResponse,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct RunResponse {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
    pub resource_usage: ResourceUsage,
    pub limits: Limits,
}

#[derive(Debug, Serialize)]
struct BuildRequest {
    environment: String,
    files: Vec<File>,
    compile_limits: LimitsOpt,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "error", content = "details", rename_all = "snake_case")]
enum BuildError {
    CompileError(RunResponse),
}

#[derive(Debug, Serialize, Default)]
struct RunRequest {
    stdin: Option<String>,
    args: Vec<String>,
    files: Vec<File>,
    run_limits: LimitsOpt,
}

#[derive(Debug, Serialize)]
struct File {
    name: String,
    content: String,
}

#[derive(Debug, Serialize, Default)]
struct LimitsOpt {
    cpus: Option<u64>,
    file_descriptors: Option<u64>,
    filesize: Option<u64>,
    memory: Option<u64>,
    processes: Option<u64>,
    time: Option<u64>,
    stdout_max_size: Option<u64>,
    stderr_max_size: Option<u64>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct Limits {
    cpus: u64,
    file_descriptors: u64,
    filesize: u64,
    memory: u64,
    processes: u64,
    time: u64,
    stdout_max_size: u64,
    stderr_max_size: u64,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct ResourceUsage {
    pub time: u64,
    pub memory: u64,
}

fn url(path: impl std::fmt::Display) -> String {
    format!("http://127.0.0.1:8000{path}")
}
