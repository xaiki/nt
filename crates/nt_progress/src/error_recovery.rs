use std::time::Duration;
use tokio::time::sleep;
use anyhow::Result;
use crate::errors::{ProgressError, ErrorSeverity};

/// Retry configuration for operations
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Delay between retries
    pub retry_delay: Duration,
    /// Whether to use exponential backoff
    pub use_exponential_backoff: bool,
    /// Base delay for exponential backoff
    pub base_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
            use_exponential_backoff: true,
            base_delay: Duration::from_millis(100),
        }
    }
}

/// Execute an operation with retry capability
pub async fn with_retry<F, T, E>(config: &RetryConfig, mut operation: F) -> Result<T>
where
    F: FnMut() -> Result<T, E>,
    E: Into<ProgressError>,
{
    let mut attempts = 0;
    let mut current_delay = config.base_delay;

    loop {
        match operation() {
            Ok(result) => return Ok(result),
            Err(error) => {
                let error = error.into();
                
                // Check if we should retry
                if attempts >= config.max_retries || !error.is_retryable() {
                    return Err(anyhow::anyhow!(error));
                }

                // Calculate next delay
                if config.use_exponential_backoff {
                    current_delay *= 2;
                } else {
                    current_delay = config.retry_delay;
                }

                // Wait before retrying
                sleep(current_delay).await;
                attempts += 1;
            }
        }
    }
}

/// Fallback configuration for operations
#[derive(Debug, Clone)]
pub struct FallbackConfig {
    /// Whether to use fallback mode
    pub use_fallback: bool,
    /// Whether to log fallback usage
    pub log_fallback: bool,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            use_fallback: true,
            log_fallback: true,
        }
    }
}

/// Execute an operation with fallback capability
pub async fn with_fallback<F, G, T>(config: &FallbackConfig, primary: F, fallback: G) -> Result<T>
where
    F: Fn() -> Result<T, anyhow::Error>,
    G: FnOnce(anyhow::Error) -> Result<T>,
{
    match primary() {
        Ok(result) => Ok(result),
        Err(error) => {
            if !config.use_fallback {
                return Err(error);
            }

            if config.log_fallback {
                eprintln!("Primary operation failed, using fallback: {}", error);
            }

            fallback(error)
        }
    }
}

/// Error recovery strategies
pub struct ErrorRecovery {
    /// Retry configuration
    pub retry_config: RetryConfig,
    /// Fallback configuration
    pub fallback_config: FallbackConfig,
}

impl Default for ErrorRecovery {
    fn default() -> Self {
        Self {
            retry_config: RetryConfig::default(),
            fallback_config: FallbackConfig::default(),
        }
    }
}

impl ErrorRecovery {
    /// Create a new error recovery instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Execute an operation with both retry and fallback capabilities
    pub async fn execute<F, G, T>(&self, mut primary: F, fallback: G) -> Result<T>
    where
        F: FnMut() -> Result<T, ProgressError>,
        G: FnOnce(anyhow::Error) -> Result<T>,
    {
        // First try with retry
        match with_retry(&self.retry_config, &mut primary).await {
            Ok(result) => Ok(result),
            Err(error) => {
                // If retry fails, try fallback
                with_fallback(&self.fallback_config, || Err(anyhow::anyhow!("primary failed")), |_| fallback(error)).await
            }
        }
    }

    /// Handle an error with appropriate recovery strategy
    pub async fn handle_error(&self, error: ProgressError) -> Result<()> {
        match error.severity() {
            ErrorSeverity::Low => {
                // For low severity errors, just log and continue
                eprintln!("Low severity error: {}", error);
                Ok(())
            }
            ErrorSeverity::Medium => {
                // For medium severity errors, try recovery if possible
                if let Some(hint) = error.recovery_hint() {
                    eprintln!("Medium severity error: {}\nRecovery hint: {}", error, hint);
                }
                Ok(())
            }
            ErrorSeverity::High => {
                // For high severity errors, attempt recovery with retry
                if error.is_retryable() {
                    let retry_error = error.into_retryable(
                        self.retry_config.max_retries,
                        self.retry_config.retry_delay,
                    );
                    Err(anyhow::anyhow!(retry_error))
                } else {
                    Err(anyhow::anyhow!(error))
                }
            }
            ErrorSeverity::Fatal => {
                // For fatal errors, no recovery is possible
                Err(anyhow::anyhow!(error))
            }
        }
    }
} 