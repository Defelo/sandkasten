#![forbid(unsafe_code)]
#![warn(clippy::dbg_macro, clippy::use_debug, clippy::todo)]

use fnct::{backend::AsyncRedisBackend, format::PostcardFormatter, AsyncCache};
use redis::aio::ConnectionManager;

pub mod api;
pub mod config;
pub mod environments;
pub mod program;
pub mod sandbox;

pub type Cache = AsyncCache<AsyncRedisBackend<ConnectionManager>, PostcardFormatter>;
