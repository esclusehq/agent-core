//! Local audit logging

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub event: AuditEvent,
    pub task_id: Option<Uuid>,
    pub task_type: Option<String>,
    pub result: Option<AuditResult>,
    pub agent_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEvent {
    TaskReceived,
    TaskStarted,
    TaskCompleted,
    TaskFailed,
    TaskRejected { reason: String },
    AgentRegistered,
    AgentDisconnected { reason: String },
    AgentShutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Failed { error: String },
}

pub struct AuditLogger {
    data_dir: PathBuf,
}

impl AuditLogger {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    pub fn log(&self, entry: AuditEntry) -> Result<(), AuditError> {
        let file_path = self.get_log_path();

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .map_err(|e| AuditError::IoError(e.to_string()))?;

        let json =
            serde_json::to_string(&entry).map_err(|e| AuditError::Serialization(e.to_string()))?;

        writeln!(file, "{}", json).map_err(|e| AuditError::IoError(e.to_string()))?;

        Ok(())
    }

    fn get_log_path(&self) -> PathBuf {
        let date = Utc::now().format("%Y-%m-%d");
        let path = self
            .data_dir
            .join("audit")
            .join(format!("audit-{}.log", date));

        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        path
    }

    pub fn log_task_received(&self, agent_id: Uuid, task_id: Uuid, task_type: &str) {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            event: AuditEvent::TaskReceived,
            task_id: Some(task_id),
            task_type: Some(task_type.to_string()),
            result: None,
            agent_id,
        };
        let _ = self.log(entry);
    }

    pub fn log_task_completed(&self, agent_id: Uuid, task_id: Uuid) {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            event: AuditEvent::TaskCompleted,
            task_id: Some(task_id),
            task_type: None,
            result: Some(AuditResult::Success),
            agent_id,
        };
        let _ = self.log(entry);
    }

    pub fn log_task_failed(&self, agent_id: Uuid, task_id: Uuid, error: &str) {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            event: AuditEvent::TaskFailed,
            task_id: Some(task_id),
            task_type: None,
            result: Some(AuditResult::Failed {
                error: error.to_string(),
            }),
            agent_id,
        };
        let _ = self.log(entry);
    }

    pub fn log_agent_registered(&self, agent_id: Uuid) {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            event: AuditEvent::AgentRegistered,
            task_id: None,
            task_type: None,
            result: None,
            agent_id,
        };
        let _ = self.log(entry);
    }
}

#[derive(Debug)]
pub enum AuditError {
    IoError(String),
    Serialization(String),
}

impl std::fmt::Display for AuditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditError::IoError(msg) => write!(f, "IO error: {}", msg),
            AuditError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for AuditError {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_audit_logger() {
        let temp_dir = TempDir::new().unwrap();
        let logger = AuditLogger::new(temp_dir.path().to_path_buf());

        logger.log_agent_registered(Uuid::new_v4());

        let log_path = temp_dir
            .path()
            .join("audit")
            .join(format!("audit-{}.log", Utc::now().format("%Y-%m-%d")));
        assert!(log_path.exists());
    }
}
