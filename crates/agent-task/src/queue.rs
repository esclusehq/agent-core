//! Task queue with priority support

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;

use agent_proto::{Task, TaskStatus};
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct QueuedTask {
    pub task: Task,
    pub status: TaskStatus,
    pub enqueued_at: chrono::DateTime<Utc>,
    pub started_at: Option<chrono::DateTime<Utc>>,
}

impl PartialEq for QueuedTask {
    fn eq(&self, other: &Self) -> bool {
        self.task.priority == other.task.priority && self.task.id == other.task.id
    }
}

impl Eq for QueuedTask {}

impl PartialOrd for QueuedTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueuedTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first (High=2 > Low=0)
        // BinaryHeap is a max-heap, so we want highest priority to compare as "greater"
        self.task.priority.cmp(&other.task.priority)
    }
}

pub struct TaskQueue {
    queue: Arc<std::sync::Mutex<BinaryHeap<QueuedTask>>>,
    by_id: Arc<std::sync::Mutex<HashMap<Uuid, QueuedTask>>>,
}

impl TaskQueue {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(std::sync::Mutex::new(BinaryHeap::new())),
            by_id: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    pub fn enqueue(&self, task: Task) {
        let queued = QueuedTask {
            task: task.clone(),
            status: TaskStatus::Pending,
            enqueued_at: Utc::now(),
            started_at: None,
        };

        {
            let mut queue = self.queue.lock().unwrap();
            queue.push(queued.clone());
        }

        {
            let mut by_id = self.by_id.lock().unwrap();
            by_id.insert(task.id, queued);
        }
    }

    pub fn dequeue(&self) -> Option<Task> {
        let mut queue = self.queue.lock().unwrap();

        if let Some(queued) = queue.pop() {
            let mut by_id = self.by_id.lock().unwrap();
            if let Some(q) = by_id.get_mut(&queued.task.id) {
                q.status = TaskStatus::Running;
                q.started_at = Some(Utc::now());
            }
            return Some(queued.task);
        }

        None
    }

    pub fn get(&self, task_id: Uuid) -> Option<QueuedTask> {
        let by_id = self.by_id.lock().unwrap();
        by_id.get(&task_id).cloned()
    }

    pub fn update_status(&self, task_id: Uuid, status: TaskStatus) {
        let mut by_id = self.by_id.lock().unwrap();
        if let Some(queued) = by_id.get_mut(&task_id) {
            queued.status = status;
        }
    }

    pub fn remove(&self, task_id: Uuid) -> Option<QueuedTask> {
        let mut by_id = self.by_id.lock().unwrap();
        by_id.remove(&task_id)
    }

    pub fn len(&self) -> usize {
        let queue = self.queue.lock().unwrap();
        queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get_all_pending(&self) -> Vec<QueuedTask> {
        let by_id = self.by_id.lock().unwrap();
        by_id
            .values()
            .filter(|q| q.status == TaskStatus::Pending)
            .cloned()
            .collect()
    }
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enqueue_dequeue() {
        use agent_proto::TaskPriority;

        let queue = TaskQueue::new();

        let task1 = Task::new("server.start".to_string(), serde_json::json!({}));
        let task2 = Task::new("server.stop".to_string(), serde_json::json!({}))
            .with_priority(TaskPriority::High);

        queue.enqueue(task1.clone());
        queue.enqueue(task2.clone());

        assert_eq!(queue.len(), 2);

        // First dequeued should be task2 (high priority)
        let dequeued = queue.dequeue().unwrap();
        assert_eq!(dequeued.priority, TaskPriority::High);
    }

    #[test]
    fn test_priority_ordering() {
        use agent_proto::TaskPriority;

        let queue = TaskQueue::new();

        let low =
            Task::new("a".to_string(), serde_json::json!({})).with_priority(TaskPriority::Low);
        let high =
            Task::new("b".to_string(), serde_json::json!({})).with_priority(TaskPriority::High);

        queue.enqueue(low);
        queue.enqueue(high);

        // First dequeued should be high priority
        let first = queue.dequeue().unwrap();
        assert_eq!(first.priority, TaskPriority::High);
    }
}
