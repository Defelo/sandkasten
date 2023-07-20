#![cfg(feature = "nix")]

use sandkasten::environments::{self, Environment};
use sandkasten_client::schemas::programs::{
    BuildRequest, BuildRunRequest, File, MainFile, RunRequest,
};

use crate::common::client;

mod common;

fn test_package(id: &str) {
    let environment = get_environment(id);

    match client().build_and_run(&BuildRunRequest {
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
    }) {
        Ok(response) => {
            if environment.compile_script.is_some() {
                let build = response.build.unwrap();
                assert_eq!(build.status, 0);
                assert!(build.stderr.is_empty());
            } else {
                assert!(response.build.is_none());
            }
            assert_eq!(response.run.status, 0);
            assert_eq!(
                response.run.stdout.trim(),
                environment.test.expected.as_deref().unwrap_or("OK")
            );
            assert!(response.run.stderr.is_empty());
        }
        Err(_) => panic!("request failed"),
    }
}

fn test_example(id: &str) {
    let environment = get_environment(id);
    let Some(content) = environment.example else {
        return;
    };

    match client().build_and_run(&BuildRunRequest {
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
    }) {
        Ok(response) => {
            assert_eq!(response.run.status, 0);
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

include!(env!("PACKAGES_TEST_SRC"));
