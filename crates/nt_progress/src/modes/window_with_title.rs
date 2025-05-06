use super::{ThreadConfig, WindowBase, HasBaseConfig, WithTitle, WithCustomSize, WithEmoji};
use std::any::Any;
use crate::errors::ModeCreationError;
use std::fmt::Debug;

/// Configuration for WindowWithTitle mode
/// 
/// In WindowWithTitle mode, the first line is considered a title
/// and is always displayed, followed by the last N-1 lines.
#[derive(Debug, Clone)]
pub struct WindowWithTitle {
    window_base: WindowBase,
    title: Option<String>,
    emojis: Vec<String>,
}

impl WindowWithTitle {
    /// Creates a new WindowWithTitle mode configuration.
    ///
    /// # Parameters
    /// * `total_jobs` - The total number of jobs to track
    /// * `max_lines` - The maximum number of lines to display, including the title
    ///
    /// # Returns
    /// A Result containing either the new WindowWithTitle or a ModeCreationError
    ///
    /// # Errors
    /// Returns an InvalidWindowSize error if max_lines is less than 2 (need room for title + content)
    pub fn new(total_jobs: usize, max_lines: usize) -> Result<Self, ModeCreationError> {
        if max_lines < 2 {
            return Err(ModeCreationError::InvalidWindowSize {
                size: max_lines,
                min_size: 2,
                mode_name: "WindowWithTitle".to_string(),
            });
        }

        Ok(Self {
            window_base: WindowBase::new(total_jobs, max_lines)?,
            title: None,
            emojis: Vec::new(),
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
    
    /// Render the title with emojis if any are present.
    ///
    /// This method formats the title with any emoji characters that have been
    /// added to the mode. The emojis are prepended to the title.
    ///
    /// # Returns
    /// The formatted title string
    fn render_title(&self) -> String {
        let title = self.title.as_deref().unwrap_or("");
        
        if self.emojis.is_empty() {
            title.to_string()
        } else {
            let emoji_part = self.emojis.join(" ");
            format!("{} {}", emoji_part, title)
        }
    }
}

impl HasBaseConfig for WindowWithTitle {
    fn base_config(&self) -> &super::BaseConfig {
        self.window_base.base_config()
    }
    
    fn base_config_mut(&mut self) -> &mut super::BaseConfig {
        self.window_base.base_config_mut()
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

        // If we have no title and no content, return an empty vector
        if self.title.is_none() && window_lines.is_empty() {
            return Vec::new();
        }

        // Start with the rendered title (including emojis)
        let mut result = vec![self.render_title()];

        // If we have more lines than just the title, add the most recent lines
        // but leave out the oldest ones to make space for the title while 
        // staying within max_lines
        if !window_lines.is_empty() {
            let remaining_lines = display_lines - 1;
            let start_idx = if window_lines.len() > remaining_lines {
                window_lines.len() - remaining_lines
            } else {
                0
            };

            for i in start_idx..window_lines.len() {
                // Skip the first message since it's already shown as title if it matches
                let title = self.title.as_deref().unwrap_or("");
                if window_lines[i] != title {
                    result.push(window_lines[i].clone());
                }
            }
        }

        result
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

impl WithTitle for WindowWithTitle {
    fn set_title(&mut self, title: String) {
        self.title = Some(title);
    }
    
    fn get_title(&self) -> &str {
        self.title.as_deref().unwrap_or("")
    }
}

impl WithCustomSize for WindowWithTitle {
    fn set_max_lines(&mut self, max_lines: usize) -> Result<(), ModeCreationError> {
        if max_lines < 2 {
            return Err(ModeCreationError::InvalidWindowSize {
                size: max_lines,
                min_size: 2,
                mode_name: "WindowWithTitle".to_string(),
            });
        }
        
        // We need to resize the window base
        let result = WindowBase::new(self.base_config().get_total_jobs(), max_lines - 1);
        match result {
            Ok(new_base) => {
                self.window_base = new_base;
                Ok(())
            },
            Err(e) => Err(e),
        }
    }
    
    fn get_max_lines(&self) -> usize {
        self.window_base.max_lines() + 1  // +1 for the title
    }
}

impl WithEmoji for WindowWithTitle {
    fn add_emoji(&mut self, emoji: &str) {
        // Validate that the emoji is not empty
        if !emoji.trim().is_empty() {
            self.emojis.push(emoji.to_string());
        }
    }
    
    fn get_emojis(&self) -> Vec<String> {
        self.emojis.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::common::TestEnv;
    use crate::ProgressDisplay;
    use crate::modes::{ThreadMode, JobTracker};
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