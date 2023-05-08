use indoc::formatdoc;
use sandkasten_client::schemas::programs::{
    BuildRequest, BuildRunRequest, File, LimitsOpt, RunRequest,
};

use crate::common::client;

mod common;

#[test]
#[ignore]
fn test_no_internet() {
    let response = client()
        .build_and_run(&BuildRunRequest {
            build: BuildRequest {
                environment: "python".into(),
                files: vec![File {
                    name: "test.py".into(),
                    content: formatdoc! {r#"
                        from http.client import *
                        c=HTTPConnection("1.1.1.1")
                        c.request("GET", "http://1.1.1.1")
                    "#},
                }],
                env_vars: vec![],
                compile_limits: Default::default(),
            },
            run: Default::default(),
        })
        .unwrap();
    assert_eq!(response.run.status, 1);
    assert_eq!(
        response.run.stderr.trim().lines().last().unwrap(),
        "OSError: [Errno 101] Network is unreachable"
    );
}

#[test]
#[ignore]
fn test_forkbomb() {
    let response = client()
        .build_and_run(&BuildRunRequest {
            build: BuildRequest {
                environment: "python".into(),
                files: vec![File {
                    name: "test.py".into(),
                    content: formatdoc! {"
                        import os
                        while True:
                            os.fork()
                    "},
                }],
                env_vars: vec![],
                compile_limits: Default::default(),
            },
            run: Default::default(),
        })
        .unwrap();
    assert_eq!(response.run.status, 1);
    assert_eq!(
        response.run.stderr.trim().lines().last().unwrap(),
        "BlockingIOError: [Errno 11] Resource temporarily unavailable"
    );
    assert!(response.run.resource_usage.time < 1000);
}

#[test]
#[ignore]
fn test_stdoutbomb() {
    let response = client()
        .build_and_run(&BuildRunRequest {
            build: BuildRequest {
                environment: "rust".into(),
                files: vec![File {
                    name: "test.rs".into(),
                    content: formatdoc! {r#"
                        fn main() {{
                            loop {{
                                println!("spam");
                                eprintln!("maps");
                            }}
                        }}
                    "#},
                }],
                env_vars: vec![],
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
        .unwrap();
    assert_eq!(response.run.status, 137);
    assert_eq!(response.run.stdout.len(), 2048);
    assert_eq!(response.run.stderr.len(), 1024);
}

#[test]
#[ignore]
fn test_flood_memory() {
    let response = client()
        .build_and_run(&BuildRunRequest {
            build: BuildRequest {
                environment: "python".into(),
                files: vec![File {
                    name: "test.py".into(),
                    content: formatdoc! {r#"
                        x = [1]
                        while True:
                            x += x
                    "#},
                }],
                env_vars: vec![],
                compile_limits: Default::default(),
            },
            run: RunRequest {
                run_limits: LimitsOpt {
                    memory: Some(1024),
                    ..Default::default()
                },
                ..Default::default()
            },
        })
        .unwrap();
    assert_ne!(response.run.status, 0);
    assert_eq!(
        response.run.stderr.trim().lines().last().unwrap(),
        "MemoryError"
    );
}

#[test]
#[ignore]
fn test_combination() {
    let response = client()
        .build_and_run(&BuildRunRequest {
            build: BuildRequest {
                environment: "python".into(),
                files: vec![File {
                    name: "test.py".into(),
                    content: formatdoc! {r#"
                        import os
                        for _ in range(10):
                            os.fork()
                        x = [1]
                        while True:
                            x += x
                    "#},
                }],
                env_vars: vec![],
                compile_limits: Default::default(),
            },
            run: RunRequest {
                run_limits: LimitsOpt {
                    memory: Some(1024),
                    processes: Some(16),
                    ..Default::default()
                },
                ..Default::default()
            },
        })
        .unwrap();
    assert_ne!(response.run.status, 0);
    let stderr = response.run.stderr;
    assert!(stderr.contains("MemoryError") || stderr.contains("Resource temporarily unavailable"));
}

#[test]
#[ignore]
fn test_large_file() {
    let response = client()
        .build_and_run(&BuildRunRequest {
            build: BuildRequest {
                environment: "bash".into(),
                files: vec![File {
                    name: "test.sh".into(),
                    content: formatdoc! {r#"
                        dd if=/dev/urandom of=/tmp/test
                    "#},
                }],
                env_vars: vec![],
                compile_limits: Default::default(),
            },
            run: Default::default(),
        })
        .unwrap();
    assert_eq!(response.run.status, 153);
    assert!(response.run.stderr.contains("File size limit exceeded"));
}

#[test]
#[ignore]
fn test_many_files() {
    let response = client()
        .build_and_run(&BuildRunRequest {
            build: BuildRequest {
                environment: "bash".into(),
                files: vec![File {
                    name: "test.sh".into(),
                    content: formatdoc! {r#"
                        cd /tmp
                        i=0
                        while true; do
                            dd if=/dev/urandom of=f$i bs=1M count=16 status=none
                            i=$((i+1))
                        done
                    "#},
                }],
                env_vars: vec![],
                compile_limits: Default::default(),
            },
            run: Default::default(),
        })
        .unwrap();
    dbg!(&response);
    assert_eq!(response.run.status, 137);
    assert!(response.run.stderr.contains("No space left on device"));
}
