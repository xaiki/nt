use std::collections::VecDeque;
use super::{ThreadConfig, BaseConfig};

/// Configuration for Window mode
/// 
/// In Window mode, the last N lines are displayed,
/// where N is specified by the user and will be adjusted
/// if it doesn't fit the terminal.
#[derive(Debug, Clone)]
pub struct Window {
    base: BaseConfig,
    lines: VecDeque<String>,
    max_lines: usize,
}

impl Window {
    pub fn new(total_jobs: usize, max_lines: usize) -> Result<Self, String> {
        if max_lines == 0 {
            return Err("Window size must be at least 1".to_string());
        }
        Ok(Self {
            base: BaseConfig::new(total_jobs),
            lines: VecDeque::with_capacity(max_lines),
            max_lines,
        })
    }
    
    pub fn get_total_jobs(&self) -> usize {
        self.base.get_total_jobs()
    }
    
    pub fn increment_completed_jobs(&self) -> usize {
        self.base.increment_completed_jobs()
    }
}

impl ThreadConfig for Window {
    fn lines_to_display(&self) -> usize {
        self.max_lines
    }

    fn handle_message(&mut self, message: String) -> Vec<String> {
        // Add new line to the end
        self.lines.push_back(message);
        
        // Remove lines from the front if we exceed max_lines
        while self.lines.len() > self.max_lines {
            self.lines.pop_front();
        }
        
        self.get_lines()
    }

    fn get_lines(&self) -> Vec<String> {
        self.lines.iter().cloned().collect()
    }

    fn clone_box(&self) -> Box<dyn ThreadConfig> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_mode() {
        let mut window = Window::new(1, 3).unwrap();
        
        // Test initial state
        assert_eq!(window.lines_to_display(), 3);
        assert_eq!(window.get_lines(), Vec::<String>::new());
        assert_eq!(window.get_total_jobs(), 1);
        
        // Test adding lines up to max_lines
        window.handle_message("line 1".to_string());
        assert_eq!(window.get_lines(), vec!["line 1"]);
        
        window.handle_message("line 2".to_string());
        assert_eq!(window.get_lines(), vec!["line 1", "line 2"]);
        
        window.handle_message("line 3".to_string());
        assert_eq!(window.get_lines(), vec!["line 1", "line 2", "line 3"]);
        
        // Test exceeding max_lines
        window.handle_message("line 4".to_string());
        assert_eq!(window.get_lines(), vec!["line 2", "line 3", "line 4"]);
        
        // Test completed jobs
        assert_eq!(window.increment_completed_jobs(), 1);
    }

    #[test]
    fn test_window_mode_invalid_size() {
        assert!(Window::new(1, 0).is_err());
    }

    #[test]
    fn test_window_mode_output_format() {
        let mut window = Window::new(5, 3).unwrap();
        
        // Test progress bar format
        window.handle_message("Progress: 2/5".to_string());
        assert_eq!(window.get_lines(), vec!["Progress: 2/5"]);
        
        // Test multiple lines with different formats
        window.handle_message("Downloading: 50%".to_string());
        window.handle_message("Processing: 75%".to_string());
        assert_eq!(window.get_lines(), vec![
            "Progress: 2/5",
            "Downloading: 50%",
            "Processing: 75%"
        ]);
        
        // Test line truncation
        window.handle_message("New line".to_string());
        assert_eq!(window.get_lines(), vec![
            "Downloading: 50%",
            "Processing: 75%",
            "New line"
        ]);
        
        // Test completed jobs counter
        assert_eq!(window.increment_completed_jobs(), 1);
        assert_eq!(window.increment_completed_jobs(), 2);
    }

    #[test]
    fn test_window_mode_edge_cases() {
        // Test with minimum valid size
        let mut window = Window::new(1, 1).unwrap();
        window.handle_message("single line".to_string());
        assert_eq!(window.get_lines(), vec!["single line"]);
        
        // Test with large number of lines
        let mut window = Window::new(1, 100).unwrap();
        for i in 0..150 {
            window.handle_message(format!("line {}", i));
        }
        assert_eq!(window.get_lines().len(), 100);
        assert_eq!(window.get_lines()[0], "line 50"); // First line should be line 50
        assert_eq!(window.get_lines()[99], "line 149"); // Last line should be line 149
    }
} 