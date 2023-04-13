use sandkasten::schemas::programs::{BuildRunRequest, BuildRunResult, RunResult};
use serde::Deserialize;

pub async fn build_and_run(request: &BuildRunRequest) -> Result<BuildRunResult, BuildError> {
    let response = reqwest::Client::new()
        .post(url("/run"))
        .json(request)
        .send()
        .await
        .unwrap();
    let status = response.status();
    dbg!(if status == 200 {
        Ok(response.json().await.unwrap())
    } else {
        Err(response.json().await.unwrap())
    })
}

#[derive(Debug, Deserialize)]
#[serde(tag = "error", content = "details", rename_all = "snake_case")]
pub enum BuildError {
    CompileError(RunResult),
}

pub fn url(path: impl std::fmt::Display) -> String {
    format!("http://127.0.0.1:8000{path}")
}
