use std::collections::HashMap;

use poem_openapi::{types::Example, NewType, Object};
use serde::Deserialize;

/// A package that can build and run programs.
#[derive(Debug, Object, Deserialize)]
pub struct Environment {
    pub name: String,
    pub version: String,
}

#[derive(Debug, NewType)]
#[oai(
    from_parameter = false,
    from_multipart = false,
    to_header = false,
    example = true
)]
pub struct ListEnvironmentsResponse(pub HashMap<String, Environment>);

impl Example for ListEnvironmentsResponse {
    fn example() -> Self {
        Self(HashMap::from([
            (
                "rust".into(),
                Environment {
                    name: "Rust".into(),
                    version: "1.64.0".into(),
                },
            ),
            (
                "python".into(),
                Environment {
                    name: "Python".into(),
                    version: "3.11.1".into(),
                },
            ),
        ]))
    }
}
