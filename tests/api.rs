use std::sync::Arc;

use indoc::formatdoc;
use regex::Regex;
use sandkasten_client::{
    schemas::{
        programs::{
            BuildError, BuildRequest, BuildRunError, BuildRunRequest, BuildRunResult, EnvVar, File,
            LimitsOpt, MainFile, RunError, RunRequest, RunResult,
        },
        ErrorResponse,
    },
    Error,
};

use crate::common::client;

mod common;

#[test]
#[ignore]
fn test_version() {
    let version = client().version().unwrap();
    assert_eq!(version, env!("CARGO_PKG_VERSION"));
}

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
                main_file: MainFile {
                    name: Some("test.py".into()),
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
                files: vec![File {
                    name: "foo.py".into(),
                    content: formatdoc! {"
                            def add(a, b):
                              return a + b
                            def mul(a, b):
                              return a * b
                        "},
                }],
                ..Default::default()
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
                main_file: MainFile {
                    name: Some("test.rs".into()),
                    content: "fn main() { fn_not_found(); }".into(),
                },
                ..Default::default()
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
                main_file: MainFile {
                    name: Some("test.rs".into()),
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
                files: vec![File {
                    name: "foo.rs".into(),
                    content: formatdoc! {r#"
                        pub fn asdf() {{
                            eprintln!("test {{}}", 7 * 191);
                        }}
                    "#},
                }],
                env_vars: vec![EnvVar {
                    name: "BUILD_VAR".into(),
                    value: "test123".into(),
                }],
                ..Default::default()
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
            main_file: MainFile {
                name: Some("test.rs".into()),
                content: "fn main() { println!(\"test\"); }".into(),
            },
            ..Default::default()
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
            main_file: MainFile {
                name: Some("test.rs".into()),
                content: "fn main() { println!(\"hello world\"); }".into(),
            },
            ..Default::default()
        })
        .unwrap();
    assert_eq!(build.compile_result.unwrap().status, 0);
    let run = client.run(build.program_id, &Default::default()).unwrap();
    assert_eq!(run.status, 0);
    assert_eq!(run.stdout, "hello world\n");
}

#[test]
#[ignore]
fn test_build_run_errors() {
    let client = client();

    let Error::ErrorResponse(err) = client
        .build_and_run(&BuildRunRequest {
            build: BuildRequest {
                environment: "this_environment_does_not_exist".into(),
                main_file: MainFile {
                    name: Some("test".into()),
                    content: "".into(),
                },
                ..Default::default()
            },
            run: Default::default(),
        })
        .unwrap_err() else { panic!() };
    assert!(matches!(
        *err,
        ErrorResponse::Inner(BuildRunError::EnvironmentNotFound)
    ));

    let Error::ErrorResponse(err) = client
        .build_and_run(&BuildRunRequest {
            build: BuildRequest {
                environment: "python".into(),
                main_file: MainFile {
                    name: Some(".".into()),
                    content: "".into(),
                },
                ..Default::default()
            },
            run: Default::default(),
        })
        .unwrap_err() else { panic!() };
    assert!(matches!(
        *err,
        ErrorResponse::Inner(BuildRunError::InvalidFileNames)
    ));

    let Error::ErrorResponse(err) = client
        .build_and_run(&BuildRunRequest {
            build: BuildRequest {
                environment: "python".into(),
                files: vec![File {
                    name: "test.py".into(),
                    content: "".into(),
                }],
                env_vars: vec![EnvVar {name: "_".into(), value: "".into()}],
                ..Default::default()
            },
            run: Default::default(),
        })
        .unwrap_err() else { panic!() };
    assert!(matches!(
        *err,
        ErrorResponse::Inner(BuildRunError::InvalidEnvVars)
    ));

    let Error::ErrorResponse(err) = client
        .build_and_run(&BuildRunRequest {
            build: BuildRequest {
                environment: "rust".into(),
                main_file: MainFile {
                    name: Some("test.rs".into()),
                    content: "fn main() {}".into(),
                },
                env_vars: vec![EnvVar {
                    name: "x".into(),
                    value: uuid::Uuid::new_v4().to_string(),
                }],
                compile_limits: LimitsOpt {cpus: Some(4096), ..Default::default()},
                ..Default::default()
            },
            run: Default::default(),
        })
        .unwrap_err() else { panic!() };
    let ErrorResponse::Inner(BuildRunError::CompileLimitsExceeded(mut les)) = *err else {panic!()};
    let le = les.pop().unwrap();
    assert_eq!(le.name, "cpus");
    assert_eq!(le.max_value, 1);
    assert!(les.pop().is_none());

    let Error::ErrorResponse(err) = client
        .build_and_run(&BuildRunRequest {
            build: BuildRequest {
                environment: "rust".into(),
                main_file: MainFile {
                    name: Some("test.rs".into()),
                    content: "fn main() {}".into(),
                },
                ..Default::default()
            },
            run: RunRequest {
                run_limits: LimitsOpt {
                    time: Some(65536), ..Default::default()
                },
                ..Default::default()
            },
        })
        .unwrap_err() else { panic!() };
    let ErrorResponse::Inner(BuildRunError::RunLimitsExceeded(mut les)) = *err else {panic!()};
    let le = les.pop().unwrap();
    assert_eq!(le.name, "time");
    assert_eq!(le.max_value, 5);
    assert!(les.pop().is_none());
}

#[test]
#[ignore]
fn test_build_errors() {
    let client = client();

    let Error::ErrorResponse(err) = client
        .build(&BuildRequest {
            environment: "this_environment_does_not_exist".into(),
            files: vec![File {
                name: "test".into(),
                content: "".into(),
            }],
            ..Default::default()
        })
        .unwrap_err() else { panic!() };
    assert!(matches!(
        *err,
        ErrorResponse::Inner(BuildError::EnvironmentNotFound)
    ));

    let Error::ErrorResponse(err) = client
        .build(&BuildRequest {
            environment: "rust".into(),
            files: vec![File {
                name: "test.rs".into(),
                content: "".into(),
            }],
            ..Default::default()
        })
        .unwrap_err() else { panic!() };
    assert!(matches!(
        *err,
        ErrorResponse::Inner(BuildError::CompileError(_))
    ));

    let Error::ErrorResponse(err) = client
        .build(&BuildRequest {
            environment: "python".into(),
            files: vec![File {
                name: ".".into(),
                content: "".into(),
            }],
            ..Default::default()
        })
        .unwrap_err() else { panic!() };
    assert!(matches!(
        *err,
        ErrorResponse::Inner(BuildError::InvalidFileNames)
    ));

    let Error::ErrorResponse(err) = client
        .build(&BuildRequest {
            environment: "python".into(),
            files: vec![File {
                name: "test.py".into(),
                content: "".into(),
            }],
            env_vars: vec![EnvVar {
                name: "_".into(),
                value: "".into(),
            }],
            ..Default::default()
        })
        .unwrap_err() else { panic!() };
    assert!(matches!(
        *err,
        ErrorResponse::Inner(BuildError::InvalidEnvVars)
    ));

    let Error::ErrorResponse(err) = client
        .build(&BuildRequest {
            environment: "rust".into(),
            main_file: MainFile {
                name: Some("test.rs".into()),
                content: "fn main() {}".into(),
            },
            env_vars: vec![EnvVar {
                name: "x".into(),
                value: uuid::Uuid::new_v4().to_string(),
            }],
            compile_limits: LimitsOpt {
                cpus: Some(4096),
                ..Default::default()
            },
            ..Default::default()
        })
        .unwrap_err() else { panic!() };
    let ErrorResponse::Inner(BuildError::CompileLimitsExceeded(mut les)) = *err else {panic!()};
    let le = les.pop().unwrap();
    assert_eq!(le.name, "cpus");
    assert_eq!(le.max_value, 1);
    assert!(les.pop().is_none());
}

#[test]
#[ignore]
fn test_run_errors() {
    let client = client();

    let program_id = client
        .build(&BuildRequest {
            environment: "python".into(),
            files: vec![File {
                name: "test.py".into(),
                content: "print('Hello World')".into(),
            }],
            ..Default::default()
        })
        .unwrap()
        .program_id;

    let Error::ErrorResponse(err) = client
        .run("00000000-0000-0000-0000-000000000000", &Default::default())
        .unwrap_err() else { panic!() };
    assert!(matches!(
        *err,
        ErrorResponse::Inner(RunError::ProgramNotFound)
    ));

    let Error::ErrorResponse(err) = client
        .run(program_id, &RunRequest {
            files: vec![File {
                name: ".".into(),
                content: "".into(),
            }],
            ..Default::default()
        })
        .unwrap_err() else { panic!() };
    assert!(matches!(
        *err,
        ErrorResponse::Inner(RunError::InvalidFileNames)
    ));

    let Error::ErrorResponse(err) = client
        .run(program_id, &RunRequest {
            env_vars: vec![EnvVar {
                name: "_".into(),
                value: "".into(),
            }],
            ..Default::default()
        })
        .unwrap_err() else { panic!() };
    assert!(matches!(
        *err,
        ErrorResponse::Inner(RunError::InvalidEnvVars)
    ));

    let Error::ErrorResponse(err) = client
        .run(program_id, &RunRequest {
            run_limits: LimitsOpt {
                cpus: Some(4096),
                ..Default::default()
            },
            ..Default::default()
        })
        .unwrap_err() else { panic!() };
    let ErrorResponse::Inner(RunError::RunLimitsExceeded(mut les)) = *err else {panic!()};
    let le = les.pop().unwrap();
    assert_eq!(le.name, "cpus");
    assert_eq!(le.max_value, 1);
    assert!(les.pop().is_none());
}

#[test]
#[ignore]
fn test_network() {
    let result = client()
        .build_and_run(&BuildRunRequest {
            build: BuildRequest {
                environment: "python".into(),
                main_file: MainFile {
                    name: Some("test.py".into()),
                    content: formatdoc! {r#"
                        from http.client import *
                        c=HTTPConnection("ip6.me")
                        c.request("GET", "http://ip6.me/api/")
                        r=c.getresponse()
                        print(r.status, r.read().decode().strip(), end='')
                    "#},
                },
                ..Default::default()
            },
            run: Default::default(),
        })
        .unwrap();
    assert_eq!(result.run.status, 0);
    let re = Regex::new(r"^200 IPv[46],[^,]+,.+$").unwrap();
    assert!(re.is_match(&result.run.stdout));
    assert!(result.run.stderr.is_empty());
}

#[test]
#[ignore]
fn test_build_race() {
    let client = Arc::new(client());

    for _ in 0..16 {
        let x = uuid::Uuid::new_v4();
        let threads = (0..256)
            .map(|i| {
                let client = Arc::clone(&client);
                std::thread::spawn(move || {
                    client.build(&BuildRequest {
                        environment: "rust".into(),
                        main_file: MainFile {
                            name: Some("test.rs".into()),
                            content: "fn main() { println!(\"hi there\"); }".into(),
                        },
                        env_vars: vec![EnvVar {
                            name: "x".into(),
                            value: format!("{x} {}", i / 64),
                        }],
                        ..Default::default()
                    })
                })
            })
            .collect::<Vec<_>>();
        let results = threads
            .into_iter()
            .map(|t| t.join().unwrap().unwrap())
            .collect::<Vec<_>>();
        let res = results.first().unwrap();
        assert!(results
            .iter()
            .take(64)
            .all(|r| r.program_id == res.program_id));
        assert!(results
            .iter()
            .all(|r| r.compile_result.as_ref().unwrap().status == 0));
        assert!(results
            .iter()
            .all(|r| r.compile_result.as_ref().unwrap().stdout.is_empty()));
        assert!(results
            .iter()
            .all(|r| r.compile_result.as_ref().unwrap().stderr.is_empty()));

        let run = client.run(res.program_id, &Default::default()).unwrap();
        assert_eq!(run.status, 0);
        assert_eq!(run.stdout, "hi there\n");
        assert!(run.stderr.is_empty());
    }
}
