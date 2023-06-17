#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(clippy::dbg_macro, clippy::use_debug, clippy::todo)]
#![warn(missing_docs, missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

#[cfg(feature = "reqwest")]
pub use client::*;

pub mod schemas;

/// The version of this client.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(feature = "reqwest")]
mod client {
    use std::{collections::HashMap, fmt::Display};

    use serde::Deserialize;
    use url::Url;

    use crate::schemas::{
        configuration::PublicConfig,
        environments::{BaseResourceUsage, Environment, GetBaseResourceUsageError},
        programs::{
            BuildError, BuildRequest, BuildResult, BuildRunError, BuildRunRequest, BuildRunResult,
            RunError, RunRequest, RunResult,
        },
        ErrorResponse,
    };

    /// An asynchronous client for Sandkasten.
    #[derive(Debug, Clone)]
    pub struct SandkastenClient {
        base_url: Url,
        client: reqwest::Client,
    }

    /// A synchronous client for Sandkasten.
    #[cfg(feature = "blocking")]
    #[derive(Debug, Clone)]
    pub struct BlockingSandkastenClient {
        base_url: Url,
        client: reqwest::blocking::Client,
    }

    impl SandkastenClient {
        /// Create a new client for the Sandkasten instance at `base_url`.
        pub fn new(base_url: Url) -> Self {
            Self {
                base_url,
                client: reqwest::Client::new(),
            }
        }

        /// Return the version of the sandkasten server.
        pub async fn version(&self) -> Result<String> {
            Ok(self.openapi_spec().await?.info.version)
        }
    }

    #[cfg(feature = "blocking")]
    impl BlockingSandkastenClient {
        /// Create a new client for the Sandkasten instance at `base_url`.
        pub fn new(base_url: Url) -> Self {
            Self {
                base_url,
                client: reqwest::blocking::Client::new(),
            }
        }

        /// Return the version of the sandkasten server.
        pub fn version(&self) -> Result<String> {
            Ok(self.openapi_spec()?.info.version)
        }
    }

    /// The errors that may occur when using the client.
    #[derive(Debug, thiserror::Error)]
    pub enum Error<E> {
        /// The endpoint url could not be parsed.
        #[error("could not parse url: {0}")]
        UrlParseError(#[from] url::ParseError),
        /// [`reqwest`] returned an error.
        #[error("reqwest error: {0}")]
        ReqwestError(#[from] reqwest::Error),
        /// Sandkasten returned an error response.
        #[error("sandkasten returned an error: {0:?}")]
        ErrorResponse(Box<ErrorResponse<E>>),
    }

    /// Type alias for `Result<T, sandkasten_client::Error<E>>`.
    pub type Result<T, E = ()> = std::result::Result<T, Error<E>>;

    macro_rules! endpoints {
        ($( $(#[doc = $doc:literal])* $vis:vis $func:ident( $(path: $args:ident),* $(,)? $(json: $data:ty)? ): $method:ident $path:literal => $ok:ty $(, $err:ty)?; )*) => {
            impl SandkastenClient {
                $(
                    $(#[doc = $doc])*
                    $vis async fn $func(&self, $($args: impl Display,)* $(data: &$data)?) -> Result<$ok, $($err)?> {
                        let response = self
                            .client
                            .$method(self.base_url.join(&format!($path))?)
                            $(.json(data as &$data))?
                            .send()
                            .await?;
                        if response.status().is_success() {
                            Ok(response.json().await?)
                        } else {
                            Err(Error::ErrorResponse(response.json().await?))
                        }
                    }
                )*
            }

            #[cfg(feature = "blocking")]
            impl BlockingSandkastenClient {
                $(
                    $(#[doc = $doc])*
                    $vis fn $func(&self, $($args: impl Display,)* $(data: &$data)?) -> Result<$ok, $($err)?> {
                        let response = self
                            .client
                            .$method(self.base_url.join(&format!($path))?)
                            $(.json(data as &$data))?
                            .send()?;
                        if response.status().is_success() {
                            Ok(response.json()?)
                        } else {
                            Err(Error::ErrorResponse(response.json()?))
                        }
                    }
                )*
            }
        };
    }

    endpoints! {
        /// Return the public configuration of Sandkasten.
        pub get_config(): get "config" => PublicConfig;
        /// Return a list of all environments.
        pub list_environments(): get "environments" => HashMap<String, Environment>;
        /// Return the base resource usage of an environment when running just a very basic program.
        pub get_base_resource_usage(path: environment): get "environments/{environment}/resource_usage" => BaseResourceUsage, GetBaseResourceUsageError;
        /// Build and immediately run a program.
        pub build_and_run(json: BuildRunRequest): post "run" => BuildRunResult, BuildRunError;
        /// Upload and compile a program.
        pub build(json: BuildRequest): post "programs" => BuildResult, BuildError;
        /// Run a program that has previously been built.
        pub run(path: program_id, json: RunRequest): post "programs/{program_id}/run" => RunResult, RunError;

        openapi_spec(): get "openapi.json" => OpenAPISpec;
    }

    #[derive(Deserialize)]
    struct OpenAPISpec {
        info: OpenAPISpecInfo,
    }

    #[derive(Deserialize)]
    struct OpenAPISpecInfo {
        version: String,
    }
}
