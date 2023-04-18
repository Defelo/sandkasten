#![cfg(feature = "nix")]

use std::collections::HashMap;

use sandkasten::schemas::programs::{BuildRequest, BuildRunRequest, File, RunRequest};
use serde::Deserialize;

use crate::common::build_and_run;

mod common;

fn test_package(id: &str) {
    let EnvironmentsConfig { mut environments }: EnvironmentsConfig = config::Config::builder()
        .add_source(config::File::with_name(env!("ENVIRONMENTS_CONFIG_PATH")))
        .build()
        .unwrap()
        .try_deserialize()
        .unwrap();

    let environment = environments.remove(id).unwrap();

    match build_and_run(&BuildRunRequest {
        build: BuildRequest {
            environment: id.to_owned(),
            files: environment.test.files,
            compile_limits: Default::default(),
        },
        run: RunRequest {
            stdin: Some("stdin".into()),
            args: ["foo", "bar", "baz"].into_iter().map(Into::into).collect(),
            files: vec![File {
                name: "test.txt".into(),
                content: "hello world".into(),
            }],
            run_limits: Default::default(),
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
            assert_eq!(response.run.stdout.trim(), "OK");
            assert!(response.run.stderr.is_empty());
        }
        Err(_) => panic!("request failed"),
    }
}

include!(env!("PACKAGES_TEST_SRC"));

#[derive(Debug, Deserialize)]
struct EnvironmentsConfig {
    environments: HashMap<String, Environment>,
}

#[derive(Debug, Deserialize)]
struct Environment {
    compile_script: Option<String>,
    test: Test,
}

#[derive(Debug, Deserialize)]
struct Test {
    files: Vec<File>,
}
