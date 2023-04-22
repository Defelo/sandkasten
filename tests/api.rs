use std::collections::HashMap;

use indoc::formatdoc;
use sandkasten::schemas::{
    environments::Environment,
    programs::{BuildRequest, BuildRunRequest, BuildRunResult, File, RunRequest, RunResult},
};

use crate::common::{build_and_run, url, BuildError};

mod common;

#[test]
#[ignore]
fn test_oai_spec() {
    reqwest::blocking::get(url("/openapi.json"))
        .unwrap()
        .error_for_status()
        .unwrap();
}

#[test]
#[ignore]
fn test_environments() {
    let environments: HashMap<String, Environment> = reqwest::blocking::get(url("/environments"))
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .unwrap();
    assert_eq!(environments.get("python").unwrap().name, "Python");
    assert_eq!(environments.get("rust").unwrap().name, "Rust");
}

#[test]
#[ignore]
fn test_build_run_python() {
    let response = build_and_run(&BuildRunRequest {
        build: BuildRequest {
            environment: "python".into(),
            files: vec![
                File {
                    name: "test.py".into(),
                    content: formatdoc! {"
                        from foo import add, mul
                        import sys
                        import time
                        print(add(6, 7))
                        print(mul(6, 7), file=sys.stderr)
                        time.sleep(0.456)
                        exit(42)
                    "},
                },
                File {
                    name: "foo.py".into(),
                    content: formatdoc! {"
                        def add(a, b):
                          return a + b
                        def mul(a, b):
                          return a * b
                    "},
                },
            ],
            compile_limits: Default::default(),
        },
        run: Default::default(),
    })
    .unwrap();
    assert!(response.build.is_none());
    assert_eq!(response.run.status, 42);
    assert_eq!(response.run.stdout, "13\n");
    assert_eq!(response.run.stderr, "42\n");
    assert!(response.run.resource_usage.time >= 456 && response.run.resource_usage.time <= 2000);
    assert!(
        response.run.resource_usage.memory >= 1000 && response.run.resource_usage.memory <= 20000
    );
}

#[test]
#[ignore]
fn test_build_run_rust_compilation_error() {
    let BuildError::CompileError(response) = build_and_run(&BuildRunRequest {
        build: BuildRequest {
            environment: "rust".into(),
            files: vec![File {
                name: "test.rs".into(),
                content: "fn main() { fn_not_found(); }".into(),
            }],
            compile_limits: Default::default(),
        },
        run: Default::default(),
    })
    .unwrap_err();
    assert_eq!(response.status, 1);
    assert!(response.stdout.is_empty());
    assert!(!response.stderr.is_empty());
}

#[test]
#[ignore]
fn test_build_run_rust_ok() {
    let response = build_and_run(&BuildRunRequest {
        build: BuildRequest {
            environment: "rust".into(),
            files: vec![
                File {
                    name: "test.rs".into(),
                    content: formatdoc! {r#"
                        mod foo;
                        fn main() {{
                            let test = ();
                            println!("foo bar");
                            foo::asdf();
                        }}
                    "#},
                },
                File {
                    name: "foo.rs".into(),
                    content: formatdoc! {r#"
                        pub fn asdf() {{
                            eprintln!("test {{}}", 7 * 191);
                        }}
                    "#},
                },
            ],
            compile_limits: Default::default(),
        },
        run: Default::default(),
    })
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

#[test]
#[ignore]
fn test_build_cached() {
    let request = BuildRunRequest {
        build: BuildRequest {
            environment: "rust".into(),
            files: vec![File {
                name: "test.rs".into(),
                content: "fn main() { println!(\"test\"); }".into(),
            }],
            compile_limits: Default::default(),
        },
        run: Default::default(),
    };

    let BuildRunResult {
        program_id,
        ttl: _,
        build,
        run,
    }: BuildRunResult = build_and_run(&request).unwrap();
    let build = build.unwrap();
    assert_eq!(run.status, 0);
    assert_eq!(run.stdout, "test\n");

    let response: BuildRunResult = build_and_run(&request).unwrap();
    assert_eq!(response.program_id, program_id);
    assert_eq!(response.build.unwrap(), build);
    assert_eq!(response.run.status, 0);
    assert_eq!(response.run.stdout, "test\n");

    let response: RunResult = reqwest::blocking::Client::new()
        .post(url(format!("/programs/{program_id}/run")))
        .json(&RunRequest::default())
        .send()
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .unwrap();
    assert_eq!(response.status, 0);
    assert_eq!(response.stdout, "test\n");
}
