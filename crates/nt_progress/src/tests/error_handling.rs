use crate::{
    config::capabilities::{WithEmoji, WithTitle},
    errors::{ProgressError, ModeCreationError, ErrorContext, ContextExt, format_error_debug}
};
use std::error::Error;
use std::io;
use anyhow::Result;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modes::window_with_title::WindowWithTitle;

    #[tokio::test]
    async fn test_error_context_propagation() -> Result<(), Box<dyn std::error::Error>> {
        let mut mode = WindowWithTitle::new(1, 3, "Test".to_string())?;
        mode.set_title_support(false);
        
        let result = mode.set_title("New Title".to_string());
        assert!(result.is_err());
        
        if let Err(e) = result {
            match e {
                ModeCreationError::TitleNotSupported { mode_name, reason } => {
                    assert_eq!(mode_name, "WindowWithTitle");
                    assert!(reason.is_some());
                    assert!(reason.unwrap().contains("Title support is disabled for this mode"));
                }
                _ => panic!("Expected TitleNotSupported error"),
            }
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_error_recovery_window_mode() -> Result<(), Box<dyn std::error::Error>> {
        let result = WindowWithTitle::new(1, 1, "Test".to_string());
        assert!(result.is_err());
        
        if let Err(e) = result {
            match e {
                ModeCreationError::InvalidWindowSize { size, min_size, mode_name, reason } => {
                    assert_eq!(size, 1);
                    assert_eq!(min_size, 2);
                    assert_eq!(mode_name, "WindowWithTitle");
                    assert!(reason.is_some());
                    assert!(reason.unwrap().contains("WindowWithTitle requires at least 2 lines: 1 for title and 1 for content"));
                }
                _ => panic!("Expected InvalidWindowSize error"),
            }
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_error_recovery_emoji() -> Result<(), Box<dyn std::error::Error>> {
        let mut mode = WindowWithTitle::new(1, 3, "Test".to_string())?;
        mode.set_emoji_support(false);
        
        let result = mode.add_emoji("🚀");
        assert!(result.is_err());
        
        if let Err(e) = result {
            match e {
                ModeCreationError::EmojiNotSupported { mode_name, reason } => {
                    assert_eq!(mode_name, "WindowWithTitle");
                    assert!(reason.is_some());
                    assert!(reason.unwrap().contains("Emoji support is disabled for this mode"));
                }
                _ => panic!("Expected EmojiNotSupported error"),
            }
        }
        
        Ok(())
    }
}

#[tokio::test]
async fn test_error_hierarchy() {
    // Test the error type hierarchy and conversions
    
    // Create a ModeCreationError
    let mode_error = ModeCreationError::InvalidWindowSize {
        size: 0,
        min_size: 1,
        mode_name: "TestMode".to_string(),
        reason: Some("Test reason".to_string()),
    };
    
    // Convert to ProgressError
    let progress_error: ProgressError = mode_error.into();
    
    // Verify the error type
    match &progress_error {
        ProgressError::ModeCreation(_) => {}, // This is expected
        _ => panic!("Incorrect error type conversion"),
    }
    
    // Test error source chain
    let source = progress_error.source();
    assert!(source.is_some());
}

#[tokio::test]
async fn test_context_ext_trait() {
    // Test the ContextExt trait for adding context to errors
    
    // Create a simple error result
    let result: Result<(), ProgressError> = Err(ProgressError::TaskOperation("Test error".to_string()));
    
    // Add context using the trait
    let ctx = ErrorContext::new("test operation", "test component")
        .with_thread_id(42)
        .with_details("test details");
    
    let result_with_context = result.context(ctx);
    
    // Verify the error contains context
    let err = result_with_context.unwrap_err();
    let err_str = err.to_string();
    
    assert!(err_str.contains("test operation"));
    assert!(err_str.contains("test component"));
    assert!(err_str.contains("thread 42"));
    assert!(err_str.contains("test details"));
}

#[tokio::test]
async fn test_error_logging_utilities() {
    // Test the error logging utilities
    
    // Create a nested error with context
    let inner_error = ProgressError::TaskOperation("Inner error".to_string());
    let ctx = ErrorContext::new("inner operation", "inner component");
    let mid_error = inner_error.into_context(ctx);
    
    let outer_ctx = ErrorContext::new("outer operation", "outer component");
    let outer_error = mid_error.into_context(outer_ctx);
    
    // Generate debug output
    let debug_str = format_error_debug(&outer_error);
    
    // Verify the debug output contains all error chain information
    assert!(debug_str.contains("Error chain"));
    assert!(debug_str.contains("inner operation"));
    assert!(debug_str.contains("outer operation"));
    assert!(debug_str.contains("Inner error"));
}

#[tokio::test]
async fn test_io_error_conversion() {
    // Test conversion from io::Error to ProgressError
    
    // Create an IO error
    let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    
    // Convert to ProgressError
    let progress_error: ProgressError = io_error.into();
    
    // Verify the error type
    match &progress_error {
        ProgressError::Io(_) => {}, // This is expected
        _ => panic!("Incorrect error type conversion"),
    }
    
    // Test error message
    assert!(progress_error.to_string().contains("File not found"));
}

#[tokio::test]
async fn test_error_recovery_retry() {
    use crate::error_recovery::{RetryConfig, with_retry};
    use std::time::Duration;

    let config = RetryConfig {
        max_retries: 3,
        retry_delay: Duration::from_millis(100),
        use_exponential_backoff: false,
        base_delay: Duration::from_millis(100),
    };

    let mut attempts = 0;
    let result = with_retry(&config, || {
        attempts += 1;
        if attempts < 3 {
            Err(ProgressError::TaskOperation("Temporary error".to_string()).into_retryable(3, Duration::from_millis(100)))
        } else {
            Ok(())
        }
    }).await;

    assert!(result.is_ok());
    assert_eq!(attempts, 3);
}

#[tokio::test]
async fn test_error_recovery_fallback() {
    use crate::error_recovery::{FallbackConfig, with_fallback};

    let config = FallbackConfig {
        use_fallback: true,
        log_fallback: false,
    };

    let result = with_fallback(
        &config,
        || Err(anyhow::anyhow!("Primary operation failed")),
        |_| Ok("Fallback result"),
    ).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Fallback result");
}

#[tokio::test]
async fn test_error_recovery_strategies() {
    use crate::error_recovery::ErrorRecovery;
    use crate::errors::ErrorSeverity;

    let recovery = ErrorRecovery::new();

    // Test low severity error
    let low_error = ProgressError::DisplayOperation("Minor display issue".to_string())
        .into_context(ErrorContext::new("test", "test").with_severity(ErrorSeverity::Low));
    let result = recovery.handle_error(low_error).await;
    assert!(result.is_ok());

    // Test medium severity error with recovery hint
    let medium_error = ProgressError::TaskOperation("Task issue".to_string())
        .into_context(
            ErrorContext::new("test", "test")
                .with_severity(ErrorSeverity::Medium)
                .with_recovery_hint("Try restarting the task")
        );
    let result = recovery.handle_error(medium_error).await;
    assert!(result.is_ok());

    // Test high severity retryable error
    let high_error = ProgressError::TaskOperation("Retryable error".to_string())
        .into_context(ErrorContext::new("test", "test").with_severity(ErrorSeverity::High));
    let result = recovery.handle_error(high_error).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Retryable error"));

    // Test fatal error
    let fatal_error = ProgressError::TaskOperation("Fatal error".to_string())
        .into_context(ErrorContext::new("test", "test").with_severity(ErrorSeverity::Fatal));
    let result = recovery.handle_error(fatal_error).await;
    assert!(result.is_err());
} 