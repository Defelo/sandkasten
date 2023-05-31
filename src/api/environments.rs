use std::{iter::Map, slice::Iter, sync::Arc};

use fnct::key;
use key_rwlock::KeyRwLock;
use poem_ext::{response, responses::ErrorResponse, shield_mw::shield};
use poem_openapi::{param::Path, OpenApi};
use sandkasten_client::schemas::{
    environments::{BaseResourceUsage, Environment, ListEnvironmentsResponse},
    programs::{BuildRequest, ResourceUsage},
};
use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::{
    config::Config,
    environments::{self, Environments},
    program::{build_program, run_program},
    Cache,
};

use super::Tags;

pub struct EnvironmentsApi {
    pub environments: Arc<Environments>,
    pub request_semaphore: Arc<Semaphore>,
    pub config: Arc<Config>,
    pub program_lock: Arc<KeyRwLock<Uuid>>,
    pub job_lock: Arc<KeyRwLock<Uuid>>,
    pub cache: Arc<Cache>,
    pub bru_lock: Arc<KeyRwLock<String>>,
}

#[OpenApi(tag = "Tags::Environments")]
impl EnvironmentsApi {
    /// Return a list of all environments.
    #[oai(path = "/environments", method = "get")]
    async fn list_environments(&self) -> ListEnvironments::Response {
        ListEnvironments::ok(ListEnvironmentsResponse(
            self.environments
                .environments
                .iter()
                .map(|(id, env)| {
                    (
                        id.clone(),
                        Environment {
                            name: env.name.clone(),
                            version: env.version.clone(),
                            default_main_file_name: env.default_main_file_name.clone(),
                        },
                    )
                })
                .collect(),
        ))
    }

    /// Return the base resource usage of an environment when running just a very basic program.
    #[oai(
        path = "/environments/:name/resource_usage",
        method = "get",
        transform = "shield"
    )]
    async fn get_base_resource_usage(&self, name: Path<String>) -> GetBaseResourceUsage::Response {
        let Some(environment) = self.environments.environments.get(&name.0) else {
            return GetBaseResourceUsage::environment_not_found();
        };

        let _guard = self.bru_lock.write(name.0.clone()).await;
        let result = self
            .cache
            .cached_result(key!(&name.0), &[], None, || async {
                let _guard = self
                    .request_semaphore
                    .acquire_many(self.config.max_concurrent_jobs as _)
                    .await?;

                get_base_resource_usage(
                    Arc::clone(&self.config),
                    Arc::clone(&self.environments),
                    Arc::clone(&self.program_lock),
                    Arc::clone(&self.job_lock),
                    &name.0,
                    environment,
                )
                .await
            })
            .await??;
        GetBaseResourceUsage::ok(result)
    }
}

response!(ListEnvironments = {
    /// Map of available environments.
    Ok(200) => ListEnvironmentsResponse,
});

response!(GetBaseResourceUsage = {
    /// Base resource usage of build and run step.
    Ok(200) => BaseResourceUsage,
    /// Environment does not exist.
    EnvironmentNotFound(404, error),
});

async fn get_base_resource_usage(
    config: Arc<Config>,
    environments: Arc<Environments>,
    program_lock: Arc<KeyRwLock<Uuid>>,
    job_lock: Arc<KeyRwLock<Uuid>>,
    environment_id: &str,
    environment: &environments::Environment,
) -> Result<BaseResourceUsage, ErrorResponse> {
    let (build, _guard) = build_program(
        Arc::clone(&config),
        Arc::clone(&environments),
        BuildRequest {
            environment: environment_id.into(),
            main_file: environment.test.main_file.clone(),
            files: environment.test.files.clone(),
            ..Default::default()
        },
        program_lock,
        Arc::clone(&job_lock),
    )
    .await?;

    let mut res = Vec::with_capacity(config.base_resource_usage_runs);
    for _ in 0..config.base_resource_usage_runs {
        res.push(
            run_program(
                Arc::clone(&config),
                Arc::clone(&environments),
                build.program_id,
                Default::default(),
                &_guard,
                Arc::clone(&job_lock),
            )
            .await?
            .resource_usage,
        );
    }

    Ok(BaseResourceUsage {
        build: build.compile_result.map(|x| x.resource_usage),
        run_min: acc(&res, |x| x.min().unwrap()),
        run_max: acc(&res, |x| x.max().unwrap()),
        run_avg: acc(&res, |x| {
            let n = x.len();
            x.sum::<u64>() / n as u64
        }),
    })
}

type Acc = fn(Map<Iter<ResourceUsage>, fn(&ResourceUsage) -> u64>) -> u64;
fn acc(res: &[ResourceUsage], f: Acc) -> ResourceUsage {
    ResourceUsage {
        time: f(res.iter().map(|r| r.time)),
        memory: f(res.iter().map(|r| r.memory)),
    }
}
