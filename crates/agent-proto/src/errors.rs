//! Shared error types for agent communication

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtoError {
    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    #[error("Protocol version mismatch: agent={0}, backend={1}")]
    VersionMismatch(u32, u32),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Invalid task payload: {0}")]
    InvalidPayload(String),
}

impl serde::Serialize for ProtoError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proto_error_serialization() {
        let err = ProtoError::InvalidMessage("test error".to_string());
        let serialized = serde_json::to_string(&err).unwrap();
        assert!(serialized.contains("test error"));
    }
}
