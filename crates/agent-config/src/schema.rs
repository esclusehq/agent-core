//! Configuration schema for agent

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    // WAJIB
    pub backend_url: String,
    pub api_key: SecretString,

    // IDENTITY
    pub agent_name: String,
    pub agent_id: Option<uuid::Uuid>,

    // CONNECTION
    pub heartbeat_interval_secs: u64,
    pub reconnect_initial_secs: u64,
    pub reconnect_max_secs: u64,
    pub reconnect_multiplier: f64,
    pub ws_ping_interval_secs: u64,

    // TASK EXECUTION
    pub max_concurrent_tasks: usize,
    pub task_timeout_default_secs: u64,

    // RUNTIME
    pub runtime_preference: RuntimePreference,

    // METRICS
    pub metrics_interval_secs: u64,
    pub metrics_listen_addr: Option<SocketAddr>,

    // LOGGING
    pub log_level: String,
    pub log_format: LogFormat,

    // TIMEOUT (D-14, D-15, D-16)
    pub default_timeout_secs: u64,                     // D-14: Global default 30 seconds
    pub operation_timeout_overrides: HashMap<String, u64>, // D-15: Per-op override
    pub enable_cancel: bool,                          // D-16: Support cancellation via tokio

    // DATA
    pub data_dir: PathBuf,

    // ALERTS (D-01)
    pub alerts: AlertsConfig,
}

impl Default for AgentConfig {
    fn default() -> Self {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("escluse-agent");

        Self {
            backend_url: "wss://app.esluce.com/api/ws/node".to_string(),
            api_key: SecretString::new(String::new()),
            agent_name: hostname::get()
                .map(|h| h.to_string_lossy().to_string())
                .unwrap_or_else(|_| "agent".to_string()),
            agent_id: None,
            heartbeat_interval_secs: 30,
            reconnect_initial_secs: 2,
            reconnect_max_secs: 120,
            reconnect_multiplier: 2.0,
            ws_ping_interval_secs: 20,
            max_concurrent_tasks: 10,
            task_timeout_default_secs: 300,
            runtime_preference: RuntimePreference::Auto,
            metrics_interval_secs: 60,
            metrics_listen_addr: None,
            log_level: "info".to_string(),
            log_format: LogFormat::Text,
            // D-14: Global default timeout 30 seconds
            default_timeout_secs: 30,
            operation_timeout_overrides: HashMap::new(),
            enable_cancel: true,
            data_dir,
            // D-01: Alert thresholds (CPU >80%, Memory >85%, Disk >90%)
            alerts: AlertsConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RuntimePreference {
    Auto,
    Docker,
    Podman,
    None,
}

impl Default for RuntimePreference {
    fn default() -> Self {
        RuntimePreference::Auto
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogFormat {
    Text,
    Json,
}

impl Default for LogFormat {
    fn default() -> Self {
        LogFormat::Text
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretString {
    inner: String,
}

impl SecretString {
    pub fn new(s: String) -> Self {
        Self { inner: s }
    }

    pub fn expose_secret(&self) -> &str {
        &self.inner
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl Default for SecretString {
    fn default() -> Self {
        Self::new(String::new())
    }
}

impl From<String> for SecretString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for SecretString {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

/// Alert configuration thresholds (D-01)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertsConfig {
    pub cpu_threshold_percent: f64,
    pub memory_threshold_percent: f64,
    pub disk_threshold_percent: f64,
}

impl Default for AlertsConfig {
    fn default() -> Self {
        Self {
            cpu_threshold_percent: 80.0,
            memory_threshold_percent: 85.0,
            disk_threshold_percent: 90.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AgentConfig::default();
        assert_eq!(config.heartbeat_interval_secs, 30);
        assert_eq!(config.max_concurrent_tasks, 10);
    }

    #[test]
    fn test_secret_string() {
        let secret = SecretString::new("api_key_123".to_string());
        assert_eq!(secret.expose_secret(), "api_key_123");
        assert!(!secret.is_empty());
    }
}
