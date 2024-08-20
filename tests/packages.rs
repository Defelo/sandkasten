#![cfg(feature = "nix")]

use sandkasten::environments::{self, Environment};
use sandkasten_client::schemas::programs::{
    BuildRequest, BuildRunRequest, BuildRunResult, File, MainFile, RunRequest,
};

use crate::common::client;

mod common;

fn test_package(id: &str) {
    let environment = get_environment(id);

    match dbg!(client().build_and_run(&BuildRunRequest {
        build: BuildRequest {
            environment: id.to_owned(),
            main_file: environment.test.main_file,
            files: environment.test.files,
            ..Default::default()
        },
        run: RunRequest {
            stdin: Some("stdin".into()),
            args: ["foo", "bar", "baz"].into_iter().map(Into::into).collect(),
            files: vec![File {
                name: "test.txt".into(),
                content: "hello world".into(),
            }],
            ..Default::default()
        },
    })) {
        Ok(response) => {
            assert_ok(&response, environment.compile_script.is_some());
            assert_eq!(
                response.run.stdout.trim(),
                environment.test.expected.as_deref().unwrap_or("OK")
            );
        }
        Err(_) => panic!("request failed"),
    }
}

fn test_example(id: &str) {
    let environment = get_environment(id);
    let Some(content) = environment.example else {
        return;
    };

    match dbg!(client().build_and_run(&BuildRunRequest {
        build: BuildRequest {
            environment: id.to_owned(),
            main_file: MainFile {
                name: None,
                content,
            },
            ..Default::default()
        },
        run: RunRequest {
            stdin: Some("Foo42".into()),
            ..Default::default()
        },
    })) {
        Ok(response) => {
            assert_ok(&response, environment.compile_script.is_some());
            assert_eq!(response.run.stdout.trim(), "Hello, Foo42!");
        }
        Err(_) => panic!("request failed"),
    }
}

fn get_environment(id: &str) -> Environment {
    let conf = sandkasten::config::load().unwrap();
    let mut environments = environments::load(&conf.environments_path).unwrap();

    environments.remove(id).unwrap()
}

fn assert_ok(response: &BuildRunResult, compiled: bool) {
    if compiled {
        let build = response.build.as_ref().unwrap();
        assert_eq!(build.status, 0);
        assert!(build.stderr.is_empty());
    } else {
        assert!(response.build.is_none());
    }
    assert_eq!(response.run.status, 0);
    assert!(response.run.stderr.is_empty());
}

include!(env!("PACKAGES_TEST_SRC"));
