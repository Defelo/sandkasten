use indoc::formatdoc;
use sandkasten_client::schemas::{
    programs::{
        BuildRequest, BuildRunError, BuildRunRequest, BuildRunResult, EnvVar, File, RunRequest,
        RunResult,
    },
    ErrorResponse,
};

use crate::common::client;

mod common;

#[test]
#[ignore]
fn test_environments() {
    let environments = client().list_environments().unwrap();
    assert_eq!(environments.get("python").unwrap().name, "Python");
    assert_eq!(environments.get("rust").unwrap().name, "Rust");
}

#[test]
#[ignore]
fn test_build_run_python() {
    let response = client()
        .build_and_run(&BuildRunRequest {
            build: BuildRequest {
                environment: "python".into(),
                files: vec![
                    File {
                        name: "test.py".into(),
                        content: formatdoc! {"
                            from foo import add, mul
                            import sys
                            import time
                            import os
                            print(add(6, 7))
                            print(mul(6, 7), file=sys.stderr)
                            print(os.environ['FOO'], os.environ['BAR'])
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
                env_vars: vec![],
                compile_limits: Default::default(),
            },
            run: RunRequest {
                env_vars: vec![
                    EnvVar {
                        name: "FOO".into(),
                        value: "hello".into(),
                    },
                    EnvVar {
                        name: "BAR".into(),
                        value: "world".into(),
                    },
                ],
                ..Default::default()
            },
        })
        .unwrap();
    assert!(response.build.is_none());
    assert_eq!(response.run.status, 42);
    assert_eq!(response.run.stdout, "13\nhello world\n");
    assert_eq!(response.run.stderr, "42\n");
    assert!(response.run.resource_usage.time >= 456 && response.run.resource_usage.time <= 2000);
    assert!(
        response.run.resource_usage.memory >= 1000 && response.run.resource_usage.memory <= 20000
    );
}

#[test]
#[ignore]
fn test_build_run_rust_compilation_error() {
    match client()
        .build_and_run(&BuildRunRequest {
            build: BuildRequest {
                environment: "rust".into(),
                files: vec![File {
                    name: "test.rs".into(),
                    content: "fn main() { fn_not_found(); }".into(),
                }],
                env_vars: vec![],
                compile_limits: Default::default(),
            },
            run: Default::default(),
        })
        .unwrap_err()
    {
        sandkasten_client::Error::ErrorResponse(err) => match *err {
            ErrorResponse::Inner(BuildRunError::CompileError(response)) => {
                assert_eq!(response.status, 1);
                assert!(response.stdout.is_empty());
                assert!(!response.stderr.is_empty());
            }
            _ => panic!(),
        },
        _ => panic!(),
    }
}

#[test]
#[ignore]
fn test_build_run_rust_ok() {
    let response = client()
        .build_and_run(&BuildRunRequest {
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
                                println!(env!("BUILD_VAR"));
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
                env_vars: vec![EnvVar {
                    name: "BUILD_VAR".into(),
                    value: "test123".into(),
                }],
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
    assert_eq!(response.run.stdout, "foo bar\ntest123\n");
    assert_eq!(response.run.stderr, "test 1337\n");
    assert!(response.run.resource_usage.time <= 100);
    assert!(
        response.run.resource_usage.memory >= 100 && response.run.resource_usage.memory <= 10000
    );
}

#[test]
#[ignore]
fn test_build_cached() {
    let client = client();
    let request = BuildRunRequest {
        build: BuildRequest {
            environment: "rust".into(),
            files: vec![File {
                name: "test.rs".into(),
                content: "fn main() { println!(\"test\"); }".into(),
            }],
            env_vars: vec![],
            compile_limits: Default::default(),
        },
        run: Default::default(),
    };

    let BuildRunResult {
        program_id,
        ttl: _,
        build,
        run,
    }: BuildRunResult = client.build_and_run(&request).unwrap();
    let build = build.unwrap();
    assert_eq!(run.status, 0);
    assert_eq!(run.stdout, "test\n");

    let response: BuildRunResult = client.build_and_run(&request).unwrap();
    assert_eq!(response.program_id, program_id);
    assert_eq!(response.build.unwrap(), build);
    assert_eq!(response.run.status, 0);
    assert_eq!(response.run.stdout, "test\n");

    let response: RunResult = client.run(program_id, &RunRequest::default()).unwrap();
    assert_eq!(response.status, 0);
    assert_eq!(response.stdout, "test\n");
}

#[test]
#[ignore]
fn test_build_then_run() {
    let client = client();
    let build = client
        .build(&BuildRequest {
            environment: "rust".into(),
            files: vec![File {
                name: "test.rs".into(),
                content: "fn main() { println!(\"hello world\"); }".into(),
            }],
            env_vars: vec![],
            compile_limits: Default::default(),
        })
        .unwrap();
    assert_eq!(build.compile_result.unwrap().status, 0);
    let run = client.run(build.program_id, &Default::default()).unwrap();
    assert_eq!(run.status, 0);
    assert_eq!(run.stdout, "hello world\n");
}
