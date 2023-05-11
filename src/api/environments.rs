use std::sync::Arc;

use poem_ext::response;
use poem_openapi::OpenApi;
use sandkasten_client::schemas::environments::{Environment, ListEnvironmentsResponse};

use crate::environments::Environments;

use super::Tags;

pub struct EnvironmentsApi {
    pub environments: Arc<Environments>,
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
}

response!(ListEnvironments = {
    /// Map of available environments.
    Ok(200) => ListEnvironmentsResponse,
});
