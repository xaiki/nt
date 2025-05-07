// Module for test helper functions that can be used in both unit and integration tests
use std::future::Future;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Helper function to run an async test with a timeout
/// 
/// This function is useful for preventing tests from hanging indefinitely,
/// particularly those involving event loops or message passing.
/// 
/// When the timeout is reached, it will panic with a message indicating the timeout,
/// which should abort any ongoing operations within the test.
pub async fn with_timeout<F, T>(test_fn: F, timeout_secs: u64) -> std::result::Result<T, std::io::Error>
where
    F: Future<Output = T>,
{
    // Guard against very long timeouts in tests
    let timeout_secs = timeout_secs.min(60); // Max 60 seconds for any test
    let timeout_duration = Duration::from_secs(timeout_secs);
    
    // Create flag for timeout tracking
    let timeout_occurred = Arc::new(AtomicBool::new(false));
    let timeout_flag = Arc::clone(&timeout_occurred);
    
    // Spawn a separate task for the timeout to ensure it can complete
    // even if the main task is blocked
    let timeout_handle = tokio::spawn(async move {
        tokio::time::sleep(timeout_duration).await;
        timeout_flag.store(true, Ordering::SeqCst);
        let msg = format!("TEST TIMEOUT: Test exceeded {} seconds limit", timeout_secs);
        eprintln!("\n\n⚠️ {}\n⚠️ THIS IS LIKELY CAUSING TEST HANGS\n⚠️ Check for resource leaks or deadlocks\n", msg);
    });
    
    // Create a timeout future with tokio::time::timeout
    match tokio::time::timeout(timeout_duration, test_fn).await {
        Ok(result) => {
            // Cancel the timeout task since we completed successfully
            timeout_handle.abort();
            Ok(result)
        },
        Err(_) => {
            // If we get here, the timeout was reached before the test completed
            let msg = format!("Test timed out after {} seconds", timeout_secs);
            if !timeout_occurred.load(Ordering::SeqCst) {
                // Make sure our timeout flag is set
                timeout_occurred.store(true, Ordering::SeqCst);
            }
            eprintln!("\n\n🛑 TIMEOUT ERROR: {}\n🛑 Test execution will now abort\n", msg);
            
            // Just panic instead of exiting process to allow other tests to run
            panic!("{}", msg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_with_timeout_success() {
        // This test should complete successfully
        let result = with_timeout(async {
            sleep(Duration::from_millis(10)).await;
            42
        }, 1).await;
        
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    #[should_panic(expected = "Test timed out after 1 seconds")]
    async fn test_with_timeout_expiration() {
        // This test should time out and panic
        let _ = with_timeout(async {
            sleep(Duration::from_secs(2)).await; // Sleep longer than timeout
            42
        }, 1).await;
    }
} 