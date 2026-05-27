//! WebSocket message types for agent-backend communication

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::task::{Task, TaskResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentToBackend {
    Register(RegisterPayload),
    Heartbeat(HeartbeatPayload),
    TaskResult(TaskResult),
    MetricsReport(MetricsPayload),
    LogLine(LogLinePayload),
    StatusUpdate(AgentStatusPayload),
    DnsStatus(DnsStatusPayload),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BackendToAgent {
    RegisterAck(RegisterAckPayload),
    TaskAssign(Task),
    TaskCancel(TaskCancelPayload),
    Ping,
    ConfigUpdate(serde_json::Value),
    DnsConfig(DnsConfigPayload),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterPayload {
    pub agent_name: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub platform: String,
    pub runtime: Option<String>,
    pub protocol_version: u32,
    pub total_memory: Option<u64>,
    pub cpu_cores: Option<u32>,
}

impl RegisterPayload {
    pub fn new(agent_name: String, capabilities: Vec<String>) -> Self {
        Self {
            agent_name,
            version: env!("CARGO_PKG_VERSION").to_string(),
            capabilities,
            platform: std::env::consts::OS.to_string(),
            runtime: None,
            protocol_version: super::protocol::PROTOCOL_VERSION,
            total_memory: None,
            cpu_cores: None,
        }
    }
    
    pub fn with_system_info(mut self, total_memory: u64, cpu_cores: u32) -> Self {
        self.total_memory = Some(total_memory);
        self.cpu_cores = Some(cpu_cores);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterAckPayload {
    pub agent_id: Uuid,
    pub heartbeat_interval_secs: u64,
    pub protocol_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatPayload {
    pub agent_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub task_count: u32,
    pub status: AgentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLinePayload {
    pub server_id: Uuid,
    pub line: String,
    pub timestamp: DateTime<Utc>,
    pub stream: LogStream,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogStream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsPayload {
    pub timestamp: DateTime<Utc>,
    pub cpu_percent: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub disk_usage: Vec<DiskUsage>,
    pub net_rx_bytes: u64,
    pub net_tx_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskUsage {
    pub mount_point: String,
    pub used_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatusPayload {
    pub agent_id: Uuid,
    pub status: AgentStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentStatus {
    Online,
    Offline,
    Busy,
    Error,
}

impl Default for AgentStatus {
    fn default() -> Self {
        AgentStatus::Online
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCancelPayload {
    pub task_id: Uuid,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsConfigPayload {
    pub api_token: String,
    pub zone_id: String,
    pub zone_name: String,
    pub wildcard_domain: String,
    pub auto_refresh: bool,
    pub refresh_interval_secs: u64,
    pub public_ip: Option<String>,
    pub subdomain: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsStatusPayload {
    pub domain: String,
    pub record_type: String,
    pub record_id: Option<String>,
    pub ip: String,
    pub status: DnsRecordStatus,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DnsRecordStatus {
    Created,
    Updated,
    Deleted,
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_payload() {
        let payload = RegisterPayload::new("test-agent".to_string(), vec!["docker".to_string()]);
        assert_eq!(payload.agent_name, "test-agent");
        assert!(payload.capabilities.contains(&"docker".to_string()));
    }

    #[test]
    fn test_heartbeat_payload() {
        let payload = HeartbeatPayload {
            agent_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            task_count: 5,
            status: AgentStatus::Online,
        };
        assert_eq!(payload.status, AgentStatus::Online);
    }
}
