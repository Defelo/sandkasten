//! Schemas for environments endpoints.

use std::collections::HashMap;

#[cfg(feature = "poem-openapi")]
use poem_openapi::{types::Example, NewType, Object};
use serde::{Deserialize, Serialize};

/// A package that can build and run programs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct Environment {
    /// The display name of the environment (e.g. `Rust` or `C++`).
    pub name: String,
    /// The version of the environment.
    pub version: String,
    /// The default name of the main file that is used if no filename is specified.
    pub default_main_file_name: String,
}

/// A map of environments where the key represents the id of the environment.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "poem-openapi", derive(NewType))]
#[cfg_attr(
    feature = "poem-openapi",
    oai(
        from_parameter = false,
        from_multipart = false,
        to_header = false,
        example = true
    )
)]
pub struct ListEnvironmentsResponse(pub HashMap<String, Environment>);

#[cfg(feature = "poem-openapi")]
impl Example for ListEnvironmentsResponse {
    fn example() -> Self {
        Self(HashMap::from([
            (
                "rust".into(),
                Environment {
                    name: "Rust".into(),
                    version: "1.64.0".into(),
                    default_main_file_name: "code.rs".into(),
                },
            ),
            (
                "python".into(),
                Environment {
                    name: "Python".into(),
                    version: "3.11.1".into(),
                    default_main_file_name: "code.py".into(),
                },
            ),
        ]))
    }
}
