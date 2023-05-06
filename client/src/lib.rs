#![forbid(unsafe_code)]
#![warn(clippy::dbg_macro, clippy::use_debug, clippy::todo)]

#[cfg(feature = "reqwest")]
pub use client::*;

pub mod schemas;

#[cfg(feature = "reqwest")]
mod client {
    use std::fmt::Display;

    use crate::schemas::{
        environments::Environment,
        programs::{
            BuildError, BuildRequest, BuildResult, BuildRunError, BuildRunRequest, BuildRunResult,
            RunError, RunRequest, RunResult,
        },
        ErrorResponse,
    };
    use std::collections::HashMap;
    use url::Url;

    pub struct SandkastenClient {
        base_url: Url,
        client: reqwest::Client,
    }

    #[cfg(feature = "blocking")]
    pub struct BlockingSandkastenClient {
        base_url: Url,
        client: reqwest::blocking::Client,
    }

    impl SandkastenClient {
        pub fn new(base_url: Url) -> Self {
            Self {
                base_url,
                client: reqwest::Client::new(),
            }
        }
    }

    #[cfg(feature = "blocking")]
    impl BlockingSandkastenClient {
        pub fn new(base_url: Url) -> Self {
            Self {
                base_url,
                client: reqwest::blocking::Client::new(),
            }
        }
    }

    #[derive(Debug, thiserror::Error)]
    pub enum Error<E> {
        #[error("could not parse url: {0}")]
        UrlParseError(#[from] url::ParseError),
        #[error("reqwest error: {0}")]
        ReqwestError(#[from] reqwest::Error),
        #[error("sandkasten returned an error: {0:?}")]
        ErrorResponse(Box<ErrorResponse<E>>),
    }

    pub type Result<T, E = ()> = std::result::Result<T, Error<E>>;

    macro_rules! endpoints {
        ($( $func:ident( $(path: $args:ident,)* $(json: $data:ty)? ): $method:ident $path:literal => $ok:ty $(, $err:ty)?; )*) => {
            impl SandkastenClient {
                $(
                    pub async fn $func(&self, $($args: impl Display,)* $(data: &$data)?) -> Result<$ok, $($err)?> {
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
                    pub fn $func(&self, $($args: impl Display,)* $(data: &$data)?) -> Result<$ok, $($err)?> {
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
        list_environments(): get "environments" => HashMap<String, Environment>;
        build_and_run(json: BuildRunRequest): post "run" => BuildRunResult, BuildRunError;
        build(json: BuildRequest): post "programs" => BuildResult, BuildError;
        run(path: program_id, json: RunRequest): post "programs/{program_id}/run" => RunResult, RunError;
    }
}
