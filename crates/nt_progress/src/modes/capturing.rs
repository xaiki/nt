use crate::core::{ThreadConfig, HasBaseConfig, BaseConfig};
use super::window_base::SingleLineBase;
use std::any::Any;
use std::fmt::Debug;

/// Configuration for Capturing mode
///
/// Capturing mode captures output without displaying it in the terminal.
/// This is useful for saving output for later use or for automated testing.
#[derive(Debug, Clone)]
pub struct Capturing {
    base: SingleLineBase,
    capture: Vec<String>,
}

impl Capturing {
    /// Creates a new Capturing mode configuration
    /// 
    /// # Parameters
    /// * `total_jobs` - The total number of jobs to track
    /// 
    /// # Returns
    /// A new Capturing configuration
    pub fn new(total_jobs: usize) -> Self {
        Self {
            base: SingleLineBase::new(total_jobs, false),
            capture: Vec::new(),
        }
    }
    
    /// Get captured output
    pub fn captured(&self) -> &[String] {
        &self.capture
    }
}

impl HasBaseConfig for Capturing {
    fn base_config(&self) -> &BaseConfig {
        self.base.base_config()
    }
    
    fn base_config_mut(&mut self) -> &mut BaseConfig {
        self.base.base_config_mut()
    }
}

// Now JobTracker is automatically implemented via the blanket implementation

impl ThreadConfig for Capturing {
    fn lines_to_display(&self) -> usize {
        1
    }

    fn handle_message(&mut self, message: String) -> Vec<String> {
        // In Capturing mode, we just replace the current line, no stdout
        self.base.update_line(message);
        self.get_lines()
    }

    fn get_lines(&self) -> Vec<String> {
        vec![self.base.get_line()]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::job_traits::JobTracker;

    #[test]
    fn test_capturing_mode() {
        let mut capturing = Capturing::new(1);
        
        // Test initial state
        assert_eq!(capturing.lines_to_display(), 1);
        assert_eq!(capturing.get_lines(), vec![""]);
        assert_eq!(capturing.get_total_jobs(), 1);
        
        // Test message handling - should replace content
        let lines = capturing.handle_message("test message".to_string());
        assert_eq!(lines, vec!["test message"]);
        
        // Test multiple messages - each replaces the previous
        capturing.handle_message("new message".to_string());
        assert_eq!(capturing.get_lines(), vec!["new message"]);
        
        // Old message is completely gone
        capturing.handle_message("final message".to_string());
        assert_eq!(capturing.get_lines(), vec!["final message"]);
        assert_eq!(capturing.get_lines().len(), 1);
        
        // Test completed jobs
        assert_eq!(capturing.increment_completed_jobs(), 1);
    }
} 