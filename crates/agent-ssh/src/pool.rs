//! SSH connection pool

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use super::SshClient;

pub struct SshPool {
    clients: Arc<RwLock<HashMap<String, SshClient>>>,
    max_idle: usize,
    #[allow(dead_code)]
    max_age: Duration,
}

impl SshPool {
    pub fn new(max_idle: usize, max_age_seconds: u64) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            max_idle,
            max_age: Duration::from_secs(max_age_seconds),
        }
    }

    pub async fn get_client(&self, key: &str) -> Option<SshClient> {
        let clients = self.clients.read().await;
        clients.get(key).cloned()
    }

    pub async fn put_client(&self, key: String, client: SshClient) {
        let mut clients = self.clients.write().await;
        
        // Check if we need to evict old clients
        if clients.len() >= self.max_idle {
            // Simple eviction: remove oldest (in a real impl, track access times)
            if let Some(first_key) = clients.keys().next().cloned() {
                clients.remove(&first_key);
            }
        }
        
        clients.insert(key, client);
    }

    pub async fn remove_client(&self, key: &str) {
        let mut clients = self.clients.write().await;
        clients.remove(key);
    }

    pub async fn clear(&self) {
        let mut clients = self.clients.write().await;
        clients.clear();
    }

    pub async fn len(&self) -> usize {
        let clients = self.clients.read().await;
        clients.len()
    }
}

impl Default for SshPool {
    fn default() -> Self {
        Self::new(10, 3600)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_creation() {
        let pool = SshPool::new(5, 600);
        assert_eq!(pool.len().await, 0);
    }
}
