use std::collections::HashMap;

#[cfg(feature = "poem-openapi")]
use poem_openapi::{types::Example, NewType, Object};
use serde::{Deserialize, Serialize};

/// A package that can build and run programs.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct Environment {
    pub name: String,
    pub version: String,
}

#[derive(Debug)]
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
