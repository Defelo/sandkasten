use std::sync::Arc;

use key_rwlock::KeyRwLock;
use poem_openapi::OpenApi;
use tokio::sync::Semaphore;
use uuid::Uuid;

use self::{configuration::ConfigurationApi, environments::EnvironmentsApi, programs::ProgramsApi};
use crate::{config::Config, environments::Environments};

mod configuration;
mod environments;
mod programs;

#[derive(poem_openapi::Tags)]
enum Tags {
    Configuration,
    Environments,
    Programs,
}

pub fn get_api(
    config: Arc<Config>,
    environments: Arc<Environments>,
    program_lock: Arc<KeyRwLock<Uuid>>,
    job_lock: Arc<KeyRwLock<Uuid>>,
) -> impl OpenApi {
    let request_semaphore = Arc::new(Semaphore::new(config.max_concurrent_jobs));
    (
        ConfigurationApi {
            config: Arc::clone(&config),
        },
        EnvironmentsApi {
            environments: Arc::clone(&environments),
            request_semaphore: Arc::clone(&request_semaphore),
            program_lock: Arc::clone(&program_lock),
            job_lock: Arc::clone(&job_lock),
            config: Arc::clone(&config),
            base_resource_usage_cache: Arc::new(
                environments
                    .iter()
                    .map(|(id, _)| (id.clone(), Default::default()))
                    .collect(),
            ),
        },
        ProgramsApi {
            request_semaphore,
            program_lock,
            job_lock,
            config,
            environments,
        },
        #[cfg(feature = "test_api")]
        test_api::TestApi,
    )
}

#[cfg(feature = "test_api")]
mod test_api {
    pub struct TestApi;

    #[poem_openapi::OpenApi]
    impl TestApi {
        #[oai(path = "/test/exit", method = "post", hidden)]
        async fn exit(&self) {
            std::process::exit(0);
        }
    }
}
