use std::sync::Arc;

use poem_ext::response;
use poem_openapi::OpenApi;

use crate::{
    environments::Environments,
    schemas::{Environment, ListEnvironmentsResponse},
};

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
