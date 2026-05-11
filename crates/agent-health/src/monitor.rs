//! Health monitoring for agent components

use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct HealthCheck {
    pub name: String,
    pub status: HealthStatus,
    pub last_check: std::time::Instant,
    pub message: Option<String>,
}

pub struct HealthMonitor {
    checks: Arc<std::sync::Mutex<std::collections::HashMap<String, HealthCheck>>>,
    tx: broadcast::Sender<HealthEvent>,
}

#[derive(Debug, Clone)]
pub enum HealthEvent {
    CheckStarted { name: String },
    CheckCompleted { name: String, status: HealthStatus },
    StatusChanged { name: String, old: HealthStatus, new: HealthStatus },
}

impl HealthMonitor {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            checks: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            tx,
        }
    }

    pub fn register_check(&self, name: impl Into<String>) {
        let name = name.into();
        let mut checks = self.checks.lock().unwrap();
        checks.insert(
            name.clone(),
            HealthCheck {
                name: name.clone(),
                status: HealthStatus::Unknown,
                last_check: std::time::Instant::now(),
                message: None,
            },
        );
        let _ = self.tx.send(HealthEvent::CheckStarted { name });
    }

    pub fn update_status(&self, name: &str, status: HealthStatus, message: Option<String>) {
        let mut checks = self.checks.lock().unwrap();
        
        if let Some(check) = checks.get_mut(name) {
            let old = check.status;
            check.status = status;
            check.last_check = std::time::Instant::now();
            check.message = message;

            if old != status {
                let _ = self.tx.send(HealthEvent::StatusChanged {
                    name: name.to_string(),
                    old,
                    new: status,
                });
            }
        }
    }

    pub fn get_status(&self, name: &str) -> Option<HealthStatus> {
        let checks = self.checks.lock().unwrap();
        checks.get(name).map(|c| c.status)
    }

    pub fn get_all_statuses(&self) -> std::collections::HashMap<String, HealthStatus> {
        let checks = self.checks.lock().unwrap();
        checks.iter()
            .map(|(k, v)| (k.clone(), v.status))
            .collect()
    }

    pub fn is_healthy(&self) -> bool {
        let checks = self.checks.lock().unwrap();
        checks.values().all(|c| c.status == HealthStatus::Healthy)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<HealthEvent> {
        self.tx.subscribe()
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_monitor() {
        let monitor = HealthMonitor::new();
        
        monitor.register_check("test");
        assert_eq!(monitor.get_status("test"), Some(HealthStatus::Unknown));
        
        monitor.update_status("test", HealthStatus::Healthy, Some("OK".to_string()));
        assert_eq!(monitor.get_status("test"), Some(HealthStatus::Healthy));
        
        assert!(monitor.is_healthy());
    }
}
