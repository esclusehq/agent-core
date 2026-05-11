//! Task dispatcher with concurrency control

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Semaphore};
use tokio::time::timeout;

use agent_proto::{Task, TaskError, TaskResult};

use crate::TaskQueue;

pub struct TaskDispatcher {
    queue: Arc<TaskQueue>,
    semaphore: Arc<Semaphore>,
    results_tx: mpsc::Sender<TaskResult>,
}

impl TaskDispatcher {
    pub fn new(queue: Arc<TaskQueue>, max_concurrent: usize, results_tx: mpsc::Sender<TaskResult>) -> Self {
        Self {
            queue,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            results_tx,
        }
    }

    pub async fn run<F, Fut>(&self, handler: F) where
        F: Fn(Task) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<serde_json::Value, TaskError>> + Send,
    {
        loop {
            let _permit = self.semaphore.acquire().await.unwrap();
            
            let task = match self.queue.dequeue() {
                Some(t) => t,
                None => continue,
            };
            
            let task_id = task.id;
            let started_at = chrono::Utc::now();
            
            let result = timeout(
                Duration::from_secs(task.timeout_secs),
                handler(task)
            ).await;
            
            let task_result = match result {
                Ok(Ok(output)) => {
                    TaskResult::completed(task_id, output, started_at)
                }
                Ok(Err(error)) => {
                    TaskResult::failed(task_id, error, started_at)
                }
                Err(_) => {
                    TaskResult::timed_out(task_id, started_at)
                }
            };
            
            let _ = self.results_tx.send(task_result).await;
        }
    }
}

pub async fn dispatch_task<F, Fut>(
    task: Task,
    handler: F,
    timeout_secs: u64,
) -> TaskResult
where
    F: Fn(Task) -> Fut,
    Fut: std::future::Future<Output = Result<serde_json::Value, TaskError>>,
{
    let started_at = chrono::Utc::now();
    let task_id = task.id;
    
    match timeout(Duration::from_secs(timeout_secs), handler(task)).await {
        Ok(Ok(output)) => TaskResult::completed(task_id, output, started_at),
        Ok(Err(error)) => TaskResult::failed(task_id, error, started_at),
        Err(_) => TaskResult::timed_out(task_id, started_at),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_proto::TaskError;

    #[tokio::test]
    async fn test_dispatch_success() {
        let task = Task::new("test".to_string(), serde_json::json!({}));
        
        let result = dispatch_task(
            task,
            |_| async { Ok(serde_json::json!({"status": "ok"})) },
            5,
        ).await;
        
        assert!(result.output.is_some());
    }

    #[tokio::test]
    async fn test_dispatch_failure() {
        let task = Task::new("test".to_string(), serde_json::json!({}));
        
        let result = dispatch_task(
            task,
            |_| async { 
                Err(TaskError::new("FAILED", "test error", false)) 
            },
            5,
        ).await;
        
        assert!(result.error.is_some());
    }
}
