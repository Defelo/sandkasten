use std::sync::Arc;

use key_rwlock::KeyRwLock;
use poem_openapi::OpenApi;
use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::{config::Config, environments::Environments};

use self::{environments::EnvironmentsApi, programs::ProgramsApi};

mod environments;
mod programs;

#[derive(poem_openapi::Tags)]
enum Tags {
    Environments,
    Programs,
}

pub fn get_api(
    config: Arc<Config>,
    environments: Arc<Environments>,
    program_lock: Arc<KeyRwLock<Uuid>>,
    job_lock: Arc<KeyRwLock<Uuid>>,
) -> impl OpenApi {
    (
        EnvironmentsApi {
            environments: Arc::clone(&environments),
        },
        ProgramsApi {
            request_semaphore: Semaphore::new(config.max_concurrent_jobs),
            program_lock,
            job_lock,
            config,
            environments,
        },
    )
}
