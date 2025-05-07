use super::{ThreadConfig, SingleLineBase, HasBaseConfig, BaseConfig, WithPassthrough};
use std::any::Any;
use crate::io::ProgressWriter;
use crate::errors::ModeCreationError;
use anyhow::Result;

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
#[derive(Debug)]
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

    /// Set a custom passthrough writer
    pub fn set_passthrough_writer(&mut self, writer: Box<dyn ProgressWriter + Send + 'static>) -> Result<(), ModeCreationError> {
        self.single_line_base.set_passthrough_writer(writer)
    }

    /// Enable or disable passthrough mode
    pub fn set_passthrough(&mut self, enabled: bool) {
        self.single_line_base.set_passthrough(enabled);
    }
}

impl ThreadConfig for Limited {
    fn lines_to_display(&self) -> usize {
        1
    }

    fn handle_message(&mut self, message: String) -> Vec<String> {
        // If passthrough is enabled, write to the passthrough writer
        if self.single_line_base.has_passthrough() {
            if let Some(writer) = self.single_line_base.get_passthrough_writer_mut() {
                let _ = writer.write_line(&message);
                let _ = ProgressWriter::flush(writer);
            }
        }
        
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
        Box::new(Self {
            single_line_base: SingleLineBase::new(
                self.base_config().get_total_jobs(),
                self.single_line_base.has_passthrough()
            ),
        })
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
    use crate::tests::common::{TestEnv, with_timeout};
    use anyhow::Result;

    #[tokio::test]
    async fn test_limited_mode_error_handling() -> Result<()> {
        // Create display OUTSIDE timeout
        let display = ProgressDisplay::new().await?;
        let mut env = TestEnv::new();
        
        // Run test logic INSIDE timeout
        let _ = with_timeout(async {
            let mut task = display.create_task(ThreadMode::Limited, 1).await?;
            
            // Test error handling
            task.capture_stdout("Test message".to_string()).await?;
            env.writeln("Test message");
            
            display.display().await?;
            env.verify();
            Ok::<(), anyhow::Error>(())
        }, 15).await?;
        
        // Clean up OUTSIDE timeout
        display.stop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_limited_mode_long_lines() -> Result<()> {
        // Create display OUTSIDE timeout
        let display = ProgressDisplay::new().await?;
        let mut env = TestEnv::new();
        
        // Run test logic INSIDE timeout
        let _ = with_timeout(async {
            let mut task = display.create_task(ThreadMode::Limited, 1).await?;
            
            // Test long lines
            let long_line = "A".repeat(200);
            task.capture_stdout(long_line.clone()).await?;
            env.writeln(&long_line);
            
            display.display().await?;
            env.verify();
            Ok::<(), anyhow::Error>(())
        }, 15).await?;
        
        // Clean up OUTSIDE timeout
        display.stop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_limited_mode_special_characters() -> Result<()> {
        // Create display OUTSIDE timeout
        let display = ProgressDisplay::new().await?;
        let mut env = TestEnv::new();
        
        // Run test logic INSIDE timeout
        let _ = with_timeout(async {
            let mut task = display.create_task(ThreadMode::Limited, 1).await?;
            
            // Test special characters
            let special_chars = "Special chars: 🦀 👋 🎉";
            task.capture_stdout(special_chars.to_string()).await?;
            env.writeln(special_chars);
            
            display.display().await?;
            env.verify();
            Ok::<(), anyhow::Error>(())
        }, 15).await?;
        
        // Clean up OUTSIDE timeout
        display.stop().await?;
        Ok(())
    }

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
    async fn test_limited_mode_concurrent_tasks() -> Result<()> {
        // Create display OUTSIDE timeout
        let display = ProgressDisplay::new().await?;
        let _env = TestEnv::new();
        
        // Run test logic INSIDE timeout
        let _ = with_timeout(async {
            let mut handles = Vec::new();
            
            for i in 0..3 {
                let display_ref = display.clone();
                let mut task_env = TestEnv::new();
                
                let handle = tokio::spawn(async move {
                    let mut task = display_ref.create_task(ThreadMode::Limited, 1).await?;
                    let message = format!("Task {} message", i);
                    task.capture_stdout(message.clone()).await?;
                    task_env.writeln(&message);
                    task.join().await?;
                    Ok::<TestEnv, anyhow::Error>(task_env)
                });
                
                handles.push(handle);
            }
            
            // Wait for all tasks to complete and combine their outputs
            let mut final_env = TestEnv::new();
            for handle in handles {
                let task_env = handle.await??;
                let content = task_env.contents();
                if !content.is_empty() {
                    final_env.write(&content);
                }
            }
            
            // Verify final state
            display.display().await?;
            final_env.verify();
            Ok::<(), anyhow::Error>(())
        }, 15).await?;
        
        // Clean up OUTSIDE timeout
        display.stop().await?;
        Ok(())
    }
} 