use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use fnct::key;
use key_rwlock::KeyRwLock;
use poem_ext::{response, responses::ErrorResponse, shield_mw::shield};
use poem_openapi::{param::Path, OpenApi};
use sandkasten_client::schemas::{
    environments::{BaseResourceUsage, Environment, ListEnvironmentsResponse, RunResourceUsage},
    programs::BuildRequest,
};
use tokio::sync::Semaphore;
use uuid::Uuid;

use super::Tags;
use crate::{
    config::Config,
    environments::{self, Environments},
    metrics::MetricsData,
    program::{build::build_program, run::run_program},
    Cache,
};

pub struct EnvironmentsApi {
    pub environments: Arc<Environments>,
    pub request_semaphore: Arc<Semaphore>,
    pub config: Arc<Config>,
    pub program_lock: Arc<KeyRwLock<Uuid>>,
    pub job_lock: Arc<KeyRwLock<Uuid>>,
    pub cache: Arc<Cache>,
    pub base_resource_usage_lock: Arc<KeyRwLock<String>>,
}

#[OpenApi(tag = "Tags::Environments")]
impl EnvironmentsApi {
    /// Return a map of all environments.
    ///
    /// The keys represent the environment ids and the values contain additional
    /// information about the environments.
    #[oai(path = "/environments", method = "get")]
    async fn list_environments(&self, metrics: MetricsData<'_>) -> ListEnvironments::Response {
        metrics.0.requests.environments.inc();
        ListEnvironments::ok(ListEnvironmentsResponse(
            self.environments
                .iter()
                .map(|(id, env)| {
                    (
                        id.clone(),
                        Environment {
                            name: env.name.clone(),
                            version: env.version.clone(),
                            default_main_file_name: env.default_main_file_name.clone(),
                            example: env.example.clone(),
                            meta: env.meta.clone(),
                        },
                    )
                })
                .collect(),
        ))
    }

    /// Return the base resource usage of an environment.
    ///
    /// The base resource usage of an environment is measured by benchmarking a
    /// very simple program in this environment that barely does anything. Note
    /// that the compile step is run only once as recompiling the same program
    /// again and again would take too much time in most cases.
    #[oai(
        path = "/environments/:name/resource_usage",
        method = "get",
        transform = "shield"
    )]
    async fn get_base_resource_usage(
        &self,
        metrics: MetricsData<'_>,
        name: Path<String>,
    ) -> GetBaseResourceUsage::Response {
        metrics
            .0
            .requests
            .resource_usage
            .with_label_values(&[&name.0])
            .inc();

        let Some(environment) = self.environments.get(&name.0) else {
            return GetBaseResourceUsage::environment_not_found();
        };

        let cache_hit = AtomicBool::new(true);
        let _guard = self.base_resource_usage_lock.write(name.0.clone()).await;
        let result = self
            .cache
            .cached_result(key!(&name.0), &[], None, || async {
                cache_hit.store(false, Ordering::Relaxed);

                let _guard = self
                    .request_semaphore
                    .acquire_many(self.config.base_resource_usage_permits)
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

        if cache_hit.load(Ordering::Relaxed) {
            metrics
                .0
                .cache_hits
                .resource_usage
                .with_label_values(&[&name.0])
                .inc();
        }

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

/// Measure the base resource usage of a given environment.
async fn get_base_resource_usage(
    config: Arc<Config>,
    environments: Arc<Environments>,
    program_lock: Arc<KeyRwLock<Uuid>>,
    job_lock: Arc<KeyRwLock<Uuid>>,
    environment_id: &str,
    environment: &environments::Environment,
) -> Result<BaseResourceUsage, ErrorResponse> {
    // compile the program once
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

    // run the program multiple times and collect the resource_usage measurements
    let mut results = Vec::with_capacity(config.base_resource_usage_runs);
    for _ in 0..config.base_resource_usage_runs {
        results.push(
            run_program(
                Arc::clone(&config),
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
        run: RunResourceUsage {
            time: results.iter().map(|x| x.time).collect(),
            memory: results.iter().map(|x| x.memory).collect(),
        },
    })
}
