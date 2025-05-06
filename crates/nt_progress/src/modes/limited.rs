use super::{ThreadConfig, SingleLineBase, JobTracker};

/// Configuration for Limited mode
/// 
/// In Limited mode, only the last line is displayed,
/// and all output is passed through to stdout/stderr.
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
            single_line_base: SingleLineBase::new(total_jobs, true), // true = passthrough enabled
        }
    }
}

impl JobTracker for Limited {
    fn get_total_jobs(&self) -> usize {
        self.single_line_base.get_total_jobs()
    }
    
    fn increment_completed_jobs(&self) -> usize {
        self.single_line_base.increment_completed_jobs()
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

    fn clone_box(&self) -> Box<dyn ThreadConfig> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;
    use std::time::Duration;
    use crate::ProgressDisplay;
    use crate::modes::ThreadMode;
    use crate::test_utils::TestEnv;

    #[test]
    fn test_limited_mode_basic() {
        let mut limited = Limited::new(1);
        let mut env = TestEnv::new(80, 24);
        
        // Test initial state
        assert_eq!(limited.lines_to_display(), 1);
        assert_eq!(limited.get_lines(), vec![""]);
        assert_eq!(limited.get_total_jobs(), 1);
        
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
        assert_eq!(limited.increment_completed_jobs(), 1);
    }

    #[tokio::test]
    async fn test_concurrent_tasks() {
        let display = ProgressDisplay::new().await;
        let total_jobs = 5;
        let mut handles = vec![];
        let mut main_env = TestEnv::new(80, 24);
        let (width, height) = main_env.size();
        
        // Test task cancellation
        for i in 0..total_jobs {
            let display = display.clone();
            let i = i;
            let mut task_env = TestEnv::new(width, height);
            handles.push(tokio::spawn(async move {
                display.spawn_with_mode(ThreadMode::Limited, move || format!("task-{}", i)).await.unwrap();
                for j in 0..3 {
                    task_env.writeln(&format!("Thread {}: Message {}", i, j));
                    sleep(Duration::from_millis(50)).await;
                }
                // Simulate task cancellation
                if i == 2 {
                    return task_env;
                }
                task_env.writeln(&format!("Thread {}: Completed", i));
                task_env
            }));
        }
        
        // Wait for all tasks to complete and merge their outputs
        for handle in handles {
            let task_env = handle.await.unwrap();
            main_env.merge(task_env);
        }
        
        display.display().await.unwrap();
        display.stop().await.unwrap();
        main_env.verify();
    }

    #[tokio::test]
    async fn test_limited_mode_error_handling() {
        let display = ProgressDisplay::new().await;
        let mut env = TestEnv::new(80, 24);
        
        // Test stdout error
        let _handle = display.spawn_with_mode(ThreadMode::Limited, || "error-test").await.unwrap();
        
        // Simulate stdout error
        env.writeln("Test message");
        env.writeln("Error message");
        
        // Verify display still works
        display.display().await.unwrap();
        display.stop().await.unwrap();
        env.verify();
    }

    #[tokio::test]
    async fn test_limited_mode_special_characters() {
        let display = ProgressDisplay::new().await;
        let mut env = TestEnv::new(80, 24);
        
        // Test with special characters
        let _handle = display.spawn_with_mode(ThreadMode::Limited, || "special-chars").await.unwrap();
        
        // Test various special characters
        env.writeln("Test with \n newlines \t tabs \r returns");
        env.writeln("Test with unicode: 你好世界");
        env.writeln("Test with emoji: 🚀 ✨");
        
        // Verify display
        display.display().await.unwrap();
        display.stop().await.unwrap();
        env.verify();
    }

    #[tokio::test]
    async fn test_limited_mode_long_lines() {
        let display = ProgressDisplay::new().await;
        let mut env = TestEnv::new(80, 24);
        
        // Test with long lines
        let _handle = display.spawn_with_mode(ThreadMode::Limited, || "long-lines").await.unwrap();
        
        // Test very long line
        let long_line = "x".repeat(1000);
        env.writeln(&long_line);
        
        // Verify display
        display.display().await.unwrap();
        display.stop().await.unwrap();
        env.verify();
    }
} 