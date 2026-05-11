//! System metrics types

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub cpu_percent: f32,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub disk_usage: Vec<DiskUsage>,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
}

impl SystemMetrics {
    pub fn memory_percent(&self) -> f32 {
        if self.memory_total_bytes == 0 {
            return 0.0;
        }
        (self.memory_used_bytes as f32 / self.memory_total_bytes as f32) * 100.0
    }

    pub fn disk_percent(&self, mount_point: &str) -> f32 {
        self.disk_usage
            .iter()
            .find(|d| d.mount_point == mount_point)
            .map(|d| {
                if d.total_bytes == 0 {
                    return 0.0;
                }
                (d.used_bytes as f32 / d.total_bytes as f32) * 100.0
            })
            .unwrap_or(0.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskUsage {
    pub mount_point: String,
    pub used_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMetrics {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub status: ProcessStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProcessStatus {
    Running,
    Sleeping,
    Stopped,
    Zombie,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            cpu_percent: 0.0,
            memory_used_bytes: 0,
            memory_total_bytes: 0,
            disk_usage: Vec::new(),
            network_rx_bytes: 0,
            network_tx_bytes: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_percent() {
        let metrics = SystemMetrics {
            timestamp: chrono::Utc::now(),
            cpu_percent: 50.0,
            memory_used_bytes: 4_000_000_000,
            memory_total_bytes: 8_000_000_000,
            disk_usage: vec![],
            network_rx_bytes: 0,
            network_tx_bytes: 0,
        };

        assert_eq!(metrics.memory_percent(), 50.0);
    }
}
