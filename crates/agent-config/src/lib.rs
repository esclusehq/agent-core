//! Agent Config - Configuration management for agents
//!
//! This crate handles loading, parsing, validation, and hot-reload
//! of agent configuration from environment variables and files.

pub mod loader;
pub mod schema;
pub mod validator;

pub use loader::{load, get_log_dir, get_log_writer, FileLogGuard};
pub use schema::*;
pub use validator::{validate, ConfigError};
