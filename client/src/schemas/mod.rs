//! Sandkasten request and response schemas.

use serde::Deserialize;

pub mod configuration;
pub mod environments;
pub mod programs;

/// The error responses that any endpoint may return.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "error", content = "reason", rename_all = "snake_case")]
pub enum GeneralError {
    /// 422 Unprocessable Content
    UnprocessableContent(String),
    /// 500 Internal Server Error
    InternalServerError,
}

/// The error responses that Sandkasten may return.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ErrorResponse<E> {
    /// An error response that any endpoint may return.
    GeneralError(GeneralError),
    /// An error response that a specific endpoint may return.
    Inner(E),
}
