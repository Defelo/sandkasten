use std::sync::Arc;

use poem_ext::response;
use poem_openapi::OpenApi;
use sandkasten_client::schemas::configuration::PublicConfig;

use super::Tags;
use crate::config::Config;

pub struct ConfigurationApi {
    pub config: Arc<Config>,
}

#[OpenApi(tag = "Tags::Configuration")]
impl ConfigurationApi {
    /// Return the public configuration of Sandkasten.
    #[oai(path = "/config", method = "get")]
    async fn get_config(&self) -> GetConfig::Response {
        GetConfig::ok(PublicConfig {
            program_ttl: self.config.program_ttl,
            max_concurrent_jobs: self.config.max_concurrent_jobs,
            compile_limits: self.config.compile_limits.clone(),
            run_limits: self.config.run_limits.clone(),
            base_resource_usage_runs: self.config.base_resource_usage_runs,
        })
    }
}

response!(GetConfig = {
    /// The public configuration of Sandkasten.
    Ok(200) => PublicConfig,
});
