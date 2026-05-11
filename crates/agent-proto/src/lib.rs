//! Agent Proto - Protocol types and messages
//! 
//! This crate contains all shared types for communication between
//! agents and the backend. It has no internal dependencies.

pub mod agent;
pub mod errors;
pub mod messages;
pub mod protocol;
pub mod task;

pub use agent::*;
pub use errors::*;
pub use messages::*;
pub use protocol::*;
pub use task::*;
