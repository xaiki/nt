use std::collections::VecDeque;
use super::{ThreadConfig, BaseConfig};

/// Configuration for Window with Title mode
/// 
/// In Window with Title mode, a title is displayed followed by the last N lines,
/// where N is specified by the user and will be adjusted if it doesn't fit the terminal.
/// This mode also supports emoji stacking.
#[derive(Debug, Clone)]
pub struct WindowWithTitle {
    base: BaseConfig,
    lines: VecDeque<String>,
    max_lines: usize,
    title: String,
    emoji_stack: Vec<String>,
}

impl WindowWithTitle {
    pub fn new(total_jobs: usize, max_lines: usize, title: String) -> Self {
        Self {
            base: BaseConfig::new(total_jobs),
            lines: VecDeque::with_capacity(max_lines),
            max_lines,
            title: if title.is_empty() { "Progress".to_string() } else { title },
            emoji_stack: Vec::new(),
        }
    }
    
    pub fn get_total_jobs(&self) -> usize {
        self.base.get_total_jobs()
    }
    
    pub fn increment_completed_jobs(&self) -> usize {
        self.base.increment_completed_jobs()
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub fn push_emoji(&mut self, emoji: String) {
        self.emoji_stack.push(emoji);
    }

    pub fn pop_emoji(&mut self) -> Option<String> {
        self.emoji_stack.pop()
    }
}

impl ThreadConfig for WindowWithTitle {
    fn lines_to_display(&self) -> usize {
        // Add 1 for the title line
        self.max_lines + 1
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
        let mut result = Vec::with_capacity(self.lines.len() + 1);
        
        // Add title with emoji stack
        let emoji_str = self.emoji_stack.join(" ");
        let title_line = if emoji_str.is_empty() {
            self.title.clone()
        } else {
            format!("{} {}", emoji_str, self.title)
        };
        result.push(title_line);
        
        // Add content lines
        result.extend(self.lines.iter().cloned());
        result
    }

    fn clone_box(&self) -> Box<dyn ThreadConfig> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_with_title_mode() {
        let mut window = WindowWithTitle::new(1, 3, "Test Title".to_string());
        
        // Test initial state
        assert_eq!(window.lines_to_display(), 4); // 3 lines + title
        assert_eq!(window.get_lines(), vec!["Test Title"]);
        assert_eq!(window.get_total_jobs(), 1);
        
        // Test adding lines up to max_lines
        window.handle_message("line 1".to_string());
        assert_eq!(window.get_lines(), vec!["Test Title", "line 1"]);
        
        window.handle_message("line 2".to_string());
        assert_eq!(window.get_lines(), vec!["Test Title", "line 1", "line 2"]);
        
        window.handle_message("line 3".to_string());
        assert_eq!(window.get_lines(), vec!["Test Title", "line 1", "line 2", "line 3"]);
        
        // Test exceeding max_lines
        window.handle_message("line 4".to_string());
        assert_eq!(window.get_lines(), vec!["Test Title", "line 2", "line 3", "line 4"]);
        
        // Test title change
        window.set_title("New Title".to_string());
        assert_eq!(window.get_lines(), vec!["New Title", "line 2", "line 3", "line 4"]);
        
        // Test emoji stack
        window.push_emoji("🚀".to_string());
        assert_eq!(window.get_lines(), vec!["🚀 New Title", "line 2", "line 3", "line 4"]);
        
        window.push_emoji("✨".to_string());
        assert_eq!(window.get_lines(), vec!["🚀 ✨ New Title", "line 2", "line 3", "line 4"]);
        
        window.pop_emoji();
        assert_eq!(window.get_lines(), vec!["🚀 New Title", "line 2", "line 3", "line 4"]);
        
        // Test completed jobs
        assert_eq!(window.increment_completed_jobs(), 1);
    }

    #[test]
    fn test_window_with_title_output_format() {
        let mut window = WindowWithTitle::new(5, 3, "Progress".to_string());
        
        // Test title with emojis
        window.push_emoji("🚀".to_string());
        window.push_emoji("✨".to_string());
        assert_eq!(window.get_lines()[0], "🚀 ✨ Progress");
        
        // Test progress messages
        window.handle_message("Downloading: 50%".to_string());
        window.handle_message("Processing: 75%".to_string());
        assert_eq!(window.get_lines(), vec![
            "🚀 ✨ Progress",
            "Downloading: 50%",
            "Processing: 75%"
        ]);
        
        // Test line truncation
        window.handle_message("New line".to_string());
        assert_eq!(window.get_lines(), vec![
            "🚀 ✨ Progress",
            "Processing: 75%",
            "New line"
        ]);
        
        // Test emoji stack changes
        window.pop_emoji();
        assert_eq!(window.get_lines()[0], "🚀 Progress");
        
        window.push_emoji("🎉".to_string());
        assert_eq!(window.get_lines()[0], "🚀 🎉 Progress");
    }

    #[test]
    fn test_window_with_title_edge_cases() {
        // Test empty title
        let mut window = WindowWithTitle::new(1, 3, String::new());
        assert_eq!(window.get_lines()[0], "Progress"); // Should use default title
        
        // Test with many emojis
        let mut window = WindowWithTitle::new(1, 3, "Test".to_string());
        for _ in 0..10 {
            window.push_emoji("🚀".to_string());
        }
        assert_eq!(window.get_lines()[0], "🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 🚀 Test");
        
        // Test with large number of lines
        let mut window = WindowWithTitle::new(1, 100, "Title".to_string());
        for i in 0..150 {
            window.handle_message(format!("line {}", i));
        }
        assert_eq!(window.get_lines().len(), 101); // 100 lines + title
        assert_eq!(window.get_lines()[0], "Title");
        assert_eq!(window.get_lines()[1], "line 50"); // First content line
        assert_eq!(window.get_lines()[100], "line 149"); // Last content line
    }
} 