//! JWT validation for tasks

use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskClaims {
    pub agent_id: Uuid,
    pub task_id: Uuid,
    pub exp: usize,
}

pub struct JwtValidator {
    public_key: DecodingKey,
    agent_id: Uuid,
}

#[derive(Debug, Error)]
pub enum JwtError {
    #[error("Invalid JWT: {0}")]
    Invalid(String),

    #[error("Token expired at {0}")]
    Expired(String),

    #[error("Task JWT is for wrong agent (expected {expected}, got {got})")]
    WrongAgent { expected: Uuid, got: Uuid },

    #[error("Task ID in JWT does not match task.id")]
    TaskIdMismatch,

    #[error("Missing required claim")]
    MissingClaim,
}

impl JwtValidator {
    pub fn new(public_key: String, agent_id: Uuid) -> Result<Self, JwtError> {
        let decoding_key = if public_key.contains("BEGIN") {
            DecodingKey::from_rsa_pem(public_key.as_bytes())
                .map_err(|e| JwtError::Invalid(format!("Invalid PEM: {}", e)))?
        } else {
            DecodingKey::from_secret(public_key.as_bytes())
        };

        Ok(Self {
            public_key: decoding_key,
            agent_id,
        })
    }

    pub fn validate(&self, token: &str, expected_task_id: Uuid) -> Result<TaskClaims, JwtError> {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;
        validation.required_spec_claims.insert("exp".to_string());

        let token_data =
            decode::<TaskClaims>(token, &self.public_key, &validation).map_err(|e| {
                match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                        JwtError::Expired("Token has expired".to_string())
                    }
                    _ => JwtError::Invalid(e.to_string()),
                }
            })?;

        if token_data.claims.agent_id != self.agent_id {
            return Err(JwtError::WrongAgent {
                expected: self.agent_id,
                got: token_data.claims.agent_id,
            });
        }

        if token_data.claims.task_id != expected_task_id {
            return Err(JwtError::TaskIdMismatch);
        }

        Ok(token_data.claims)
    }
}

pub fn decode_token(token: &str) -> Result<TaskClaims, JwtError> {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = true;

    decode::<TaskClaims>(token, &DecodingKey::from_secret(&[]), &validation)
        .map_err(|e| JwtError::Invalid(e.to_string()))
        .map(|data| data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_validator_creation() {
        let validator = JwtValidator::new("secret_key".to_string(), Uuid::new_v4());
        assert!(validator.is_ok());
    }
}
