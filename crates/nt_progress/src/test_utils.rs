use std::io::Write;
use vt100::Parser;

/// A test environment for terminal output testing
pub struct TestEnv {
    parser: Parser,
    pub expected: Vec<String>,
    width: u16,
    height: u16,
}

impl TestEnv {
    /// Create a new test environment with the specified terminal size
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            parser: Parser::new(height, width, 0),
            expected: Vec::new(),
            width,
            height,
        }
    }

    /// Create a new test environment with the same dimensions as another
    pub fn new_like(other: &TestEnv) -> Self {
        Self::new(other.width, other.height)
    }

    /// Merge another test environment's output into this one
    pub fn merge(&mut self, other: TestEnv) {
        self.expected.extend(other.expected);
    }

    /// Get the current terminal contents
    pub fn contents(&self) -> String {
        let mut result = String::new();
        let mut current_line = String::new();
        
        for row in 0..self.height {
            let lines = self.parser.screen().rows(row, 1);
            for line in lines {
                if !line.trim().is_empty() {
                    current_line.push_str(&line);
                }
            }
            if !current_line.is_empty() {
                result.push_str(&current_line);
                result.push('\n');
                current_line.clear();
            }
        }
        result.trim_end().to_string()
    }

    /// Get the current cursor position
    pub fn cursor_pos(&self) -> (u16, u16) {
        let pos = self.parser.screen().cursor_position();
        (pos.0 as u16, pos.1 as u16)
    }

    /// Get the terminal size
    pub fn size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    /// Write text to the terminal
    pub fn write(&mut self, text: &str) -> &mut Self {
        self.parser.process(text.as_bytes());
        self.expected.push(text.to_string());
        self
    }

    /// Write a line to the terminal
    pub fn writeln(&mut self, text: &str) -> &mut Self {
        self.expected.push(format!("{}\n", text));
        self.parser.process(format!("{}\n", text).as_bytes());
        self
    }

    /// Clear the terminal
    pub fn clear(&mut self) -> &mut Self {
        self.parser.process(b"\x1B[2J");
        self.expected.clear();
        self
    }

    /// Move the cursor to a specific position
    pub fn move_to(&mut self, x: u16, y: u16) -> &mut Self {
        self.parser.process(format!("\x1B[{};{}H", y + 1, x + 1).as_bytes());
        self
    }

    /// Set the foreground color
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

    /// Reset all styles
    pub fn reset_styles(&mut self) -> &mut Self {
        self.parser.process(b"\x1B[0m");
        self
    }

    /// Verify the current output matches the expected output
    pub fn verify(&self) {
        let actual = self.contents();
        let expected = self.expected.join("");
        assert_eq!(actual, expected, "\nExpected:\n{}\n\nActual:\n{}\n", expected, actual);
    }
}

impl Write for TestEnv {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.parser.process(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
} 