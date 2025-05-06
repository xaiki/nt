use super::{ThreadConfig, WindowBase, JobTracker};
use std::any::Any;

/// Configuration for WindowWithTitle mode
/// 
/// In WindowWithTitle mode, the first line is considered a title
/// and is always displayed, followed by the last N-1 lines.
#[derive(Debug, Clone)]
pub struct WindowWithTitle {
    window_base: WindowBase,
    title: Option<String>,
}

impl WindowWithTitle {
    pub fn new(total_jobs: usize, max_lines: usize) -> Result<Self, String> {
        if max_lines < 2 {
            return Err("Max lines for WindowWithTitle must be at least 2".to_string());
        }

        Ok(Self {
            window_base: WindowBase::new(total_jobs, max_lines)?,
            title: None,
        })
    }
    
    /// Set or update the title of the window.
    ///
    /// This method allows changing the title of the window after it has been created.
    /// If no title has been set yet, this will set the initial title.
    ///
    /// # Parameters
    /// * `new_title` - The new title to set
    pub fn set_title(&mut self, new_title: String) {
        self.title = Some(new_title);
    }
}

impl JobTracker for WindowWithTitle {
    fn get_total_jobs(&self) -> usize {
        self.window_base.get_total_jobs()
    }
    
    fn increment_completed_jobs(&self) -> usize {
        self.window_base.increment_completed_jobs()
    }
}

impl ThreadConfig for WindowWithTitle {
    fn lines_to_display(&self) -> usize {
        self.window_base.max_lines()
    }

    fn handle_message(&mut self, message: String) -> Vec<String> {
        if self.title.is_none() {
            self.title = Some(message.clone());
        }

        self.window_base.add_message(message);
        self.get_lines()
    }

    fn get_lines(&self) -> Vec<String> {
        let window_lines = self.window_base.get_lines();
        let display_lines = self.window_base.max_lines();

        if let Some(title) = &self.title {
            // Start with the title
            let mut result = vec![title.clone()];

            // If we have more lines than just the title, add the most recent lines
            // but leave out the oldest ones to make space for the title while 
            // staying within max_lines
            if window_lines.len() > 0 {
                let remaining_lines = display_lines - 1;
                let start_idx = if window_lines.len() > remaining_lines {
                    window_lines.len() - remaining_lines
                } else {
                    0
                };

                for i in start_idx..window_lines.len() {
                    // Skip the first message since it's already shown as title
                    if window_lines[i] != *title {
                        result.push(window_lines[i].clone());
                    }
                }
            }

            result
        } else {
            window_lines
        }
    }

    fn clone_box(&self) -> Box<dyn ThreadConfig> {
        Box::new(self.clone())
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::common::TestEnv;
    use crate::ProgressDisplay;
    use crate::modes::ThreadMode;
    use tokio::time::sleep;
    use std::time::Duration;

    #[test]
    fn test_window_with_title_mode_basic() {
        let mut window = WindowWithTitle::new(1, 4).unwrap();
        let mut env = TestEnv::new(80, 24);
        
        // Test initial state
        assert_eq!(window.lines_to_display(), 4);
        assert_eq!(window.get_lines(), Vec::<String>::new());
        assert_eq!(window.get_total_jobs(), 1);
        
        // Test adding title
        env.writeln("Title Line");
        window.handle_message("Title Line".to_string());
        assert_eq!(window.get_lines(), vec!["Title Line"]);
        env.verify();
        
        // Test adding content lines
        env.writeln("line 1");
        window.handle_message("line 1".to_string());
        assert_eq!(window.get_lines(), vec!["Title Line", "line 1"]);
        env.verify();
        
        env.writeln("line 2");
        window.handle_message("line 2".to_string());
        assert_eq!(window.get_lines(), vec!["Title Line", "line 1", "line 2"]);
        env.verify();
        
        env.writeln("line 3");
        window.handle_message("line 3".to_string());
        assert_eq!(window.get_lines(), vec!["Title Line", "line 1", "line 2", "line 3"]);
        env.verify();
        
        // Test exceeding max_lines - should keep title and most recent lines
        env.writeln("line 4");
        window.handle_message("line 4".to_string());
        assert_eq!(window.get_lines(), vec!["Title Line", "line 2", "line 3", "line 4"]);
        env.verify();
        
        // Test completed jobs
        assert_eq!(window.increment_completed_jobs(), 1);
    }

    #[test]
    fn test_window_with_title_mode_invalid_size() {
        assert!(WindowWithTitle::new(1, 1).is_err());
    }

    #[tokio::test]
    async fn test_window_with_title_mode_concurrent() {
        let display = ProgressDisplay::new().await;
        let mut handles = vec![];
        
        // Spawn multiple tasks in WindowWithTitle mode
        for i in 0..3 {
            let display = display.clone();
            let mut env = TestEnv::new(80, 24);
            let i = i;
            handles.push(tokio::spawn(async move {
                display.spawn_with_mode(ThreadMode::WindowWithTitle(4), move || format!("Task {} Title", i)).await.unwrap();
                for j in 0..5 {
                    env.writeln(&format!("Thread {}: Message {}", i, j));
                    sleep(Duration::from_millis(50)).await;
                }
                env
            }));
        }
        
        // Wait for all tasks to complete and combine their outputs
        let mut final_env = TestEnv::new(80, 24);
        for handle in handles {
            let task_env = handle.await.unwrap();
            for line in task_env.expected {
                final_env.write(&line);
            }
        }
        
        // Verify final state
        display.display().await.unwrap();
        display.stop().await.unwrap();
        final_env.verify();
    }

    #[tokio::test]
    async fn test_window_with_title_mode_special_characters() {
        let display = ProgressDisplay::new().await;
        let mut env = TestEnv::new(80, 24);
        
        // Test with special characters
        let _handle = display.spawn_with_mode(ThreadMode::WindowWithTitle(4), || "Special Chars Title").await.unwrap();
        
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
    async fn test_window_with_title_mode_long_lines() {
        let display = ProgressDisplay::new().await;
        let mut env = TestEnv::new(80, 24);
        
        // Test with long lines
        let _handle = display.spawn_with_mode(ThreadMode::WindowWithTitle(4), || "Long Lines Title").await.unwrap();
        
        // Test very long line
        let long_line = "x".repeat(1000);
        env.writeln(&long_line);
        
        // Verify display
        display.display().await.unwrap();
        display.stop().await.unwrap();
        env.verify();
    }
} 