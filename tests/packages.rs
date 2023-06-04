#![cfg(feature = "nix")]

use sandkasten::environments;
use sandkasten_client::schemas::programs::{BuildRequest, BuildRunRequest, File, RunRequest};

use crate::common::client;

mod common;

fn test_package(id: &str) {
    let conf = sandkasten::config::load().unwrap();
    let mut environments = environments::load(&conf.environments_path).unwrap();

    let environment = environments.remove(id).unwrap();

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
            assert_eq!(response.run.stdout.trim(), "OK");
            assert!(response.run.stderr.is_empty());
        }
        Err(_) => panic!("request failed"),
    }
}

include!(env!("PACKAGES_TEST_SRC"));
