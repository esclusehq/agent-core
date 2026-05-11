//! Configuration validation

use super::AgentConfig;

pub fn validate(config: &AgentConfig) -> Result<(), Vec<ConfigError>> {
    let mut errors = Vec::new();

    if !config.backend_url.starts_with("wss://")
        && !config.backend_url.starts_with("ws://")
        && !config.backend_url.starts_with("http://")
        && !config.backend_url.starts_with("https://")
    {
        errors.push(ConfigError::Invalid {
            field: "backend_url".to_string(),
            message: "must start with wss://, ws://, https://, or http://".to_string(),
        });
    }

    if config.api_key.is_empty() {
        errors.push(ConfigError::Missing {
            field: "api_key".to_string(),
        });
    }

    if config.heartbeat_interval_secs < 10 || config.heartbeat_interval_secs > 300 {
        errors.push(ConfigError::OutOfRange {
            field: "heartbeat_interval_secs".to_string(),
            min: 10,
            max: 300,
            value: config.heartbeat_interval_secs as i64,
        });
    }

    if config.max_concurrent_tasks == 0 {
        errors.push(ConfigError::Invalid {
            field: "max_concurrent_tasks".to_string(),
            message: "must be >= 1".to_string(),
        });
    }

    if let Some(addr) = &config.metrics_listen_addr {
        let is_loopback = addr.ip().is_loopback();
        let is_unspecified = addr.ip().is_unspecified();
        if is_unspecified && !is_loopback {
            errors.push(ConfigError::Invalid {
                field: "metrics_listen_addr".to_string(),
                message: "must be 127.0.0.1 or specific IP, not 0.0.0.0".to_string(),
            });
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Missing {
        field: String,
    },
    Invalid {
        field: String,
        message: String,
    },
    OutOfRange {
        field: String,
        min: i64,
        max: i64,
        value: i64,
    },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Missing { field } => write!(f, "field '{}' is missing", field),
            ConfigError::Invalid { field, message } => {
                write!(f, "field '{}' is invalid: {}", field, message)
            }
            ConfigError::OutOfRange {
                field,
                min,
                max,
                value,
            } => {
                write!(
                    f,
                    "field '{}' value {} is out of range [{}, {}]",
                    field, value, min, max
                )
            }
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_config() {
        let mut config = AgentConfig::default();
        config.backend_url = "wss://api.example.com".to_string();
        config.api_key = "test_key".into();

        let result = validate(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_missing_api_key() {
        let mut config = AgentConfig::default();
        config.backend_url = "wss://api.example.com".to_string();
        config.api_key = "".into();

        let result = validate(&config);
        assert!(result.is_err());
    }
}
