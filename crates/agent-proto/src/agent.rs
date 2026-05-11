//! Agent types for agent information and status

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub agent_id: Uuid,
    pub agent_name: String,
    pub version: String,
    pub platform: String,
    pub capabilities: Vec<String>,
    pub runtime: Option<String>,
    pub status: AgentState,
    pub registered_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

impl AgentInfo {
    pub fn new(agent_name: String, capabilities: Vec<String>) -> Self {
        Self {
            agent_id: Uuid::new_v4(),
            agent_name,
            version: env!("CARGO_PKG_VERSION").to_string(),
            platform: std::env::consts::OS.to_string(),
            capabilities,
            runtime: None,
            status: AgentState::Pending,
            registered_at: Utc::now(),
            last_seen: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentState {
    Pending,
    Registered,
    Connected,
    Disconnected,
    Error,
}

impl Default for AgentState {
    fn default() -> Self {
        AgentState::Pending
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    pub agent_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub task_count: u32,
    pub status: AgentState,
}

impl Heartbeat {
    pub fn new(agent_id: Uuid, task_count: u32, status: AgentState) -> Self {
        Self {
            agent_id,
            timestamp: Utc::now(),
            task_count,
            status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_info_creation() {
        let info = AgentInfo::new("test-agent".to_string(), vec!["docker".to_string()]);
        assert_eq!(info.agent_name, "test-agent");
        assert_eq!(info.status, AgentState::Pending);
    }

    #[test]
    fn test_heartbeat() {
        let agent_id = Uuid::new_v4();
        let heartbeat = Heartbeat::new(agent_id, 5, AgentState::Connected);
        assert_eq!(heartbeat.agent_id, agent_id);
        assert_eq!(heartbeat.task_count, 5);
    }
}
