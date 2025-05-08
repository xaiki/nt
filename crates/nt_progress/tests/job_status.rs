use nt_progress::core::job_traits::{FailureHandlingJob, JobStatusTracker};
use nt_progress::core::base_config::JobStatus;
use nt_progress::config::ThreadMode;
use nt_progress::config::Config;

#[test]
fn test_job_status_flow() {
    // Create a configuration with window mode
    let mut config = Config::new(ThreadMode::Window(5), 10).unwrap();
    
    // Test initial status (should be Pending)
    assert_eq!(config.get_status(), JobStatus::Pending);
    
    // Start the job
    config.mark_running();
    assert!(config.is_running());
    
    // Simulate a failure
    config.mark_failed("Test failure");
    assert_eq!(config.get_status(), JobStatus::Failed);
    assert!(config.has_failed());
    assert_eq!(config.get_error_message(), Some("Test failure".to_string()));
    
    // Retry the job
    config.retry();
    assert_eq!(config.get_status(), JobStatus::Retry);
    assert!(config.is_retrying());
    
    // Resume running after retry
    config.mark_running();
    assert!(config.is_running());
    
    // Complete the job
    config.mark_completed();
    assert!(config.is_completed());
    assert_eq!(config.get_status(), JobStatus::Completed);
    assert!(!config.has_failed());
}

#[test]
fn test_retry_limits() {
    // Create a configuration with window mode
    let mut config = Config::new(ThreadMode::Window(5), 10).unwrap();
    
    // Set a retry limit of 2
    config.set_max_retries(2);
    assert_eq!(config.get_max_retries(), 2);
    
    // First failure
    config.mark_failed("First failure");
    assert!(config.has_failed());
    
    // First retry
    config.retry();
    assert_eq!(config.get_retry_count(), 1);
    assert!(!config.has_reached_retry_limit());
    
    // Second failure
    config.mark_failed("Second failure");
    
    // Second retry - should be at limit now
    config.retry();
    assert_eq!(config.get_retry_count(), 2);
    assert!(config.has_reached_retry_limit());
    
    // Reset by marking as completed
    config.mark_completed();
    assert_eq!(config.get_retry_count(), 0);
    assert!(!config.has_reached_retry_limit());
}

#[test]
fn test_status_transitions() {
    // Create a configuration with window mode
    let mut config = Config::new(ThreadMode::Window(5), 10).unwrap();
    
    // Test valid state transitions
    config.mark_running();
    assert_eq!(config.get_status(), JobStatus::Running);
    
    config.mark_failed("Failure during running");
    assert_eq!(config.get_status(), JobStatus::Failed);
    
    config.retry();
    assert_eq!(config.get_status(), JobStatus::Retry);
    
    config.mark_running();
    assert_eq!(config.get_status(), JobStatus::Running);
    
    config.mark_completed();
    assert_eq!(config.get_status(), JobStatus::Completed);
    
    // After completed, failure should still change the status
    config.mark_failed("Failure after completion");
    assert_eq!(config.get_status(), JobStatus::Failed);
}

#[test]
fn test_error_handling() {
    // Create a configuration with window mode
    let mut config = Config::new(ThreadMode::Window(5), 10).unwrap();
    
    // Test error handling
    assert_eq!(config.get_failure_count(), 0);
    
    // First failure
    config.mark_failed("Error 1");
    assert_eq!(config.get_failure_count(), 1);
    assert_eq!(config.get_error_message(), Some("Error 1".to_string()));
    
    // Retrying clears the error message but keeps the failure count
    config.retry();
    assert_eq!(config.get_failure_count(), 1);
    assert_eq!(config.get_error_message(), None);
    
    // New failure increments the count
    config.mark_failed("Error 2");
    assert_eq!(config.get_failure_count(), 2);
    assert_eq!(config.get_error_message(), Some("Error 2".to_string()));
    
    // Mark as succeeded resets everything
    config.mark_succeeded();
    assert_eq!(config.get_failure_count(), 0);
    assert_eq!(config.get_error_message(), None);
} 