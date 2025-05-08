use std::collections::{VecDeque, HashMap};
use std::fmt::Debug;

use crate::errors::ModeCreationError;
use crate::io::ProgressWriter;

use crate::core::base_config::BaseConfig;
use crate::core::job_traits::HasBaseConfig;

/// Base implementation for window-based display modes.
/// 
/// WindowBase provides a scrolling window of lines that can be displayed
/// in the terminal, supporting thread-based output buffering and line wrapping.
#[derive(Debug, Clone)]
pub struct WindowBase {
    base: BaseConfig,
    lines: VecDeque<String>,
    max_lines: usize,
    thread_buffers: HashMap<String, VecDeque<String>>,
    is_threaded_mode: bool,
    line_wrapping: bool,
}

impl WindowBase {
    /// Creates a new WindowBase with the specified total jobs and maximum lines.
    ///
    /// # Parameters
    /// * `total_jobs` - The total number of jobs to track
    /// * `max_lines` - The maximum number of lines to display
    ///
    /// # Returns
    /// A Result containing either the new WindowBase or a ModeCreationError
    ///
    /// # Errors
    /// Returns an InvalidWindowSize error if max_lines is less than 1
    pub fn new(total_jobs: usize, max_lines: usize) -> Result<Self, ModeCreationError> {
        if max_lines < 1 {
            return Err(ModeCreationError::InvalidWindowSize {
                size: max_lines,
                min_size: 1,
                mode_name: "WindowBase".to_string(),
                reason: Some("Window size must be at least 1 line".to_string()),
            });
        }
        
        Ok(Self {
            base: BaseConfig::new(total_jobs),
            lines: VecDeque::with_capacity(max_lines),
            max_lines,
            thread_buffers: HashMap::new(),
            is_threaded_mode: false,
            line_wrapping: false,
        })
    }
    
    /// Add a message to the window.
    ///
    /// This method adds a new message to the window, potentially splitting it
    /// into multiple lines if it contains line breaks or if line wrapping is enabled.
    ///
    /// # Parameters
    /// * `message` - The message to add
    pub fn add_message(&mut self, message: String) {
        // Check for thread identification pattern
        if message.starts_with('[') && message.contains(']') {
            let end_idx = message.find(']').unwrap();
            let thread_id = message[1..end_idx].to_string();
            let content = message[end_idx + 1..].trim().to_string();
            
            let buffer = self.thread_buffers.entry(thread_id).or_default();
            buffer.push_back(content);
            
            self.is_threaded_mode = true;
        } else if self.is_threaded_mode {
            // If we're in threaded mode but get a message without a thread ID,
            // add it to all thread buffers
            for buffer in self.thread_buffers.values_mut() {
                buffer.push_back(message.clone());
            }
        } else {
            // Regular message processing
            if message.contains('\n') {
                // Split by newlines and add each line
                for line in message.split('\n') {
                    if !line.is_empty() {
                        self.add_single_line(line.to_string());
                    }
                }
            } else {
                self.add_single_line(message);
            }
        }
    }
    
    /// Add a single line to the window.
    ///
    /// This is a helper method that adds a single line to the window,
    /// potentially wrapping it if line wrapping is enabled.
    ///
    /// # Parameters
    /// * `line` - The line to add
    fn add_single_line(&mut self, line: String) {
        if self.line_wrapping {
            // Implement basic line wrapping for testing purposes
            // In a real implementation, we'd use the actual terminal width
            const WRAP_WIDTH: usize = 40;
            let mut remaining = line;
            
            while !remaining.is_empty() {
                if remaining.len() <= WRAP_WIDTH {
                    self.lines.push_back(remaining);
                    break;
                }
                
                // Try to find a space to break at
                let mut break_pos = WRAP_WIDTH;
                while break_pos > 0 && !remaining.is_char_boundary(break_pos) {
                    break_pos -= 1;
                }
                
                // Find a good breaking position (space)
                let optimal_pos = remaining[..break_pos].rfind(' ').unwrap_or(break_pos);
                let actual_pos = if optimal_pos == 0 { break_pos } else { optimal_pos };
                
                // Split the line and add the first part
                let (first, rest) = remaining.split_at(actual_pos);
                self.lines.push_back(first.to_string());
                
                // Continue with the rest of the line
                remaining = rest.trim_start().to_string();
            }
        } else {
            self.lines.push_back(line);
        }
        
        // Ensure we don't exceed max_lines
        while self.lines.len() > self.max_lines {
            self.lines.pop_front();
        }
    }
    
    /// Get the current lines to display.
    ///
    /// This method returns a vector of strings representing the current
    /// lines to display in the window.
    ///
    /// # Returns
    /// A vector of strings representing the lines to display
    pub fn get_lines(&self) -> Vec<String> {
        if self.is_threaded_mode {
            // In threaded mode, return the most recent line from each thread
            let mut result = Vec::with_capacity(self.thread_buffers.len());
            
            for (thread_id, buffer) in &self.thread_buffers {
                if let Some(line) = buffer.back() {
                    result.push(format!("[{}] {}", thread_id, line));
                }
            }
            
            result
        } else {
            // In regular mode, return all lines
            self.lines.iter().cloned().collect()
        }
    }
    
    /// Get the maximum number of lines that can be displayed.
    ///
    /// # Returns
    /// The maximum number of lines
    pub fn max_lines(&self) -> usize {
        self.max_lines
    }
    
    /// Clear all content from the window.
    ///
    /// This method clears all lines and thread buffers from the window.
    pub fn clear(&mut self) {
        self.lines.clear();
        self.thread_buffers.clear();
        self.is_threaded_mode = false;
    }
    
    /// Check if the window is empty.
    ///
    /// # Returns
    /// `true` if the window has no content, `false` otherwise
    pub fn is_empty(&self) -> bool {
        if self.is_threaded_mode {
            self.thread_buffers.is_empty() || self.thread_buffers.values().all(|buffer| buffer.is_empty())
        } else {
            self.lines.is_empty()
        }
    }
    
    /// Get the number of lines currently displayed.
    ///
    /// # Returns
    /// The number of lines
    pub fn line_count(&self) -> usize {
        if self.is_threaded_mode {
            self.thread_buffers.len()
        } else {
            self.lines.len()
        }
    }
    
    /// Get a reference to the BaseConfig.
    ///
    /// # Returns
    /// A reference to the BaseConfig
    pub fn base_config(&self) -> &BaseConfig {
        &self.base
    }
    
    /// Get a mutable reference to the BaseConfig.
    ///
    /// # Returns
    /// A mutable reference to the BaseConfig
    pub fn base_config_mut(&mut self) -> &mut BaseConfig {
        &mut self.base
    }
    
    /// Set whether line wrapping is enabled.
    ///
    /// # Parameters
    /// * `enabled` - Whether to enable line wrapping
    pub fn set_line_wrapping(&mut self, enabled: bool) {
        self.line_wrapping = enabled;
    }
    
    /// Check if line wrapping is enabled.
    ///
    /// # Returns
    /// `true` if line wrapping is enabled, `false` otherwise
    pub fn has_line_wrapping(&self) -> bool {
        self.line_wrapping
    }
}

/// Base implementation for single-line display modes.
/// 
/// SingleLineBase provides a single-line display that can be updated
/// with new messages, optionally passing through the raw output.
#[derive(Debug)]
pub struct SingleLineBase {
    base: BaseConfig,
    current_line: String,
    passthrough: bool,
    passthrough_writer: Option<Box<dyn ProgressWriter + Send + 'static>>,
}

impl Clone for SingleLineBase {
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
            current_line: self.current_line.clone(),
            passthrough: self.passthrough,
            passthrough_writer: None, // Can't clone the writer
        }
    }
}

impl SingleLineBase {
    /// Creates a new SingleLineBase with the specified total jobs.
    ///
    /// # Parameters
    /// * `total_jobs` - The total number of jobs to track
    /// * `passthrough` - Whether to enable passthrough mode
    ///
    /// # Returns
    /// A new SingleLineBase instance
    pub fn new(total_jobs: usize, passthrough: bool) -> Self {
        Self {
            base: BaseConfig::new(total_jobs),
            current_line: String::new(),
            passthrough,
            passthrough_writer: None,
        }
    }
    
    /// Update the current line with a new message.
    ///
    /// # Parameters
    /// * `message` - The new message
    pub fn update_line(&mut self, message: String) {
        self.current_line = message;
    }
    
    /// Get the current line.
    ///
    /// # Returns
    /// The current line
    pub fn get_line(&self) -> String {
        self.current_line.clone()
    }
    
    /// Check if passthrough is enabled.
    ///
    /// # Returns
    /// `true` if passthrough is enabled, `false` otherwise
    pub fn has_passthrough(&self) -> bool {
        self.passthrough
    }
}

/// Trait for modes that support output passthrough
pub trait WithPassthrough {
    /// Enable or disable passthrough mode
    fn set_passthrough(&mut self, enabled: bool);
    
    /// Check if passthrough is enabled
    fn has_passthrough(&self) -> bool;
    
    /// Get a mutable reference to the current passthrough writer
    fn get_passthrough_writer_mut(&mut self) -> Option<&mut dyn ProgressWriter>;
    
    /// Set a custom passthrough writer
    fn set_passthrough_writer(&mut self, writer: Box<dyn ProgressWriter + Send + 'static>) -> Result<(), ModeCreationError>;
}

impl WithPassthrough for SingleLineBase {
    fn set_passthrough(&mut self, enabled: bool) {
        self.passthrough = enabled;
    }
    
    fn has_passthrough(&self) -> bool {
        self.passthrough
    }
    
    fn get_passthrough_writer_mut(&mut self) -> Option<&mut dyn ProgressWriter> {
        self.passthrough_writer.as_deref_mut().map(|w| w as &mut dyn ProgressWriter)
    }
    
    fn set_passthrough_writer(&mut self, writer: Box<dyn ProgressWriter + Send + 'static>) -> Result<(), ModeCreationError> {
        self.passthrough_writer = Some(writer);
        Ok(())
    }
}

impl HasBaseConfig for WindowBase {
    fn base_config(&self) -> &BaseConfig {
        &self.base
    }
    
    fn base_config_mut(&mut self) -> &mut BaseConfig {
        &mut self.base
    }
}

impl HasBaseConfig for SingleLineBase {
    fn base_config(&self) -> &BaseConfig {
        &self.base
    }
    
    fn base_config_mut(&mut self) -> &mut BaseConfig {
        &mut self.base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_window_base_creation() {
        let window = WindowBase::new(10, 5).unwrap();
        assert_eq!(window.max_lines(), 5);
        assert_eq!(window.get_lines().len(), 0);
        assert!(window.is_empty());
    }
    
    #[test]
    fn test_window_base_invalid_size() {
        let result = WindowBase::new(10, 0);
        assert!(result.is_err());
        if let Err(ModeCreationError::InvalidWindowSize { size, min_size, mode_name, reason: _ }) = result {
            assert_eq!(size, 0);
            assert_eq!(min_size, 1);
            assert_eq!(mode_name, "WindowBase");
        } else {
            panic!("Expected InvalidWindowSize error");
        }
    }
    
    #[test]
    fn test_window_base_add_message() {
        let mut window = WindowBase::new(10, 3).unwrap();
        
        window.add_message("Line 1".to_string());
        window.add_message("Line 2".to_string());
        
        let lines = window.get_lines();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "Line 1");
        assert_eq!(lines[1], "Line 2");
        
        // Add more lines than max_lines
        window.add_message("Line 3".to_string());
        window.add_message("Line 4".to_string());
        
        let lines = window.get_lines();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Line 2");
        assert_eq!(lines[1], "Line 3");
        assert_eq!(lines[2], "Line 4");
    }
    
    #[test]
    fn test_window_base_multiline_message() {
        let mut window = WindowBase::new(10, 3).unwrap();
        
        window.add_message("Line 1\nLine 2\nLine 3".to_string());
        
        let lines = window.get_lines();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Line 1");
        assert_eq!(lines[1], "Line 2");
        assert_eq!(lines[2], "Line 3");
    }
    
    #[test]
    fn test_window_base_threaded_messages() {
        let mut window = WindowBase::new(10, 5).unwrap();
        
        window.add_message("[thread1] Message 1".to_string());
        window.add_message("[thread2] Message 2".to_string());
        window.add_message("[thread1] Message 3".to_string());
        
        let lines = window.get_lines();
        assert_eq!(lines.len(), 2);
        assert!(lines.contains(&"[thread1] Message 3".to_string()));
        assert!(lines.contains(&"[thread2] Message 2".to_string()));
    }
    
    #[test]
    fn test_window_base_clear() {
        let mut window = WindowBase::new(10, 3).unwrap();
        
        window.add_message("Line 1".to_string());
        window.add_message("Line 2".to_string());
        assert_eq!(window.line_count(), 2);
        
        window.clear();
        assert_eq!(window.line_count(), 0);
        assert!(window.is_empty());
    }
    
    #[test]
    fn test_single_line_base_creation() {
        let base = SingleLineBase::new(10, false);
        assert_eq!(base.get_line(), "");
        assert!(!base.has_passthrough());
    }
    
    #[test]
    fn test_single_line_base_update() {
        let mut base = SingleLineBase::new(10, false);
        
        base.update_line("Test line".to_string());
        assert_eq!(base.get_line(), "Test line");
        
        base.update_line("Updated line".to_string());
        assert_eq!(base.get_line(), "Updated line");
    }
    
    #[test]
    fn test_single_line_base_passthrough() {
        let mut base = SingleLineBase::new(10, true);
        assert!(base.has_passthrough());
        
        base.set_passthrough(false);
        assert!(!base.has_passthrough());
    }
} 