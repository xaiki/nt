use super::{ThreadConfig, SingleLineBase, JobTracker};

/// Configuration for Capturing mode
/// 
/// In Capturing mode, only one line is displayed at a time,
/// with each new message replacing the previous one.
/// Unlike Limited mode, it does not send output to stdout/stderr.
#[derive(Debug, Clone)]
pub struct Capturing {
    single_line_base: SingleLineBase,
}

impl Capturing {
    pub fn new(total_jobs: usize) -> Self {
        Self {
            single_line_base: SingleLineBase::new(total_jobs, false), // false = no passthrough
        }
    }
}

impl JobTracker for Capturing {
    fn get_total_jobs(&self) -> usize {
        self.single_line_base.get_total_jobs()
    }
    
    fn increment_completed_jobs(&self) -> usize {
        self.single_line_base.increment_completed_jobs()
    }
}

impl ThreadConfig for Capturing {
    fn lines_to_display(&self) -> usize {
        1
    }

    fn handle_message(&mut self, message: String) -> Vec<String> {
        // In Capturing mode, we just replace the current line, no stdout
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