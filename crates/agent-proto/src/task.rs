//! Task types for agent task management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub task_type: String,
    pub payload: serde_json::Value,
    pub priority: TaskPriority,
    pub timeout_secs: u64,
    pub depends_on: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub idempotency_key: Option<String>,
    pub authorization: String,
}

impl Task {
    pub fn new(task_type: String, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            task_type,
            payload,
            priority: TaskPriority::Normal,
            timeout_secs: 300,
            depends_on: Vec::new(),
            created_at: Utc::now(),
            idempotency_key: None,
            authorization: String::new(),
        }
    }

    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    pub fn with_id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }

    pub fn with_authorization(mut self, authorization: String) -> Self {
        self.authorization = authorization;
        self
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Normal
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: Uuid,
    pub status: TaskStatus,
    pub output: Option<serde_json::Value>,
    pub error: Option<TaskError>,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub retry_count: u32,
}

impl TaskResult {
    pub fn completed(task_id: Uuid, output: serde_json::Value, started_at: DateTime<Utc>) -> Self {
        Self {
            task_id,
            status: TaskStatus::Completed,
            output: Some(output),
            error: None,
            started_at,
            ended_at: Utc::now(),
            retry_count: 0,
        }
    }

    pub fn failed(task_id: Uuid, error: TaskError, started_at: DateTime<Utc>) -> Self {
        Self {
            task_id,
            status: TaskStatus::Failed,
            output: None,
            error: Some(error),
            started_at,
            ended_at: Utc::now(),
            retry_count: 0,
        }
    }

    pub fn timed_out(task_id: Uuid, started_at: DateTime<Utc>) -> Self {
        Self {
            task_id,
            status: TaskStatus::TimedOut,
            output: None,
            error: Some(TaskError {
                code: "TIMEOUT".to_string(),
                message: "Task timed out".to_string(),
                retryable: true,
                details: None,
            }),
            started_at,
            ended_at: Utc::now(),
            retry_count: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
    pub details: Option<serde_json::Value>,
}

impl TaskError {
    pub fn new(code: impl Into<String>, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            retryable,
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

impl std::fmt::Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for TaskError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new(
            "server.start".to_string(),
            serde_json::json!({ "server_id": "test" }),
        );
        assert_eq!(task.task_type, "server.start");
        assert_eq!(task.priority, TaskPriority::Normal);
    }

    #[test]
    fn test_task_result_completed() {
        let started = Utc::now();
        let result = TaskResult::completed(
            Uuid::new_v4(),
            serde_json::json!({ "container_id": "abc" }),
            started,
        );
        assert_eq!(result.status, TaskStatus::Completed);
        assert!(result.output.is_some());
    }
}
