use crate::core::{ThreadConfig, HasBaseConfig, BaseConfig};
use super::window_base::WindowBase;
use crate::config::capabilities::{WithTitle, WithCustomSize, WithEmoji, WithTitleAndEmoji, StandardWindow, WithWrappedText, WithProgress};
use crate::core::job_traits::JobTracker;
use std::any::Any;
use crate::errors::ModeCreationError;
use std::fmt::Debug;
use anyhow::Result;

/// Configuration for WindowWithTitle mode
/// 
/// In WindowWithTitle mode, the first line is considered a title
/// and is always displayed, followed by the last N-1 lines.
#[derive(Debug, Clone)]
pub struct WindowWithTitle {
    window_base: WindowBase,
    title: String,
    emojis: Vec<String>,
    supports_emoji: bool,
    supports_title: bool,
}

impl WindowWithTitle {
    /// Creates a new WindowWithTitle mode configuration.
    ///
    /// # Parameters
    /// * `total_jobs` - The total number of jobs to track
    /// * `max_lines` - The maximum number of lines to display, including the title
    /// * `title` - The title of the window
    ///
    /// # Returns
    /// A Result containing either the new WindowWithTitle or a ModeCreationError
    ///
    /// # Errors
    /// Returns an InvalidWindowSize error if max_lines is less than 2 (need room for title + content)
    pub fn new(total_jobs: usize, max_lines: usize, title: String) -> Result<Self, ModeCreationError> {
        if max_lines < 2 {
            return Err(ModeCreationError::InvalidWindowSize {
                size: max_lines,
                min_size: 2,
                mode_name: "WindowWithTitle".to_string(),
                reason: Some("WindowWithTitle requires at least 2 lines: 1 for title and 1 for content".to_string()),
            });
        }
        
        // Ensure title is not empty
        let title = if title.is_empty() {
            "Progress".to_string()
        } else {
            title
        };
        
        Ok(Self {
            window_base: WindowBase::new(total_jobs, max_lines - 1)?,
            title,
            emojis: Vec::new(),
            supports_emoji: true,  // Enable emoji support by default
            supports_title: true,  // Enable title support by default
        })
    }
    
    /// Render the title with emojis if any are present.
    ///
    /// This method formats the title with any emoji characters that have been
    /// added to the mode. The emojis are prepended to the title.
    ///
    /// # Returns
    /// The formatted title string
    fn render_title(&self, width: usize) -> String {
        if width == 0 {
            return String::new();
        }

        let mut title = self.title.clone();
        
        // Add emojis if supported and present
        if self.supports_emoji && !self.emojis.is_empty() {
            let emoji_str = self.emojis.join(" ");
            title = format!("{} {}", emoji_str, title);
        }
        
        let title_width = title.chars().count();
        
        // If title is too long, truncate it
        if title_width > width {
            let mut chars = title.chars().collect::<Vec<_>>();
            let truncate_width = width.saturating_sub(3);
            chars.truncate(truncate_width);
            
            // Ensure we don't break in the middle of an emoji
            while !chars.is_empty() && chars.last().unwrap().is_ascii_control() {
                chars.pop();
            }
            
            title = chars.into_iter().collect::<String>();
            title.push_str("...");
        }
        
        title
    }
    
    /// Enable or disable title support.
    ///
    /// # Parameters
    /// * `enabled` - Whether to enable or disable title support
    pub fn set_title_support(&mut self, enabled: bool) {
        self.supports_title = enabled;
    }

    /// Enable or disable emoji support.
    ///
    /// # Parameters
    /// * `enabled` - Whether to enable or disable emoji support
    pub fn set_emoji_support(&mut self, enabled: bool) {
        self.supports_emoji = enabled;
    }

    /// Check if title support is enabled.
    ///
    /// # Returns
    /// true if title support is enabled, false otherwise
    pub fn has_title_support(&self) -> bool {
        self.supports_title
    }

    /// Check if emoji support is enabled.
    ///
    /// # Returns
    /// true if emoji support is enabled, false otherwise
    pub fn has_emoji_support(&self) -> bool {
        self.supports_emoji
    }

    pub fn render(&self, width: usize) -> String {
        let mut output = String::new();
        
        // Add title line
        output.push_str(&self.render_title(width));
        output.push('\n');
        
        // Add content lines
        output.push_str(&self.window_base.get_lines().join("\n"));
        
        output
    }
}

impl HasBaseConfig for WindowWithTitle {
    fn base_config(&self) -> &BaseConfig {
        self.window_base.base_config()
    }
    
    fn base_config_mut(&mut self) -> &mut BaseConfig {
        self.window_base.base_config_mut()
    }
}

impl ThreadConfig for WindowWithTitle {
    fn lines_to_display(&self) -> usize {
        self.window_base.max_lines() + 1 // +1 for title
    }

    fn handle_message(&mut self, message: String) -> Vec<String> {
        // Add message to window base
        self.window_base.add_message(message);
        
        // Return the current lines
        self.get_lines()
    }

    fn get_lines(&self) -> Vec<String> {
        // Get lines from window base
        let mut lines = self.window_base.get_lines();
        
        // Insert title at the beginning
        lines.insert(0, self.render_title(80));
        
        lines
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
    fn set_title(&mut self, title: String) -> Result<(), ModeCreationError> {
        if !self.supports_title {
            return Err(ModeCreationError::TitleNotSupported {
                mode_name: "WindowWithTitle".to_string(),
                reason: Some("Title support is disabled for this mode".to_string()),
            });
        }
        
        self.title = if title.is_empty() {
            "Progress".to_string()
        } else {
            title
        };
        Ok(())
    }
    
    fn get_title(&self) -> &str {
        &self.title
    }
}

impl WithCustomSize for WindowWithTitle {
    fn set_max_lines(&mut self, max_lines: usize) -> Result<(), ModeCreationError> {
        if max_lines < 2 {
            return Err(ModeCreationError::InvalidWindowSize {
                size: max_lines,
                min_size: 2,
                mode_name: "WindowWithTitle".to_string(),
                reason: Some("WindowWithTitle requires at least 2 lines: 1 for title and 1 for content".to_string()),
            });
        }
        
        // Get the current lines
        let current_lines = self.window_base.get_lines();
        
        // Create a new window base with the updated size
        let mut new_base = WindowBase::new(self.base_config().get_total_jobs(), max_lines - 1)?;
        
        // Copy over the existing lines
        for line in current_lines {
            new_base.add_message(line);
        }
        
        // Update the window base
        self.window_base = new_base;
        Ok(())
    }
    
    fn get_max_lines(&self) -> usize {
        self.window_base.max_lines() + 1
    }
}

impl WithEmoji for WindowWithTitle {
    fn add_emoji(&mut self, emoji: &str) -> Result<(), ModeCreationError> {
        if !self.supports_emoji {
            return Err(ModeCreationError::EmojiNotSupported {
                mode_name: "WindowWithTitle".to_string(),
                reason: Some("Emoji support is disabled for this mode".to_string()),
            });
        }
        
        if emoji.trim().is_empty() {
            return Err(ModeCreationError::Implementation("Emoji cannot be empty".to_string()));
        }
        
        // Check if emoji is already present
        if !self.emojis.contains(&emoji.to_string()) {
            self.emojis.push(emoji.to_string());
        }
        Ok(())
    }
    
    fn get_emojis(&self) -> Vec<String> {
        if self.supports_emoji {
            self.emojis.clone()
        } else {
            Vec::new()
        }
    }
}

impl WithTitleAndEmoji for WindowWithTitle {
    fn reset_with_title(&mut self, title: String) -> Result<(), ModeCreationError> {
        if !self.supports_title {
            return Err(ModeCreationError::TitleNotSupported {
                mode_name: "WindowWithTitle".to_string(),
                reason: Some("Title support is disabled for this mode".to_string()),
            });
        }
        if !self.supports_emoji {
            return Err(ModeCreationError::EmojiNotSupported {
                mode_name: "WindowWithTitle".to_string(),
                reason: Some("Emoji support is disabled for this mode".to_string()),
            });
        }
        self.emojis.clear();
        self.set_title(title)
    }
    
    fn get_formatted_title(&self) -> String {
        let mut title = String::new();
        
        // Add emojis if supported and present
        if self.supports_emoji && !self.emojis.is_empty() {
            title.push_str(&self.emojis.join(" "));
            title.push(' ');
        }
        
        // Add title if supported
        if self.supports_title {
            title.push_str(&self.title);
        }
        
        title
    }
}

impl StandardWindow for WindowWithTitle {
    fn clear(&mut self) {
        self.window_base.clear();
    }
    
    fn get_content(&self) -> Vec<String> {
        self.window_base.get_lines()
    }
    
    fn add_line(&mut self, line: String) {
        self.window_base.add_message(line);
    }
    
    fn is_empty(&self) -> bool {
        self.window_base.is_empty()
    }
    
    fn line_count(&self) -> usize {
        self.window_base.line_count()
    }
}

impl WithWrappedText for WindowWithTitle {
    fn set_line_wrapping(&mut self, enabled: bool) {
        self.window_base.set_line_wrapping(enabled);
    }
    
    fn has_line_wrapping(&self) -> bool {
        self.window_base.has_line_wrapping()
    }
}

impl WithProgress for WindowWithTitle {
    fn get_completed_jobs(&self) -> usize {
        self.base_config().get_completed_jobs()
    }
    
    fn set_progress_format(&mut self, format: &str) {
        self.base_config_mut().set_progress_format(format);
    }
    
    fn get_progress_format(&self) -> &str {
        self.base_config().get_progress_format()
    }
    
    fn update_progress(&mut self) -> f64 {
        let completed = self.base_config().increment_completed_jobs();
        ((completed as f64) / (self.get_total_jobs() as f64) * 100.0).min(100.0)
    }
    
    fn set_progress(&mut self, completed: usize) -> f64 {
        self.base_config_mut().set_completed_jobs(completed);
        ((completed as f64) / (self.get_total_jobs() as f64) * 100.0).min(100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn test_window_with_title_mode_basic() -> Result<()> {
        let mut mode = WindowWithTitle::new(1, 4, "Test Title".to_string())?;
        
        // Test initial state
        assert_eq!(mode.get_lines(), vec!["Test Title"]);
        
        // Test adding messages
        mode.handle_message("Message 1".to_string());
        assert_eq!(mode.get_lines(), vec!["Test Title", "Message 1"]);
        
        mode.handle_message("Message 2".to_string());
        assert_eq!(mode.get_lines(), vec!["Test Title", "Message 1", "Message 2"]);
        
        // Add a third message to ensure older messages are kept
        mode.handle_message("Message 3".to_string());
        assert_eq!(mode.get_lines(), vec!["Test Title", "Message 1", "Message 2", "Message 3"]);
        
        // Test title update
        mode.set_title("New Title".to_string())?;
        assert_eq!(mode.get_lines(), vec!["New Title", "Message 1", "Message 2", "Message 3"]);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_window_with_title_mode_concurrent() -> Result<()> {
        let mut mode = WindowWithTitle::new(2, 4, "Concurrent Test".to_string())?;
        
        // Test thread messages
        mode.handle_message("Thread 1: Starting".to_string());
        mode.handle_message("Thread 2: Starting".to_string());
        
        let lines = mode.get_lines();
        assert_eq!(lines[0], "Concurrent Test");
        assert!(lines.contains(&"Thread 1: Starting".to_string()));
        assert!(lines.contains(&"Thread 2: Starting".to_string()));
        
        Ok(())
    }

    #[tokio::test]
    async fn test_window_with_title_mode_long_lines() -> Result<()> {
        let mut mode = WindowWithTitle::new(1, 3, "Long Lines".to_string())?;
        
        let long_line = "A".repeat(100);
        mode.handle_message(long_line.clone());
        
        let lines = mode.get_lines();
        assert_eq!(lines[0], "Long Lines");
        assert_eq!(lines[1], long_line);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_window_with_title_mode_special_characters() -> Result<()> {
        let mut mode = WindowWithTitle::new(1, 3, "Special Chars".to_string())?;
        
        let special_chars = "🌟✨🎉";
        mode.handle_message(special_chars.to_string());
        
        let lines = mode.get_lines();
        assert_eq!(lines[0], "Special Chars");
        assert_eq!(lines[1], special_chars);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_window_with_title_mode_emoji() -> Result<()> {
        let mut mode = WindowWithTitle::new(1, 3, "Emoji Test".to_string())?;
        
        mode.add_emoji("🚀")?;
        let lines = mode.get_lines();
        assert_eq!(lines[0], "🚀 Emoji Test");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_window_with_title_mode_emoji_errors() -> Result<()> {
        let mut mode = WindowWithTitle::new(1, 3, "Emoji Errors".to_string())?;
        mode.supports_emoji = false;
        
        assert!(mode.add_emoji("🚀").is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_window_with_title_mode_multiple_emojis() -> Result<()> {
        let mut mode = WindowWithTitle::new(1, 3, "Multiple Emojis".to_string())?;
        
        mode.add_emoji("🚀")?;
        mode.add_emoji("✨")?;
        
        let lines = mode.get_lines();
        assert_eq!(lines[0], "🚀 ✨ Multiple Emojis");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_window_with_title_mode_set_title() -> Result<()> {
        let mut mode = WindowWithTitle::new(1, 3, "Initial Title".to_string())?;
        
        mode.set_title("New Title".to_string())?;
        assert_eq!(mode.get_title(), "New Title");
        assert_eq!(mode.get_lines()[0], "New Title");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_window_with_title_mode_set_title_error() -> Result<()> {
        let mut mode = WindowWithTitle::new(1, 3, "Title Test".to_string())?;
        mode.supports_title = false;
        
        assert!(mode.set_title("New Title".to_string()).is_err());
        assert_eq!(mode.get_title(), "Title Test");
        
        Ok(())
    }
} 