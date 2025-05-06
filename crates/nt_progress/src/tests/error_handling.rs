use crate::{
    ProgressDisplay, 
    ThreadMode, 
    errors::{ProgressError, ModeCreationError, ErrorContext, ContextExt, format_error_debug}
};
use std::error::Error;
use std::io;
use anyhow::Result;

#[tokio::test]
async fn test_error_context_propagation() {
    // Test that error context is properly propagated through the error chain
    let display = ProgressDisplay::new().await;
    
    // Attempt to create a window with an invalid size
    let result = display.spawn_with_mode(ThreadMode::Window(0), || "test").await;
    
    // The result should be an error
    assert!(result.is_err());
    
    // Convert the error to a string and check for context information
    let error_str = result.unwrap_err().to_string();
    assert!(error_str.contains("ProgressDisplay"));  // Component name
    assert!(error_str.contains("spawning task"));    // Operation name
}

#[tokio::test]
async fn test_error_recovery_window_mode() {
    // Test the recovery strategy in create_thread_config for Window mode
    
    // Create a window with an invalid size (0) using the recovery function
    let mut thread_config = crate::modes::create_thread_config(ThreadMode::Window(0), 1);
    
    // The function should recover and return a valid config
    // This can be verified by checking that it has a reasonable number of lines
    assert!(thread_config.lines_to_display() > 0);
    
    // The message handling should work normally
    let lines = thread_config.handle_message("Test message".to_string());
    assert!(!lines.is_empty());
    assert_eq!(lines[0], "Test message");
}

#[tokio::test]
async fn test_error_recovery_window_with_title_mode() {
    // Test the recovery strategy in create_thread_config for WindowWithTitle mode
    
    // Create a window with an invalid size (1) using the recovery function
    let mut thread_config = crate::modes::create_thread_config(ThreadMode::WindowWithTitle(1), 1);
    
    // The function should recover and return a valid config
    // This can be verified by checking that it has a reasonable number of lines
    assert!(thread_config.lines_to_display() > 0);
    
    // The message handling should work normally
    let lines = thread_config.handle_message("Test message".to_string());
    assert!(!lines.is_empty());
}

#[tokio::test]
async fn test_error_hierarchy() {
    // Test the error type hierarchy and conversions
    
    // Create a ModeCreationError
    let mode_error = ModeCreationError::InvalidWindowSize {
        size: 0,
        min_size: 1,
        mode_name: "TestMode".to_string(),
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