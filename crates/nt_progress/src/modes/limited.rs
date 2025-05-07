use super::{ThreadConfig, SingleLineBase, HasBaseConfig, BaseConfig};
use std::any::Any;

/// Configuration for Limited mode
/// 
/// In Limited mode, messages are passed through to stdout/stderr,
/// but only the most recent message is kept for display.
///
/// # Features
///
/// - Displays only the last message received
/// - Requires only a single line of display space
/// - Passes output through to stdout/stderr
///
/// # Example
///
/// ```
/// use nt_progress::modes::{ThreadConfig, Limited};
///
/// let mut limited = Limited::new(1);
/// let lines = limited.handle_message("test message".to_string());
/// assert_eq!(lines, vec!["test message"]);
/// ```
#[derive(Debug, Clone)]
pub struct Limited {
    single_line_base: SingleLineBase,
}

impl Limited {
    /// Create a new Limited mode configuration.
    ///
    /// # Parameters
    /// * `total_jobs` - The total number of jobs to track
    ///
    /// # Returns
    /// A new Limited instance
    pub fn new(total_jobs: usize) -> Self {
        Self {
            single_line_base: SingleLineBase::new(total_jobs, true),
        }
    }
}

impl ThreadConfig for Limited {
    fn lines_to_display(&self) -> usize {
        1
    }

    fn handle_message(&mut self, message: String) -> Vec<String> {
        self.single_line_base.update_line(message);
        self.get_lines()
    }

    fn get_lines(&self) -> Vec<String> {
        vec![self.single_line_base.get_line()]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn ThreadConfig> {
        Box::new(self.clone())
    }
}

impl HasBaseConfig for Limited {
    fn base_config(&self) -> &BaseConfig {
        self.single_line_base.base_config()
    }

    fn base_config_mut(&mut self) -> &mut BaseConfig {
        self.single_line_base.base_config_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProgressDisplay;
    use crate::modes::ThreadMode;
    use crate::terminal::TestEnv;
    use tokio::time::sleep;
    use std::time::Duration;
    use crate::tests::common::with_timeout;
    use anyhow::Result;

    #[test]
    fn test_limited_mode_basic() {
        let mut limited = Limited::new(1);
        let mut env = TestEnv::new_with_size(80, 24);
        
        // Test initial state
        assert_eq!(limited.lines_to_display(), 1);
        assert_eq!(limited.get_lines(), vec![""]);
        assert_eq!(limited.base_config().get_total_jobs(), 1);
        
        // Test message handling
        env.writeln("test message");
        let lines = limited.handle_message("test message".to_string());
        assert_eq!(lines, vec!["test message"]);
        env.verify();
        
        // Test multiple messages
        env.writeln("new message");
        limited.handle_message("new message".to_string());
        assert_eq!(limited.get_lines(), vec!["new message"]);
        env.verify();
        
        // Test completed jobs
        assert_eq!(limited.base_config_mut().increment_completed_jobs(), 1);
    }

    #[tokio::test]
    async fn test_concurrent_tasks() -> Result<()> {
        // Create display outside timeout
        let display = ProgressDisplay::new().await?;
        let total_jobs = 5;
        let mut main_env = TestEnv::new_with_size(80, 24);
        let (width, height) = main_env.size();
        
        // Run test within timeout
        let _ = with_timeout(async {
            let mut handles = vec![];
            
            // Test task cancellation
            for i in 0..total_jobs {
                let display = display.clone();
                let i = i;
                let mut task_env = TestEnv::new_with_size(width, height);
                handles.push(tokio::spawn(async move {
                    let mut task = display.spawn_with_mode(ThreadMode::Limited, move || format!("task-{}", i)).await?;
                    for j in 0..3 {
                        let message = format!("Thread {}: Message {}", i, j);
                        task.capture_stdout(message.clone()).await?;
                        task_env.writeln(&message);
                        sleep(Duration::from_millis(50)).await;
                    }
                    // Simulate task cancellation
                    if i == 2 {
                        return Ok::<_, anyhow::Error>(task_env);
                    }
                    let message = format!("Thread {}: Completed", i);
                    task.capture_stdout(message.clone()).await?;
                    task_env.writeln(&message);
                    Ok(task_env)
                }));
            }
            
            // Wait for all tasks to complete and merge their outputs
            for handle in handles {
                let task_env = handle.await??;
                main_env.merge(task_env);
            }
            
            display.display().await?;
            Ok::<_, anyhow::Error>(())
        }, 5).await?;
        
        // Always clean up outside timeout
        display.stop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_limited_mode_error_handling() {
        let display = ProgressDisplay::new().await;
        let mut env = TestEnv::new_with_size(80, 24);
        
        // Test stdout error
        let _handle = display.as_ref().expect("Failed to get display").spawn_with_mode(ThreadMode::Limited, || "error-test").await.unwrap();
        
        // Simulate stdout error
        env.writeln("Test message");
        env.writeln("Error message");
        
        // Verify display still works
        display.as_ref().expect("Failed to get display").display().await.unwrap();
        display.as_ref().expect("Failed to get display").stop().await.unwrap();
        env.verify();
    }

    #[tokio::test]
    async fn test_limited_mode_special_characters() {
        let display = ProgressDisplay::new().await;
        let mut env = TestEnv::new_with_size(80, 24);
        
        // Test with special characters
        let _handle = display.as_ref().expect("Failed to get display").spawn_with_mode(ThreadMode::Limited, || "special-chars").await.unwrap();
        
        // Test various special characters
        env.writeln("Test with \n newlines \t tabs \r returns");
        env.writeln("Test with unicode: 你好世界");
        env.writeln("Test with emoji: 🚀 ✨");
        
        // Verify display
        display.as_ref().expect("Failed to get display").display().await.unwrap();
        display.as_ref().expect("Failed to get display").stop().await.unwrap();
        env.verify();
    }

    #[tokio::test]
    async fn test_limited_mode_long_lines() {
        let display = ProgressDisplay::new().await;
        let mut env = TestEnv::new_with_size(80, 24);
        
        // Test with long lines
        let _handle = display.as_ref().expect("Failed to get display").spawn_with_mode(ThreadMode::Limited, || "long-lines").await.unwrap();
        
        // Test very long line
        let long_line = "x".repeat(1000);
        env.writeln(&long_line);
        
        // Verify display
        display.as_ref().expect("Failed to get display").display().await.unwrap();
        display.as_ref().expect("Failed to get display").stop().await.unwrap();
        env.verify();
    }
} 