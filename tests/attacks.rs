use indoc::formatdoc;
use sandkasten::schemas::programs::{BuildRequest, BuildRunRequest, File, LimitsOpt, RunRequest};

use crate::common::build_and_run;

mod common;

#[test]
#[ignore]
fn test_no_internet() {
    let response = build_and_run(&BuildRunRequest {
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
    let response = build_and_run(&BuildRunRequest {
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
    let response = build_and_run(&BuildRunRequest {
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
