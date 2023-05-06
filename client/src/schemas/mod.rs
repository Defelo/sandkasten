#![forbid(unsafe_code)]
#![warn(clippy::dbg_macro, clippy::use_debug, clippy::todo)]

use serde::Deserialize;

pub mod environments;
pub mod programs;

#[derive(Debug, Deserialize)]
#[serde(tag = "error", content = "reason", rename_all = "snake_case")]
pub enum GeneralError {
    UnprocessableContent(String),
    InternalServerError,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ErrorResponse<E> {
    GeneralError(GeneralError),
    Inner(E),
}
