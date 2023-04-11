use std::sync::Arc;

use poem_openapi::OpenApi;

use crate::{config::Config, environments::Environments};

use self::{environments::EnvironmentsApi, programs::ProgramsApi};

mod environments;
mod programs;

#[derive(poem_openapi::Tags)]
enum Tags {
    Environments,
    Programs,
}

pub fn get_api(config: Arc<Config>, environments: Arc<Environments>) -> impl OpenApi {
    (
        EnvironmentsApi {
            environments: Arc::clone(&environments),
        },
        ProgramsApi {
            config,
            environments,
            compile_lock: Default::default(),
        },
    )
}
