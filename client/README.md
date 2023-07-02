[![check](https://github.com/Defelo/sandkasten/actions/workflows/check.yml/badge.svg)](https://github.com/Defelo/sandkasten/actions/workflows/check.yml)
[![test](https://github.com/Defelo/sandkasten/actions/workflows/test.yml/badge.svg)](https://github.com/Defelo/sandkasten/actions/workflows/test.yml)
[![codecov](https://codecov.io/gh/Defelo/sandkasten/branch/develop/graph/badge.svg?token=Y5CE2887KO)](https://codecov.io/gh/Defelo/sandkasten)
![Version](https://img.shields.io/github/v/tag/Defelo/sandkasten?include_prereleases&label=version)
[![dependency status](https://deps.rs/repo/github/Defelo/sandkasten/status.svg)](https://deps.rs/repo/github/Defelo/sandkasten)

# Sandkasten Client
[Sandkasten](https://github.com/Defelo/sandkasten) client library for running untrusted code

## Example
```rust,no_run
use sandkasten_client::{
    schemas::programs::{BuildRequest, BuildRunRequest, MainFile},
    SandkastenClient,
};

#[tokio::main]
async fn main() {
    let client = SandkastenClient::new("http://your-sandkasten-instance").parse().unwrap());
    let result = client
        .build_and_run(&BuildRunRequest {
            build: BuildRequest {
                environment: "python".into(),
                main_file: MainFile {
                    name: Some("test.py".into()),
                    content: "print(6 * 7, end='')".into(),
                },
                ..Default::default()
            },
            run: Default::default(),
        })
        .await
        .unwrap();
    assert_eq!(result.run.stdout, "42");
}
```
