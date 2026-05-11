//! Agent Event - Internal pub/sub event bus

use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum AgentEvent {
    TaskReceived {
        task_id: uuid::Uuid,
        task_type: String,
    },
    TaskStarted {
        task_id: uuid::Uuid,
    },
    TaskCompleted {
        task_id: uuid::Uuid,
    },
    TaskFailed {
        task_id: uuid::Uuid,
        error: String,
    },
    TaskCancelled {
        task_id: uuid::Uuid,
    },
    AgentConnected,
    AgentDisconnected {
        reason: String,
    },
    AgentRegistered {
        agent_id: uuid::Uuid,
    },
    ServerStatusChanged {
        server_id: uuid::Uuid,
        status: String,
    },
    ServerLogLine {
        server_id: uuid::Uuid,
        line: String,
        stream: LogStream,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogStream {
    Stdout,
    Stderr,
}

pub struct EventBus {
    sender: broadcast::Sender<AgentEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn publish(&self, event: AgentEvent) {
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AgentEvent> {
        self.sender.subscribe()
    }

    pub fn sender(&self) -> broadcast::Sender<AgentEvent> {
        self.sender.clone()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_bus_creation() {
        let bus = EventBus::new(100);
        assert!(bus.subscribe().try_recv().is_err());
    }

    #[tokio::test]
    async fn test_event_publish() {
        let bus = EventBus::new(100);
        let mut sub = bus.subscribe();
        
        bus.publish(AgentEvent::AgentConnected);
        
        let event = sub.recv().await;
        assert!(matches!(event, Ok(AgentEvent::AgentConnected)));
    }
}
