//! [Sandkasten](https://github.com/Defelo/sandkasten) client library for running untrusted code
//!
//! #### Example
//! ```no_run
//! use sandkasten_client::{
//!     schemas::programs::{BuildRequest, BuildRunRequest, File},
//!     SandkastenClient,
//! };
//!
//! # async fn f() {
//! let client = SandkastenClient::new("http://your-sandkasten-instance".parse().unwrap());
//! let result = client
//!     .build_and_run(&BuildRunRequest {
//!         build: BuildRequest {
//!             environment: "python".into(),
//!             files: vec![File {
//!                 name: "test.py".into(),
//!                 content: "print(6 * 7, end='')".into(),
//!             }],
//!             ..Default::default()
//!         },
//!         run: Default::default(),
//!     })
//!     .await
//!     .unwrap();
//! assert_eq!(result.run.stdout, "42");
//! # }
//! ```

#![forbid(unsafe_code)]
#![warn(clippy::dbg_macro, clippy::use_debug, clippy::todo)]
#![warn(missing_docs, missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

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

    /// An asynchronous client for Sandkasten.
    #[derive(Debug)]
    pub struct SandkastenClient {
        base_url: Url,
        client: reqwest::Client,
    }

    /// A synchronous client for Sandkasten.
    #[cfg(feature = "blocking")]
    #[derive(Debug)]
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
        ($( $(#[doc = $doc:literal])* $func:ident( $(path: $args:ident,)* $(json: $data:ty)? ): $method:ident $path:literal => $ok:ty $(, $err:ty)?; )*) => {
            impl SandkastenClient {
                $(
                    $(#[doc = $doc])*
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
                    $(#[doc = $doc])*
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
        /// Return a list of all environments.
        list_environments(): get "environments" => HashMap<String, Environment>;
        /// Build and immediately run a program.
        build_and_run(json: BuildRunRequest): post "run" => BuildRunResult, BuildRunError;
        /// Upload and compile a program.
        build(json: BuildRequest): post "programs" => BuildResult, BuildError;
        /// Run a program that has previously been built.
        run(path: program_id, json: RunRequest): post "programs/{program_id}/run" => RunResult, RunError;
    }
}
