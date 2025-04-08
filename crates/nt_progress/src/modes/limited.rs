use std::io::{self, Write};
use super::{ThreadConfig, BaseConfig};

/// Configuration for Limited mode
/// 
/// In Limited mode, messages are passed through to stdout/stderr
/// and only the last message is kept for display
#[derive(Debug, Clone)]
pub struct Limited {
    pub base: BaseConfig,
    last_line: String,
}

impl Limited {
    pub fn new(total_jobs: usize) -> Self {
        Self {
            base: BaseConfig::new(total_jobs),
            last_line: String::new(),
        }
    }
    
    pub fn get_total_jobs(&self) -> usize {
        self.base.total_jobs
    }
    
    pub fn increment_completed_jobs(&self) -> usize {
        self.base.completed_jobs.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1
    }
}

impl ThreadConfig for Limited {
    fn lines_to_display(&self) -> usize {
        1
    }

    fn handle_message(&mut self, message: String) -> Vec<String> {
        // Pass message to stdout/stderr
        if let Err(e) = writeln!(io::stdout(), "{}", message) {
            eprintln!("Error writing to stdout: {}", e);
        }

        // Update last line
        self.last_line = message;
        self.get_lines()
    }

    fn get_lines(&self) -> Vec<String> {
        vec![self.last_line.clone()]
    }

    fn clone_box(&self) -> Box<dyn ThreadConfig> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limited_mode() {
        let mut limited = Limited::new(1);
        
        // Test initial state
        assert_eq!(limited.lines_to_display(), 1);
        assert_eq!(limited.get_lines(), vec![""]);
        assert_eq!(limited.get_total_jobs(), 1);
        
        // Test message handling
        let lines = limited.handle_message("test message".to_string());
        assert_eq!(lines, vec!["test message"]);
        
        // Test multiple messages
        limited.handle_message("new message".to_string());
        assert_eq!(limited.get_lines(), vec!["new message"]);
        
        // Test completed jobs
        assert_eq!(limited.increment_completed_jobs(), 1);
    }
} 