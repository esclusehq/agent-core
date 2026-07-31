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
    LogLine(LogLinePayload),
    StatusUpdate(AgentStatusPayload),
    DnsStatus(DnsStatusPayload),
    CrashReport(CrashReportPayload),
    ContainerEvent(ContainerEventPayload),
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
    RelayConfigSync(RelayConfigPayload),
    #[serde(rename = "relay_connect")]
    RelayConnect(RelayConnectPayload),
    #[serde(rename = "relay_disconnect")]
    RelayDisconnect(RelayDisconnectPayload),
    #[serde(rename = "error")]
    BackendError(BackendErrorPayload),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterPayload {
    #[serde(rename = "name")]
    pub agent_name: String,
    #[serde(rename = "agent_version")]
    pub version: String,
    pub capabilities: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(rename = "container_runtime")]
    pub runtime: Option<String>,
    #[serde(skip)]
    pub protocol_version: u32,
    #[serde(default)]
    pub total_memory: Option<u64>,
    #[serde(default)]
    pub cpu_cores: Option<u32>,
    #[serde(rename = "id")]
    pub agent_id: Option<Uuid>,
    pub ip: String,
    #[serde(default)]
    pub os_info: Option<String>,
    #[serde(default)]
    pub podman_version: Option<String>,
    #[serde(default)]
    pub containers: Vec<serde_json::Value>,
}

impl RegisterPayload {
    pub fn new(agent_name: String, capabilities: Vec<String>) -> Self {
        Self {
            agent_name,
            version: env!("CARGO_PKG_VERSION").to_string(),
            capabilities,
            platform: Some(std::env::consts::OS.to_string()),
            runtime: None,
            protocol_version: super::protocol::PROTOCOL_VERSION,
            total_memory: None,
            cpu_cores: None,
            agent_id: None,
            ip: String::new(),
            os_info: Some(std::env::consts::OS.to_string()),
            podman_version: None,
            containers: Vec::new(),
        }
    }

    pub fn with_system_info(mut self, total_memory: u64, cpu_cores: u32) -> Self {
        self.total_memory = Some(total_memory);
        self.cpu_cores = Some(cpu_cores);
        self
    }

    pub fn with_node_info(mut self, agent_id: Uuid, ip: String, os_info: String, podman_version: Option<String>) -> Self {
        self.agent_id = Some(agent_id);
        self.ip = ip;
        self.os_info = Some(os_info);
        self.podman_version = podman_version;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterAckPayload {
    #[serde(default, rename = "node_id")]
    pub agent_id: Option<Uuid>,
    #[serde(default)]
    pub heartbeat_interval_secs: u64,
    #[serde(default)]
    pub protocol_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatPayload {
    #[serde(rename = "node_id")]
    pub agent_id: Uuid,
    pub status: String,
    #[serde(default)]
    pub metrics: Option<NodeMetrics>,
    #[serde(default)]
    pub containers: Vec<ContainerStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub cpu_usage: f64,
    pub memory_used: u64,
    pub memory_total: u64,
    pub disk_used: u64,
    pub disk_total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStatus {
    pub id: String,
    pub name: String,
    pub status: String,
    #[serde(default)]
    pub cpu: f64,
    #[serde(default)]
    pub memory: u64,
    #[serde(default)]
    pub memory_limit: Option<u64>,
    #[serde(default)]
    pub disk_usage: Option<u64>,
    #[serde(default)]
    pub players: Option<i32>,
    #[serde(default)]
    pub tps: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLinePayload {
    #[serde(rename = "node_id")]
    pub agent_id: Uuid,
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
    #[serde(rename = "node_id")]
    pub agent_id: Uuid,
    pub status: AgentStatus,
    pub task_id: Option<Uuid>,
    pub progress: Option<f32>,
    pub message: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashReportPayload {
    #[serde(rename = "node_id")]
    pub agent_id: Uuid,
    pub exit_code: i32,
    pub log_excerpt: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayServerInfo {
    pub server_id: Uuid,
    pub subdomain: String,
    pub local_mc_addr: String,
    pub public_port: u16,
    pub loader: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayConfigPayload {
    pub relay_token: String,
    pub gateway_url: String,
    pub region: String,
    pub servers: Vec<RelayServerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerEventPayload {
    #[serde(rename = "node_id")]
    pub agent_id: Uuid,
    pub event: String,
    pub container_id: String,
    pub container_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendErrorPayload {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayDisconnectPayload {
    pub server_id: Uuid,
}

/// Backend tells the agent to open a relay tunnel for a single server.
/// Sent on server create if a node is already assigned, and on agent
/// reconnect for every server assigned to that node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayConnectPayload {
    pub server_id: Uuid,
    /// Short hex subdomain (e.g. "a3f8b") the server is reachable at
    /// on the relay gateway.
    pub subdomain: String,
    /// Gateway public port (e.g. 25565 for Minecraft Java).
    pub public_port: u16,
    /// Local address the relay client should forward to
    /// (e.g. "127.0.0.1:25565").
    pub local_mc_addr: String,
    /// Optional loader discriminator. "bedrock" triggers UDP relay
    /// instead of TCP subdomain routing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loader: Option<String>,
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
            status: "online".to_string(),
            metrics: Some(NodeMetrics {
                cpu_usage: 42.5,
                memory_used: 8589934592,
                memory_total: 17179869184,
                disk_used: 107374182400,
                disk_total: 536870912000,
            }),
            containers: vec![],
        };
        assert_eq!(payload.status, "online");
        assert!(payload.metrics.is_some());
        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["status"], "online");
        assert!(json["metrics"]["cpu_usage"].is_number());
    }

    #[test]
    fn test_crash_report_payload() {
        let payload = CrashReportPayload {
            agent_id: Uuid::nil(),
            exit_code: 1,
            log_excerpt: "panic".into(),
            timestamp: Utc::now(),
        };
        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["node_id"], Uuid::nil().to_string());
        assert_eq!(json["exit_code"], 1);
        assert_eq!(json["log_excerpt"], "panic");
    }

    #[test]
    fn test_relay_config_payload() {
        let server = RelayServerInfo {
            server_id: Uuid::new_v4(),
            subdomain: "test".into(),
            local_mc_addr: "127.0.0.1:25565".into(),
            public_port: 25565,
            loader: None,
        };
        let payload = RelayConfigPayload {
            relay_token: "tok".into(),
            gateway_url: "https://gw".into(),
            region: "us".into(),
            servers: vec![server],
        };
        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["relay_token"], "tok");
        assert_eq!(json["servers"][0]["subdomain"], "test");
        let deserialized: RelayConfigPayload = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.relay_token, "tok");
        assert_eq!(deserialized.servers.len(), 1);
    }

    #[test]
    fn test_container_event_payload() {
        let payload = ContainerEventPayload {
            agent_id: Uuid::new_v4(),
            event: "die".into(),
            container_id: "abc123".into(),
            container_name: "mc-server".into(),
        };
        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["event"], "die");
        let deserialized: ContainerEventPayload = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.container_id, "abc123");
    }

    #[test]
    fn test_backend_error_payload() {
        let payload = BackendErrorPayload {
            code: "ERR_001".into(),
            message: "something went wrong".into(),
        };
        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["code"], "ERR_001");
        assert_eq!(json["message"], "something went wrong");
    }

    #[test]
    fn test_register_extended() {
        let payload = RegisterPayload::new("agent".into(), vec!["docker".into()])
            .with_node_info(Uuid::nil(), "10.0.0.1".into(), "linux".into(), Some("4.0".into()));
        assert_eq!(payload.agent_id, Some(Uuid::nil()));
        assert_eq!(payload.ip, "10.0.0.1");
        assert_eq!(payload.os_info, Some("linux".to_string()));
        assert_eq!(payload.podman_version, Some("4.0".into()));
    }

    #[test]
    fn test_agent_status_extended() {
        let payload = AgentStatusPayload {
            agent_id: Uuid::new_v4(),
            status: AgentStatus::Busy,
            task_id: Some(Uuid::new_v4()),
            progress: Some(0.5),
            message: Some("working".into()),
        };
        let json = serde_json::to_value(&payload).unwrap();
        assert!(json["task_id"].is_string());
        assert_eq!(json["progress"], 0.5);
        assert_eq!(json["message"], "working");
    }

    #[test]
    fn test_logline_agent_id() {
        let payload = LogLinePayload {
            agent_id: Uuid::nil(),
            line: "test".into(),
            timestamp: Utc::now(),
            stream: LogStream::Stdout,
        };
        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["node_id"], Uuid::nil().to_string());
        assert!(json.get("server_id").is_none());
    }

    #[test]
    fn test_enum_tagged_json() {
        let crash = AgentToBackend::CrashReport(CrashReportPayload {
            agent_id: Uuid::nil(),
            exit_code: 1,
            log_excerpt: "oops".into(),
            timestamp: Utc::now(),
        });
        let json = serde_json::to_value(&crash).unwrap();
        assert_eq!(json["type"], "crash_report");
        assert_eq!(json["exit_code"], 1);

        let relay_sync = BackendToAgent::RelayConfigSync(RelayConfigPayload {
            relay_token: "tok".into(),
            gateway_url: "https://gw".into(),
            region: "us".into(),
            servers: vec![],
        });
        let json = serde_json::to_value(&relay_sync).unwrap();
        assert_eq!(json["type"], "relay_config_sync");
        assert_eq!(json["relay_token"], "tok");
    }

    #[test]
    fn test_relay_connect_payload() {
        let connect = BackendToAgent::RelayConnect(RelayConnectPayload {
            server_id: Uuid::new_v4(),
            subdomain: "a3f8b".into(),
            public_port: 25565,
            local_mc_addr: "127.0.0.1:25565".into(),
            loader: None,
        });
        let json = serde_json::to_value(&connect).unwrap();
        assert_eq!(json["type"], "relay_connect");
        assert_eq!(json["subdomain"], "a3f8b");
        assert_eq!(json["public_port"], 25565);
        assert!(json.get("loader").is_none());

        let deserialized: BackendToAgent = serde_json::from_value(json).unwrap();
        match deserialized {
            BackendToAgent::RelayConnect(p) => {
                assert_eq!(p.local_mc_addr, "127.0.0.1:25565");
                assert!(p.loader.is_none());
            }
            other => panic!("expected RelayConnect, got {:?}", other),
        }
    }
}
