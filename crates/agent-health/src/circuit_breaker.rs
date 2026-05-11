//! Circuit breaker implementation

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    failure_count: Arc<AtomicU32>,
    success_count: Arc<AtomicU32>,
    failure_threshold: u32,
    success_threshold: u32,
    open_duration: Duration,
    last_opened_at: Arc<Mutex<Option<Instant>>>,
}

impl CircuitBreaker {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            failure_count: Arc::new(AtomicU32::new(0)),
            success_count: Arc::new(AtomicU32::new(0)),
            failure_threshold: 5,
            success_threshold: 2,
            open_duration: Duration::from_secs(30),
            last_opened_at: Arc::new(Mutex::new(None)),
        }
    }

    pub fn with_failure_threshold(mut self, threshold: u32) -> Self {
        self.failure_threshold = threshold;
        self
    }

    pub fn with_success_threshold(mut self, threshold: u32) -> Self {
        self.success_threshold = threshold;
        self
    }

    pub fn with_open_duration(mut self, duration: Duration) -> Self {
        self.open_duration = duration;
        self
    }

    pub fn state(&self) -> CircuitState {
        let mut state = self.state.lock().unwrap();
        if *state == CircuitState::Open {
            let last_opened = self.last_opened_at.lock().unwrap();
            if let Some(opened_at) = *last_opened {
                if opened_at.elapsed() >= self.open_duration {
                    *state = CircuitState::HalfOpen;
                }
            }
        }
        *state
    }

    pub fn is_available(&self) -> bool {
        self.state() != CircuitState::Open
    }

    pub fn record_success(&self) {
        let _ = self.failure_count.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(0));
        
        if self.state() == CircuitState::HalfOpen {
            let count = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
            if count >= self.success_threshold {
                let mut state = self.state.lock().unwrap();
                *state = CircuitState::Closed;
                let _ = self.success_count.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(0));
            }
        }
    }

    pub fn record_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        
        if count >= self.failure_threshold {
            let mut state = self.state.lock().unwrap();
            *state = CircuitState::Open;
            let _ = self.last_opened_at.lock().unwrap().insert(Instant::now());
        }
    }

    pub async fn call<F, Fut, T, E>(&self, operation: F) -> Result<T, CircuitError<E>>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
    {
        if !self.is_available() {
            return Err(CircuitError::Open);
        }

        match operation().await {
            Ok(result) => {
                self.record_success();
                Ok(result)
            }
            Err(error) => {
                self.record_failure();
                Err(CircuitError::Inner(error))
            }
        }
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum CircuitError<E> {
    Open,
    Inner(E),
}

impl<E: std::fmt::Debug> std::fmt::Display for CircuitError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitError::Open => write!(f, "circuit breaker is open"),
            CircuitError::Inner(e) => write!(f, "operation failed: {:?}", e),
        }
    }
}

impl<E: std::fmt::Debug + std::fmt::Display> std::error::Error for CircuitError<E> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_closed() {
        let cb = CircuitBreaker::new();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.is_available());
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_on_failures() {
        let cb = CircuitBreaker::new().with_failure_threshold(3);
        
        for _ in 0..3 {
            cb.record_failure();
        }
        
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.is_available());
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open() {
        let cb = CircuitBreaker::new()
            .with_failure_threshold(2)
            .with_open_duration(Duration::from_millis(50));
        
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        assert_eq!(cb.state(), CircuitState::HalfOpen);
    }
}
