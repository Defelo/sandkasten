use std::collections::HashMap;

use serde::Deserialize;

use common::*;

mod common;

async fn test_package(id: &str) {
    let EnvironmentsConfig { mut environments }: EnvironmentsConfig = config::Config::builder()
        .add_source(config::File::with_name(env!("ENVIRONMENTS_CONFIG_PATH")))
        .build()
        .unwrap()
        .try_deserialize()
        .unwrap();

    let environment = environments.remove(id).unwrap();

    let response = reqwest::Client::new()
        .post(url("/run"))
        .json(&BuildRunRequest {
            build: BuildRequest {
                environment: id.to_owned(),
                files: environment.test.files,
                compile_limits: Default::default(),
            },
            run: Default::default(),
        })
        .send()
        .await
        .unwrap();
    let status = response.status();
    if status == 200 {
        let response: BuildRunResponse = dbg!(response.json().await.unwrap());
        if environment.compile_script.is_some() {
            let build = response.build.unwrap();
            assert_eq!(build.status, 0);
            assert!(build.stdout.is_empty());
            assert!(build.stderr.is_empty());
        } else {
            assert!(response.build.is_none());
        }
        assert_eq!(response.run.status, 0);
        assert_eq!(response.run.stdout.trim(), "OK");
        assert!(response.run.stderr.is_empty());
    } else {
        dbg!(response.json::<BuildError>().await.unwrap());
        panic!("request failed");
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
