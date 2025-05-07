use super::{ThreadConfig, WindowBase, HasBaseConfig, WithCustomSize, StandardWindow};
use super::capabilities::WithWrappedText;
use std::any::Any;
use crate::errors::ModeCreationError;
use std::fmt::Debug;

/// Configuration for Window mode
/// 
/// In Window mode, the last N lines are displayed,
/// where N is specified by the user and will be adjusted
/// if it doesn't fit the terminal.
#[derive(Debug, Clone)]
pub struct Window {
    window_base: WindowBase,
}

impl Window {
    /// Creates a new Window mode configuration.
    ///
    /// # Parameters
    /// * `total_jobs` - The total number of jobs to track
    /// * `max_lines` - The maximum number of lines to display
    ///
    /// # Returns
    /// A Result containing either the new Window or a ModeCreationError
    ///
    /// # Errors
    /// Returns an error if max_lines is invalid (e.g., zero)
    pub fn new(total_jobs: usize, max_lines: usize) -> Result<Self, ModeCreationError> {
        Ok(Self {
            window_base: WindowBase::new(total_jobs, max_lines)?,
        })
    }
}

impl HasBaseConfig for Window {
    fn base_config(&self) -> &super::BaseConfig {
        self.window_base.base_config()
    }
    
    fn base_config_mut(&mut self) -> &mut super::BaseConfig {
        self.window_base.base_config_mut()
    }
}

impl ThreadConfig for Window {
    fn lines_to_display(&self) -> usize {
        self.window_base.max_lines()
    }

    fn handle_message(&mut self, message: String) -> Vec<String> {
        self.window_base.add_message(message);
        self.window_base.get_lines()
    }

    fn get_lines(&self) -> Vec<String> {
        self.window_base.get_lines()
    }

    fn clone_box(&self) -> Box<dyn ThreadConfig> {
        Box::new(self.clone())
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl WithCustomSize for Window {
    fn set_max_lines(&mut self, max_lines: usize) -> Result<(), ModeCreationError> {
        if max_lines < 1 {
            return Err(ModeCreationError::InvalidWindowSize {
                size: max_lines,
                min_size: 1,
                mode_name: "Window".to_string(),
                reason: Some("Window mode requires at least 1 line to display content".to_string()),
            });
        }
        
        // We need to recreate the window base
        let result = WindowBase::new(self.base_config().get_total_jobs(), max_lines);
        match result {
            Ok(new_base) => {
                self.window_base = new_base;
                Ok(())
            },
            Err(e) => Err(e),
        }
    }
    
    fn get_max_lines(&self) -> usize {
        self.window_base.max_lines()
    }
}

impl StandardWindow for Window {
    fn clear(&mut self) {
        // Clear all content from the window
        self.window_base.clear();
    }
    
    fn get_content(&self) -> Vec<String> {
        // Get the current content as a vector of strings
        self.window_base.get_lines()
    }
    
    fn add_line(&mut self, line: String) {
        // Add a single line to the window
        self.window_base.add_message(line);
    }
    
    fn is_empty(&self) -> bool {
        // Check if the window is empty
        self.window_base.is_empty()
    }
    
    fn line_count(&self) -> usize {
        // Get the number of lines currently displayed
        self.window_base.line_count()
    }
}

impl WithWrappedText for Window {
    fn set_line_wrapping(&mut self, enabled: bool) {
        self.window_base.set_line_wrapping(enabled);
    }
    
    fn has_line_wrapping(&self) -> bool {
        self.window_base.has_line_wrapping()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::TestEnv;
    use crate::ProgressDisplay;
    use crate::modes::{ThreadMode, JobTracker};
    use tokio::time::sleep;
    use std::time::Duration;
    use crate::tests::common::with_timeout;
    use anyhow::Result;

    #[test]
    fn test_window_mode_basic() {
        let mut window = Window::new(1, 3).unwrap();
        let mut env = TestEnv::new_with_size(80, 24);
        
        // Test initial state
        assert_eq!(window.lines_to_display(), 3);
        assert_eq!(window.get_lines(), Vec::<String>::new());
        assert_eq!(window.get_total_jobs(), 1);
        
        // Test adding lines up to max_lines
        env.writeln("line 1");
        window.handle_message("line 1".to_string());
        assert_eq!(window.get_lines(), vec!["line 1"]);
        env.verify();
        
        env.writeln("line 2");
        window.handle_message("line 2".to_string());
        assert_eq!(window.get_lines(), vec!["line 1", "line 2"]);
        env.verify();
        
        env.writeln("line 3");
        window.handle_message("line 3".to_string());
        assert_eq!(window.get_lines(), vec!["line 1", "line 2", "line 3"]);
        env.verify();
        
        // Test exceeding max_lines
        env.writeln("line 4");
        window.handle_message("line 4".to_string());
        assert_eq!(window.get_lines(), vec!["line 2", "line 3", "line 4"]);
        env.verify();
        
        // Test completed jobs
        assert_eq!(window.increment_completed_jobs(), 1);
    }

    #[test]
    fn test_window_mode_invalid_size() {
        assert!(Window::new(1, 0).is_err());
    }

    #[tokio::test]
    async fn test_window_mode_concurrent() -> Result<()> {
        // Create display OUTSIDE timeout
        let display = ProgressDisplay::new().await?;
        let _env = TestEnv::new();
        
        // Run test logic INSIDE timeout
        let _ = with_timeout(async {
            let total_jobs = 3;
            let mut handles = vec![];
            
            // Spawn multiple tasks in Window mode
            for i in 0..total_jobs {
                let display_ref = display.clone();
                let mut task_env = TestEnv::new();
                let i = i;
                handles.push(tokio::spawn(async move {
                    let mut task = display_ref.spawn_with_mode(ThreadMode::Window(3), move || format!("task-{}", i)).await?;
                    for j in 0..5 {
                        let message = format!("Thread {}: Message {}", i, j);
                        task.capture_stdout(message.clone()).await?;
                        task_env.writeln(&message);
                        sleep(Duration::from_millis(50)).await;
                    }
                    Ok::<TestEnv, anyhow::Error>(task_env)
                }));
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

    #[tokio::test]
    async fn test_window_mode_special_characters() -> Result<()> {
        // Create display OUTSIDE timeout
        let display = ProgressDisplay::new().await?;
        let mut env = TestEnv::new();
        
        // Run test logic INSIDE timeout
        let _ = with_timeout(async {
            let mut task = display.spawn_with_mode(ThreadMode::Window(3), || "special-chars").await?;
            
            // Test various special characters
            let special_chars = vec![
                "Test with \n newlines \t tabs \r returns",
                "Test with unicode: 你好世界",
                "Test with emoji: 🚀 ✨"
            ];
            
            for chars in special_chars {
                task.capture_stdout(chars.to_string()).await?;
                env.writeln(chars);
            }
            
            display.display().await?;
            env.verify();
            Ok::<(), anyhow::Error>(())
        }, 15).await?;
        
        // Clean up OUTSIDE timeout
        display.stop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_window_mode_long_lines() -> Result<()> {
        // Create display OUTSIDE timeout
        let display = ProgressDisplay::new().await?;
        let mut env = TestEnv::new();
        
        // Run test logic INSIDE timeout
        let _ = with_timeout(async {
            let mut task = display.spawn_with_mode(ThreadMode::Window(3), || "long-lines").await?;
            
            // Test very long line
            let long_line = "x".repeat(1000);
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
    #[ignore = "Replaced by direct test"]
    async fn test_window_mode_line_wrapping() -> Result<()> {
        // Create display OUTSIDE timeout
        let display = ProgressDisplay::new().await?;
        let _env = TestEnv::new_with_size(40, 24); // Set a small width to test wrapping
        
        // Run test logic INSIDE timeout
        let _ = with_timeout(async {
            let mut task = display.spawn_with_mode(ThreadMode::Window(5), || "wrapping-test").await?;
            
            // Configure line wrapping
            {
                let mut config = task.thread_config.lock().await;
                if let Some(window) = config.as_type_mut::<Window>() {
                    window.set_line_wrapping(true);
                }
            }
            
            // Test long line that should wrap - use a very long line to ensure wrapping
            let long_line = "This is an extremely long line that will definitely exceed any reasonable terminal width and force line wrapping to occur. It contains many words and characters to ensure that it will be wrapped into multiple lines by our line wrapping implementation. This should result in several lines of output rather than just one very long line that would extend beyond the edge of the terminal.";
            task.capture_stdout(long_line.to_string()).await?;
            
            // In the wrapped version, this should produce multiple lines
            display.display().await?;
            
            // Get the lines from the window
            let lines = {
                let config = task.thread_config.lock().await;
                config.get_lines()
            };
            
            // Verify we get multiple lines from the single input
            assert!(lines.len() > 1, "Line wrapping should have created multiple lines");
            
            // Check that the full text is preserved across the wrapped lines
            let combined = lines.join("");
            assert!(combined.contains("This is an extremely long line"), "Original content should be preserved");
            
            // Disable line wrapping and test again
            {
                let mut config = task.thread_config.lock().await;
                if let Some(window) = config.as_type_mut::<Window>() {
                    window.set_line_wrapping(false);
                }
            }
            
            // Should now be a single line again
            let short_line = "Another line that won't be wrapped now";
            task.capture_stdout(short_line.to_string()).await?;
            display.display().await?;
            
            // Get the lines from the window
            let lines = {
                let config = task.thread_config.lock().await;
                config.get_lines()
            };
            
            // This time we should have the text as a single line
            assert!(lines.contains(&short_line.to_string()), 
                   "With wrapping disabled, line should be preserved as-is");
            
            Ok::<(), anyhow::Error>(())
        }, 30).await?;
        
        // Clean up OUTSIDE timeout
        display.stop().await?;
        Ok(())
    }

    #[test]
    fn test_window_direct_line_wrapping() {
        // Create window instance directly
        let mut window = Window::new(10, 5).unwrap();
        
        // Enable line wrapping
        window.set_line_wrapping(true);
        
        // Add a very long line that should wrap
        let long_line = "This is an extremely long line that will definitely exceed any reasonable terminal width and force line wrapping to occur.";
        window.handle_message(long_line.to_string());
        
        // Check that we have multiple lines
        let lines = window.get_lines();
        assert!(lines.len() > 1, "Line wrapping should have created multiple lines");
        
        // Check content is preserved
        let combined = lines.join("");
        assert!(combined.contains("This is an extremely long line"), 
               "Original content should be preserved");
        
        // Disable line wrapping
        window.set_line_wrapping(false);
        
        // Add another line that would be wrapped if wrapping was still enabled
        window.handle_message("Another long line that would be wrapped if wrapping was enabled".to_string());
        
        // Check that the new line was added as a single line
        let lines = window.get_lines();
        assert!(lines.contains(&"Another long line that would be wrapped if wrapping was enabled".to_string()),
               "With wrapping disabled, line should be preserved as-is");
    }
} 