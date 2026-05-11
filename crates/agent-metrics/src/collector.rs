//! Metrics collector

use std::time::Duration;
use tokio::sync::broadcast;

use super::SystemMetrics;

pub struct MetricsCollector {
    sender: broadcast::Sender<SystemMetrics>,
    interval_secs: u64,
}

impl MetricsCollector {
    pub fn new(interval_secs: u64) -> Self {
        let (sender, _) = broadcast::channel(100);
        Self { sender, interval_secs }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SystemMetrics> {
        self.sender.subscribe()
    }

    pub fn get_metrics(&self) -> SystemMetrics {
        collect_system_metrics()
    }

    pub async fn run(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(self.interval_secs));
        
        loop {
            interval.tick().await;
            let metrics = self.get_metrics();
            let _ = self.sender.send(metrics);
        }
    }
}

pub fn collect_system_metrics() -> SystemMetrics {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();
    
    let cpu_percent = sys.global_cpu_usage();
    
    let memory_used = sys.used_memory();
    let memory_total = sys.total_memory();
    
    let disks = sysinfo::Disks::new_with_refreshed_list();
    let mut disk_usage = Vec::new();
    for disk in disks.list() {
        disk_usage.push(super::DiskUsage {
            mount_point: disk.mount_point().to_string_lossy().to_string(),
            used_bytes: disk.total_space() - disk.available_space(),
            total_bytes: disk.total_space(),
        });
    }
    
    let networks = sysinfo::Networks::new_with_refreshed_list();
    let mut network_rx = 0u64;
    let mut network_tx = 0u64;
    for (_name, data) in networks.list() {
        network_rx += data.total_received();
        network_tx += data.total_transmitted();
    }
    
    SystemMetrics {
        timestamp: chrono::Utc::now(),
        cpu_percent,
        memory_used_bytes: memory_used,
        memory_total_bytes: memory_total,
        disk_usage,
        network_rx_bytes: network_rx,
        network_tx_bytes: network_tx,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_metrics() {
        let metrics = collect_system_metrics();
        
        assert!(metrics.cpu_percent >= 0.0);
        assert!(metrics.memory_total_bytes > 0);
    }
}
