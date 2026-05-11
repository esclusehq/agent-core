//! Retry logic with exponential backoff

use std::time::Duration;
use tokio::time::sleep;

// D-13: Trait for classifying errors as transient (retryable) or permanent
/// Trait for classifying errors as transient (retryable) or permanent
pub trait IsTransient {
    /// Returns true if error is transient and should be retried
    fn is_transient(&self) -> bool;
}

/// Common transient errors in web-agent
#[derive(Debug)]
pub enum TransientError {
    NetworkError(String),
    TimeoutError(String),
    ConnectionRefused,
    ServiceUnavailable,
}

impl IsTransient for TransientError {
    fn is_transient(&self) -> bool {
        match self {
            TransientError::NetworkError(_) => true,
            TransientError::TimeoutError(_) => true,
            TransientError::ConnectionRefused => true,
            TransientError::ServiceUnavailable => true,
        }
    }
}

impl std::fmt::Display for TransientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransientError::NetworkError(s) => write!(f, "NetworkError: {}", s),
            TransientError::TimeoutError(s) => write!(f, "TimeoutError: {}", s),
            TransientError::ConnectionRefused => write!(f, "ConnectionRefused"),
            TransientError::ServiceUnavailable => write!(f, "ServiceUnavailable"),
        }
    }
}

// D-13: Generic version for any error type that implements IsTransient
/// Retry with transient-only classification - retries only transient errors
pub async fn retry_transient_only<E, Fut, T>(
    config: &RetryConfig,
    operation: impl Fn(u32) -> Fut,
) -> Result<T, RetryError<E>>
where
    E: IsTransient + std::fmt::Debug,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut delay = config.initial_delay;

    for attempt in 1..=config.max_attempts {
        match operation(attempt).await {
            Ok(value) => return Ok(value),
            Err(error) => {
                // D-13: Only retry on transient failures
                if !error.is_transient() {
                    return Err(RetryError::Permanent { error });
                }

                if attempt == config.max_attempts {
                    return Err(RetryError::Exhausted {
                        attempts: attempt,
                        last_error: error,
                    });
                }

                let sleep_duration = if config.jitter {
                    apply_jitter(delay)
                } else {
                    delay
                };

                sleep(sleep_duration).await;
                delay = Duration::from_secs_f64(delay.as_secs_f64() * config.multiplier)
                    .min(config.max_delay);
            }
        }
    }

    unreachable!()
}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryConfig {
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            ..Default::default()
        }
    }

    pub fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    pub fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = multiplier;
        self
    }

    pub fn with_jitter(mut self, jitter: bool) -> Self {
        self.jitter = jitter;
        self
    }
}

pub async fn retry_with_backoff<F, Fut, T, E>(
    config: &RetryConfig,
    operation: F,
) -> Result<T, RetryError<E>>
where
    F: Fn(u32) -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut delay = config.initial_delay;
    
    for attempt in 1..=config.max_attempts {
        match operation(attempt).await {
            Ok(value) => return Ok(value),
            Err(error) => {
                if attempt == config.max_attempts {
                    return Err(RetryError::Exhausted {
                        attempts: attempt,
                        last_error: error,
                    });
                }
                
                let sleep_duration = if config.jitter {
                    apply_jitter(delay)
                } else {
                    delay
                };
                
                sleep(sleep_duration).await;
                delay = Duration::from_secs_f64(delay.as_secs_f64() * config.multiplier)
                    .min(config.max_delay);
            }
        }
    }
    
    unreachable!()
}

fn apply_jitter(delay: Duration) -> Duration {
    use std::time::Instant;
    
    let base = delay.as_secs_f64();
    let jitter = base * 0.4;
    let random = (Instant::now().elapsed().as_nanos() % 1000) as f64 / 1000.0;
    let adjusted = base - jitter + (jitter * 2.0 * random);
    
    Duration::from_secs_f64(adjusted.max(0.0))
}

#[derive(Debug)]
pub enum RetryError<E> {
    Exhausted {
        attempts: u32,
        last_error: E,
    },
    // D-13: Non-retryable error
    Permanent { error: E },
}

impl<E: std::fmt::Debug> std::fmt::Display for RetryError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RetryError::Exhausted { attempts, last_error } => {
                write!(f, "retry exhausted after {} attempts: {:?}", attempts, last_error)
            }
            RetryError::Permanent { error } => {
                write!(f, "permanent error (not retryable): {:?}", error)
            }
        }
    }
}

impl<E: std::fmt::Debug> std::error::Error for RetryError<E> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retry_success_first_try() {
        let config = RetryConfig::new(3);
        let result = retry_with_backoff(&config, |_| async {
            Ok::<_, ()>("success")
        }).await;
        
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        let config = RetryConfig::new(3).with_initial_delay(Duration::from_millis(10));
        let result = retry_with_backoff(&config, |attempt| async move {
            Err::<(), _>(attempt)
        }).await;
        
        assert!(matches!(result, Err(RetryError::Exhausted { attempts: 3, .. })));
    }
}
