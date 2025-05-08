use crate::core::job_traits::JobTracker;
use crate::errors::ModeCreationError;

/// Capability for modes that support setting and retrieving a title.
///
/// # Examples
/// ```rust
/// # use nt_progress::modes::{WindowWithTitle, capabilities::WithTitle};
/// # use nt_progress::errors::ModeCreationError;
/// # fn example() -> Result<(), ModeCreationError> {
/// let mut mode = WindowWithTitle::new(10, 5, "Initial Title".to_string())?;
/// mode.set_title("New Title".to_string())?;
/// assert_eq!(mode.get_title(), "New Title");
/// # Ok(())
/// # }
/// ```
pub trait WithTitle: Send + Sync {
    /// Set the title for this display mode.
    ///
    /// # Parameters
    /// * `title` - The new title string
    ///
    /// # Returns
    /// `Ok(())` if successful, or an error if the title could not be set
    ///
    /// # Errors
    /// Returns `ModeCreationError` if the title is invalid or cannot be set.
    fn set_title(&mut self, title: String) -> Result<(), ModeCreationError>;
    
    /// Get the current title.
    ///
    /// # Returns
    /// A reference to the current title string.
    fn get_title(&self) -> &str;
}

/// Capability for modes that can have a custom size.
///
/// This trait allows modes to customize the number of lines they display.
pub trait WithCustomSize: Send + Sync {
    /// Set the maximum number of lines to display.
    ///
    /// # Parameters
    /// * `max_lines` - The maximum number of lines to display
    ///
    /// # Returns
    /// `Ok(())` if successful, or an error if the size is invalid
    ///
    /// # Errors
    /// Returns `ModeCreationError` if the size is invalid.
    fn set_max_lines(&mut self, max_lines: usize) -> Result<(), ModeCreationError>;
    
    /// Get the maximum number of lines that can be displayed.
    ///
    /// # Returns
    /// The maximum number of lines
    fn get_max_lines(&self) -> usize;
}

/// Capability for modes that can display emoji characters.
///
/// This trait allows modes to add emoji characters to their display.
pub trait WithEmoji: Send + Sync {
    /// Add an emoji to the display.
    ///
    /// # Parameters
    /// * `emoji` - The emoji string to add
    ///
    /// # Returns
    /// `Ok(())` if successful, or an error if the emoji could not be added
    ///
    /// # Errors
    /// Returns `ModeCreationError` if the emoji is invalid or cannot be added.
    fn add_emoji(&mut self, emoji: &str) -> Result<(), ModeCreationError>;
    
    /// Get the current emojis.
    ///
    /// # Returns
    /// A vector of emoji strings
    fn get_emojis(&self) -> Vec<String>;
}

/// Capability for modes that support both titles and emojis.
///
/// This trait combines the WithTitle and WithEmoji traits to provide
/// a unified interface for modes that support both features.
pub trait WithTitleAndEmoji: WithTitle + WithEmoji {
    /// Set the title and add an emoji in a single operation.
    ///
    /// # Parameters
    /// * `title` - The new title string
    /// * `emoji` - The emoji string to add
    ///
    /// # Returns
    /// `Ok(())` if successful, or an error if either operation fails
    ///
    /// # Errors
    /// Returns `ModeCreationError` if either operation fails.
    fn set_title_with_emoji(&mut self, title: String, emoji: &str) -> Result<(), ModeCreationError> {
        self.set_title(title)?;
        self.add_emoji(emoji)
    }
    
    /// Clear all emojis and set a new title.
    ///
    /// # Parameters
    /// * `title` - The new title string
    ///
    /// # Returns
    /// `Ok(())` if successful, or an error if the title cannot be set
    ///
    /// # Errors
    /// Returns `ModeCreationError` if the title cannot be set.
    fn reset_with_title(&mut self, title: String) -> Result<(), ModeCreationError>;
    
    /// Get the fully formatted title with emojis.
    ///
    /// # Returns
    /// A string containing the title with emojis
    fn get_formatted_title(&self) -> String;
}

/// Capability for modes that implement standard window functionality.
///
/// This trait defines standard operations for window-based display modes.
pub trait StandardWindow: WithCustomSize {
    /// Clear all content from the window.
    fn clear(&mut self);
    
    /// Get the current content as a vector of strings.
    ///
    /// # Returns
    /// A vector of strings representing the current content
    fn get_content(&self) -> Vec<String>;
    
    /// Add a single line to the window.
    ///
    /// # Parameters
    /// * `line` - The line to add
    fn add_line(&mut self, line: String);
    
    /// Check if the window is empty.
    ///
    /// # Returns
    /// `true` if the window is empty, `false` otherwise
    fn is_empty(&self) -> bool;
    
    /// Get the number of lines currently displayed.
    ///
    /// # Returns
    /// The number of lines
    fn line_count(&self) -> usize;
}

/// Capability for modes that support line wrapping for long text.
///
/// This trait allows modes to enable or disable line wrapping.
pub trait WithWrappedText: Send + Sync {
    /// Enable or disable line wrapping.
    ///
    /// # Parameters
    /// * `enabled` - Whether to enable or disable line wrapping
    fn set_line_wrapping(&mut self, enabled: bool);
    
    /// Check if line wrapping is enabled.
    ///
    /// # Returns
    /// true if line wrapping is enabled, false otherwise
    fn has_line_wrapping(&self) -> bool;
}

/// Capability for modes that support progress tracking and display.
///
/// This trait extends the JobTracker trait to add progress display functionality.
pub trait WithProgress: JobTracker + Send + Sync {
    /// Calculate the current progress as a percentage.
    ///
    /// # Returns
    /// A float between 0.0 and 100.0 representing the progress percentage
    fn get_progress_percentage(&self) -> f64 {
        let total = self.get_total_jobs();
        if total == 0 {
            return 0.0;
        }
        
        let completed = self.get_completed_jobs();
        ((completed as f64) / (total as f64) * 100.0).min(100.0)
    }
    
    /// Get the number of completed jobs.
    ///
    /// # Returns
    /// The number of completed jobs
    fn get_completed_jobs(&self) -> usize;
    
    /// Set the progress display format.
    ///
    /// # Parameters
    /// * `format` - The format string for progress display
    fn set_progress_format(&mut self, format: &str);
    
    /// Get the current progress display format.
    ///
    /// # Returns
    /// The current progress format string
    fn get_progress_format(&self) -> &str;
    
    /// Update the progress by incrementing the completed jobs counter.
    ///
    /// This also updates time estimates.
    ///
    /// # Returns
    /// The updated progress percentage
    fn update_progress(&mut self) -> f64 {
        let _ = self.increment_completed_jobs();
        self.update_time_estimates()
    }
    
    /// Update the progress to a specific number of completed jobs.
    ///
    /// This also updates time estimates.
    ///
    /// # Parameters
    /// * `completed` - The number of completed jobs
    ///
    /// # Returns
    /// The updated progress percentage
    fn set_progress(&mut self, completed: usize) -> f64 {
        // The default implementation can be overridden by specific modes
        let previous = self.get_completed_jobs();
        
        if completed > previous {
            // We need to update our completion count
            // For each step, call update_progress which will
            // update our time estimates
            for _ in 0..(completed - previous) {
                self.increment_completed_jobs();
            }
        }
        
        self.update_time_estimates()
    }
    
    /// Get the estimated time remaining until completion.
    ///
    /// This method calculates the estimated time remaining based on the current progress
    /// and the rate of progress. If the progress data is insufficient to make a reliable
    /// estimate, this method returns None.
    ///
    /// # Returns
    /// Some(Duration) with the estimated time remaining, or None if an estimate cannot be made.
    fn get_estimated_time_remaining(&self) -> Option<std::time::Duration> {
        None
    }
    
    /// Get the current progress speed in units per second.
    ///
    /// This method calculates the speed of progress based on recent updates.
    /// If there is insufficient data to calculate a speed, this method returns None.
    ///
    /// # Returns
    /// Some(f64) with the speed in units per second, or None if the speed cannot be calculated.
    fn get_progress_speed(&self) -> Option<f64> {
        None
    }
    
    /// Get the elapsed time since the progress tracking began.
    ///
    /// # Returns
    /// The duration since progress tracking began.
    fn get_elapsed_time(&self) -> std::time::Duration {
        std::time::Duration::from_secs(0)
    }
    
    /// Update time estimates based on current progress.
    ///
    /// # Returns
    /// The current progress percentage
    fn update_time_estimates(&mut self) -> f64 {
        self.get_progress_percentage()
    }
}

/// Enum of available capabilities that display modes can support.
///
/// This enum is used to check and represent the capabilities
/// that different display modes support.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Capability {
    /// The mode supports setting and getting a title.
    Title,
    
    /// The mode supports customizing the display size.
    CustomSize,
    
    /// The mode supports adding emoji characters.
    Emoji,
    
    /// The mode supports both title and emoji capabilities.
    TitleAndEmoji,
    
    /// The mode supports standard window operations.
    StandardWindow,

    /// The mode supports line wrapping for long text.
    WrappedText,
    
    /// The mode supports progress tracking and percentage display.
    Progress,
    
    /// The mode supports job prioritization.
    PrioritizedJob,
    
    /// The mode supports pausing and resuming jobs.
    PausableJob,
    
    /// The mode supports job dependencies.
    DependentJob,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ModeCreationError;
    
    // Simple mock implementor for WithTitle
    struct MockTitled {
        title: String,
    }
    
    impl MockTitled {
        fn new(title: &str) -> Self {
            Self {
                title: title.to_string(),
            }
        }
    }
    
    impl WithTitle for MockTitled {
        fn set_title(&mut self, title: String) -> Result<(), ModeCreationError> {
            self.title = title;
            Ok(())
        }
        
        fn get_title(&self) -> &str {
            &self.title
        }
    }
    
    impl WithEmoji for MockTitled {
        fn add_emoji(&mut self, _emoji: &str) -> Result<(), ModeCreationError> {
            // Simple mock implementation
            Ok(())
        }
        
        fn get_emojis(&self) -> Vec<String> {
            vec![]
        }
    }
    
    impl WithTitleAndEmoji for MockTitled {
        fn reset_with_title(&mut self, title: String) -> Result<(), ModeCreationError> {
            self.title = title;
            Ok(())
        }
        
        fn get_formatted_title(&self) -> String {
            self.title.clone()
        }
    }
    
    #[test]
    fn test_with_title_and_emoji() {
        let mut titled = MockTitled::new("Initial");
        
        // Test individual trait methods
        assert_eq!(titled.get_title(), "Initial");
        titled.set_title("Updated".to_string()).unwrap();
        assert_eq!(titled.get_title(), "Updated");
        
        // Test combined trait methods
        titled.set_title_with_emoji("Combined".to_string(), "🔥").unwrap();
        assert_eq!(titled.get_title(), "Combined");
        
        titled.reset_with_title("Reset".to_string()).unwrap();
        assert_eq!(titled.get_title(), "Reset");
        assert_eq!(titled.get_formatted_title(), "Reset");
    }
    
    #[test]
    fn test_with_progress() {
        struct MockProgress {
            total: usize,
            completed: usize,
            format: String,
        }
        
        impl MockProgress {
            fn new(total: usize) -> Self {
                Self {
                    total,
                    completed: 0,
                    format: "{completed}/{total}".to_string(),
                }
            }
        }
        
        impl std::fmt::Debug for MockProgress {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "MockProgress({}/{})", self.completed, self.total)
            }
        }
        
        impl JobTracker for MockProgress {
            fn get_total_jobs(&self) -> usize {
                self.total
            }
            
            fn increment_completed_jobs(&self) -> usize {
                // This is a mock, so we don't actually increment
                self.completed + 1
            }
            
            fn set_total_jobs(&mut self, total: usize) {
                self.total = total;
            }
        }
        
        impl WithProgress for MockProgress {
            fn get_completed_jobs(&self) -> usize {
                self.completed
            }
            
            fn set_progress_format(&mut self, format: &str) {
                self.format = format.to_string();
            }
            
            fn get_progress_format(&self) -> &str {
                &self.format
            }
            
            fn update_progress(&mut self) -> f64 {
                self.completed += 1;
                self.get_progress_percentage()
            }
            
            fn set_progress(&mut self, completed: usize) -> f64 {
                self.completed = completed;
                self.get_progress_percentage()
            }
        }
        
        let mut progress = MockProgress::new(10);
        
        assert_eq!(progress.get_total_jobs(), 10);
        assert_eq!(progress.get_completed_jobs(), 0);
        assert_eq!(progress.get_progress_percentage(), 0.0);
        
        progress.update_progress();
        assert_eq!(progress.get_completed_jobs(), 1);
        assert_eq!(progress.get_progress_percentage(), 10.0);
        
        progress.set_progress(5);
        assert_eq!(progress.get_completed_jobs(), 5);
        assert_eq!(progress.get_progress_percentage(), 50.0);
        
        progress.set_progress_format("{percent}%");
        assert_eq!(progress.get_progress_format(), "{percent}%");
    }
} 