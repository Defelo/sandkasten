#![forbid(unsafe_code)]
#![warn(clippy::dbg_macro, clippy::use_debug, clippy::todo)]

pub mod api;
pub mod config;
pub mod environments;
pub mod metrics;
pub mod program;
pub mod sandbox;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
