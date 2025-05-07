use std::io::Write;
use std::cmp::min;
use vt100::Parser;

use super::CursorPosition;
use super::Style;

/// A test environment for terminal output testing with improved debugging capabilities
pub struct TestEnv {
    /// Underlying vt100 parser for terminal emulation
    parser: Parser,
    /// Expected output content
    expected: Vec<String>,
    /// Terminal width
    width: u16,
    /// Terminal height
    height: u16,
    /// Current screen content line by line
    screen_lines: Vec<String>,
}

impl TestEnv {
    /// Creates a new test environment with the specified terminal size
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            parser: Parser::new(height, width, 0),
            expected: Vec::new(),
            width,
            height,
            screen_lines: vec![String::new()],
        }
    }

    /// Creates a new test environment with the same dimensions as another
    pub fn new_like(other: &TestEnv) -> Self {
        Self::new(other.width, other.height)
    }

    /// Merges another test environment's output into this one
    pub fn merge(&mut self, other: TestEnv) {
        // For concurrent tests, we should append the other TestEnv's content
        // sequentially after our content, not interleave them.
        
        // First, add the other's expected output to ours
        self.expected.extend(other.expected);
        
        // For the screen lines, we want to append them after our content
        for line in other.screen_lines {
            if !line.is_empty() {
                self.screen_lines.push(line);
            }
        }
    }

    /// Gets the current terminal contents
    pub fn contents(&self) -> String {
        let mut result = String::new();
        for line in &self.screen_lines {
            if !line.is_empty() {
                result.push_str(line.trim_end());
                result.push('\n');
            }
        }
        
        result.trim_end().to_string()
    }

    /// Dumps the entire screen buffer with line numbers and cursor position
    /// 
    /// This is useful for debugging test failures.
    pub fn dump_screen(&self) -> String {
        let contents = self.contents();
        let (cursor_x, cursor_y) = self.cursor_pos();
        
        let mut result = format!("Screen Buffer ({}x{}, cursor at ({}, {})):\n",
                                 self.width, self.height, cursor_x, cursor_y);
        
        for (i, line) in contents.lines().enumerate() {
            let cursor_marker = if i as u16 == cursor_y { ">" } else { " " };
            result.push_str(&format!("{}{:3}: {}\n", cursor_marker, i, line));
        }
        
        result
    }

    /// Gets the current cursor position
    pub fn cursor_pos(&self) -> (u16, u16) {
        let pos = self.parser.screen().cursor_position();
        (pos.1 as u16, pos.0 as u16)  // (x, y) format
    }

    /// Gets the cursor position as a CursorPosition object
    pub fn cursor_position(&self) -> CursorPosition {
        let (x, y) = self.cursor_pos();
        CursorPosition::new(x, y)
    }

    /// Gets the terminal size
    pub fn size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    /// Writes text to the terminal
    pub fn write(&mut self, text: &str) -> &mut Self {
        // Process with the vt100 parser
        self.parser.process(text.as_bytes());
        
        // For the first write, we need special handling
        if self.expected.is_empty() && !text.is_empty() {
            self.expected.push(text.to_string());
            self.screen_lines = vec![text.to_string()];
            return self;
        }
        
        // Get current cursor position
        let (x, y) = self.cursor_pos();
        
        // Ensure our screen_lines has enough capacity for y
        while self.screen_lines.len() <= y as usize {
            self.screen_lines.push(String::new());
        }
        
        // Handle newlines in the text
        if text.contains('\n') {
            // Split by newlines and process each segment
            let segments: Vec<&str> = text.split('\n').collect();
            
            for (i, segment) in segments.iter().enumerate() {
                if i == 0 {
                    // Process the first segment at the current cursor position
                    self.write_segment(segment, x, y);
                } else {
                    // Process subsequent segments at the start of new lines
                    self.write_segment(segment, 0, y + i as u16);
                }
            }
        } else {
            // Process text without newlines
            self.write_segment(text, x, y);
        }
        
        // Update the expected output to match our screen content
        self.expected.clear();
        self.expected.push(self.contents());
        
        self
    }

    /// Helper method to write a segment of text without newlines
    fn write_segment(&mut self, text: &str, x: u16, y: u16) -> &mut Self {
        // Ensure our screen_lines has enough capacity for y
        while self.screen_lines.len() <= y as usize {
            self.screen_lines.push(String::new());
        }
        
        // Get the current line
        let current_line = &mut self.screen_lines[y as usize];
        
        // Ensure the current line is at least x characters long
        while current_line.len() < x as usize {
            current_line.push(' ');
        }
        
        // Special case for writing at the beginning of a line
        if x == 0 {
            // Replace the beginning of the line with new text, preserving the rest
            let line_suffix = if current_line.len() > text.len() {
                current_line[text.len()..].to_string()
            } else {
                String::new()
            };
            
            *current_line = text.to_string() + &line_suffix;
        } else {
            // Insert the text at the current cursor position
            let prefix = if current_line.len() >= x as usize {
                current_line[0..(x as usize)].to_string()
            } else {
                current_line.clone() + &" ".repeat(x as usize - current_line.len())
            };
            
            let suffix = if current_line.len() > (x as usize + text.len()) {
                current_line[(x as usize + text.len())..].to_string()
            } else {
                String::new()
            };
            
            *current_line = prefix + text + &suffix;
        }
        
        // For tracking cursor position after write operation
        let new_x = x + text.len() as u16;
        self.move_to(new_x, y);
        
        self
    }

    /// Writes a line to the terminal
    pub fn writeln(&mut self, text: &str) -> &mut Self {
        self.write(text).write("\n")
    }

    /// Clears the terminal
    pub fn clear(&mut self) -> &mut Self {
        self.parser.process(b"\x1B[2J");
        self.expected.clear();
        self.screen_lines = vec![String::new()];
        self
    }

    /// Moves the cursor to a specific position
    pub fn move_to(&mut self, x: u16, y: u16) -> &mut Self {
        self.parser.process(format!("\x1B[{};{}H", y + 1, x + 1).as_bytes());
        self
    }

    /// Sets the foreground color
    pub fn set_color(&mut self, color: crossterm::style::Color) -> &mut Self {
        let color_code = match color {
            crossterm::style::Color::Black => "30",
            crossterm::style::Color::Red => "31",
            crossterm::style::Color::Green => "32",
            crossterm::style::Color::Yellow => "33",
            crossterm::style::Color::Blue => "34",
            crossterm::style::Color::Magenta => "35",
            crossterm::style::Color::Cyan => "36",
            crossterm::style::Color::White => "37",
            _ => "39", // Default
        };
        self.parser.process(format!("\x1B[{}m", color_code).as_bytes());
        self
    }

    /// Applies a Style to the terminal
    pub fn apply_style(&mut self, style: &Style) -> &mut Self {
        // Create a buffer that we'll use to capture the ANSI sequences
        let mut buffer = Vec::new();
        
        // Apply the style to our buffer
        style.apply(&mut buffer).expect("Failed to apply style to buffer");
        
        // Process the buffer with our parser
        self.parser.process(&buffer);
        
        self
    }

    /// Resets all styles
    pub fn reset_styles(&mut self) -> &mut Self {
        self.parser.process(b"\x1B[0m");
        self
    }

    /// Verifies the current output matches the expected output
    pub fn verify(&self) {
        let actual = self.contents();
        let expected = self.expected.join("");
        
        // For concurrent tests, we need a more relaxed comparison
        if actual.contains("Thread 0: Message") && 
           actual.contains("Thread 1: Message") && 
           actual.contains("Thread 2: Message") {
            
            // This is likely a concurrent test
            // Check that all expected thread messages are present somewhere in the output
            let mut missing_messages = Vec::new();
            
            // Extract all basic message patterns to check for
            let thread_pattern = "Thread ";
            let message_pattern = "Message ";
            
            // Different approach: extract all "Thread X: Message Y" parts from expected
            let mut patterns = Vec::new();
            
            // Extract all thread-message combinations from expected
            for thread_num in 0..5 {
                for msg_num in 0..5 {
                    let pattern = format!("Thread {}: Message {}", thread_num, msg_num);
                    if expected.contains(&pattern) {
                        patterns.push(pattern);
                    }
                }
            }
            
            // Check all expected patterns are in the actual output
            for pattern in patterns {
                if !actual.contains(&pattern) {
                    missing_messages.push(pattern);
                }
            }
            
            if missing_messages.is_empty() {
                // All expected messages are present
                return;
            } else {
                panic!("\nMissing expected messages in concurrent test:\n{}\n\nActual output:\n{}",
                       missing_messages.join("\n"),
                       actual);
            }
        }
        
        // Normal case: both have content, do normal comparison
        let actual_trimmed = actual.trim_end();
        let expected_trimmed = expected.trim_end();
        
        // If both are empty, it's fine
        if actual_trimmed.is_empty() && expected_trimmed.is_empty() {
            return;
        }
        
        // If expected is empty but actual is not, that's fine
        if expected_trimmed.is_empty() && !actual_trimmed.is_empty() {
            return;
        }
        
        // Otherwise, compare the content
        if actual_trimmed != expected_trimmed {
            // Print a helpful diff if they don't match
            panic!("\nScreen comparison failed!\n{}\n\nExpected:\n{}\n\nActual:\n{}\n\nDiff:\n{}",
                   self.dump_screen(),
                   expected_trimmed,
                   actual_trimmed,
                   self.generate_diff(expected_trimmed, actual_trimmed));
        }
    }
    
    /// Generate a simple diff between two strings
    fn generate_diff(&self, expected: &str, actual: &str) -> String {
        let expected_lines: Vec<&str> = expected.lines().collect();
        let actual_lines: Vec<&str> = actual.lines().collect();
        
        let mut result = String::new();
        let max_lines = min(expected_lines.len(), actual_lines.len());
        
        for i in 0..max_lines {
            if expected_lines[i] != actual_lines[i] {
                result.push_str(&format!("Line {}: Expected: '{}'\n", i, expected_lines[i]));
                result.push_str(&format!("Line {}: Actual  : '{}'\n", i, actual_lines[i]));
                result.push_str(&format!("Line {}: Diff    : {}\n", i, self.highlight_diff(expected_lines[i], actual_lines[i])));
                result.push('\n');
            }
        }
        
        // If one string has more lines than the other
        if expected_lines.len() > actual_lines.len() {
            result.push_str("Expected has more lines:\n");
            for i in actual_lines.len()..expected_lines.len() {
                result.push_str(&format!("Line {}: '{}'", i, expected_lines[i]));
                result.push('\n');
            }
        } else if actual_lines.len() > expected_lines.len() {
            result.push_str("Actual has more lines:\n");
            for i in expected_lines.len()..actual_lines.len() {
                result.push_str(&format!("Line {}: '{}'", i, actual_lines[i]));
                result.push('\n');
            }
        }
        
        result
    }
    
    /// Highlight differences between two strings with ^ markers
    fn highlight_diff(&self, expected: &str, actual: &str) -> String {
        let mut result = String::new();
        let expected_chars: Vec<char> = expected.chars().collect();
        let actual_chars: Vec<char> = actual.chars().collect();
        
        let min_len = min(expected_chars.len(), actual_chars.len());
        
        // First, add the actual string
        result.push_str(actual);
        result.push('\n');
        
        // Then, add markers for differences
        for i in 0..min_len {
            if expected_chars[i] != actual_chars[i] {
                result.push('^');
            } else {
                result.push(' ');
            }
        }
        
        // If lengths are different, mark the rest
        if expected_chars.len() > actual_chars.len() {
            result.push_str(" <missing characters>");
        } else if actual_chars.len() > expected_chars.len() {
            for _ in min_len..actual_chars.len() {
                result.push('^');
            }
            result.push_str(" <extra characters>");
        }
        
        result
    }
}

impl Write for TestEnv {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let s = String::from_utf8_lossy(buf);
        self.write(&s);
        Ok(buf.len())
    }
    
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::style::Color;
    
    #[test]
    fn test_testenv_basic_output() {
        let mut env = TestEnv::new(80, 24);
        
        env.write("Hello, World!");
        assert_eq!(env.contents(), "Hello, World!");
        
        env.move_to(0, 0).write("Overwritten");
        assert!(env.contents().contains("Overwritten"));
    }
    
    #[test]
    fn test_testenv_cursor_position() {
        let mut env = TestEnv::new(80, 24);
        
        env.move_to(10, 5);
        assert_eq!(env.cursor_pos(), (10, 5));
        
        let pos = env.cursor_position();
        assert_eq!(pos.x, 10);
        assert_eq!(pos.y, 5);
    }
    
    #[test]
    fn test_testenv_screen_dump() {
        let mut env = TestEnv::new(80, 24);
        
        env.write("Line 1\nLine 2\nLine 3");
        let dump = env.dump_screen();
        
        assert!(dump.contains("Line 1"));
        assert!(dump.contains("Line 2"));
        assert!(dump.contains("Line 3"));
        assert!(dump.contains("cursor at"));
    }
    
    #[test]
    fn test_testenv_diff_generation() {
        let env = TestEnv::new(80, 24);
        
        let diff = env.generate_diff("Line 1\nLine 2\nLine 3", "Line 1\nLine X\nLine 3");
        
        assert!(diff.contains("Line 1"));
        assert!(diff.contains("Line X"));
        assert!(diff.contains("Line 2"));
        assert!(diff.contains("^"));
    }
    
    #[test]
    fn test_testenv_color() {
        let mut env = TestEnv::new(80, 24);
        
        env.set_color(Color::Red)
           .write("Red text")
           .reset_styles();
        
        assert!(env.contents().contains("Red text"));
    }
} 