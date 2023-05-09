use sandkasten_client::{
    schemas::programs::{BuildRequest, BuildRunRequest, File},
    SandkastenClient,
};

#[tokio::test]
#[ignore]
async fn test_environments() {
    let environments = client().list_environments().await.unwrap();
    assert_eq!(environments.get("python").unwrap().name, "Python");
    assert_eq!(environments.get("rust").unwrap().name, "Rust");
}

#[tokio::test]
#[ignore]
async fn test_build_run() {
    let result = client()
        .build_and_run(&BuildRunRequest {
            build: BuildRequest {
                environment: "rust".into(),
                files: vec![File {
                    name: "test.rs".into(),
                    content: "fn main() { print!(\"Hello World!\"); }".into(),
                }],
                ..Default::default()
            },
            run: Default::default(),
        })
        .await
        .unwrap();
    assert_eq!(result.build.unwrap().status, 0);
    assert_eq!(result.run.status, 0);
    assert_eq!(result.run.stdout, "Hello World!");
    assert!(result.run.stderr.is_empty());
}

#[tokio::test]
#[ignore]
async fn test_build_then_run() {
    let result = client()
        .build(&BuildRequest {
            environment: "rust".into(),
            files: vec![File {
                name: "test.rs".into(),
                content: "fn main() { print!(\"Hello World!\"); }".into(),
            }],
            ..Default::default()
        })
        .await
        .unwrap();

    let build = result.compile_result.unwrap();
    assert_eq!(build.status, 0);
    assert!(build.stdout.is_empty());
    assert!(build.stderr.is_empty());

    let result = client()
        .run(result.program_id, &Default::default())
        .await
        .unwrap();
    assert_eq!(result.status, 0);
    assert_eq!(result.stdout, "Hello World!");
    assert!(result.stderr.is_empty());
}

fn client() -> SandkastenClient {
    SandkastenClient::new(
        option_env!("TARGET_HOST")
            .unwrap_or("http://127.0.0.1:8000")
            .parse()
            .unwrap(),
    )
}
