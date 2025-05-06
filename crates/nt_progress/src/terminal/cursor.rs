use std::fmt;
use std::io::Write;
use anyhow::Result;

/// Represents a cursor position in the terminal
/// 
/// The position is zero-indexed, with (0, 0) being the top-left corner.
/// x represents the column (horizontal position)
/// y represents the row (vertical position)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorPosition {
    /// Column position (horizontal, x-coordinate)
    pub x: u16,
    /// Row position (vertical, y-coordinate)
    pub y: u16,
}

impl CursorPosition {
    /// Creates a new cursor position
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
    
    /// Creates a cursor position at the origin (0, 0)
    pub fn origin() -> Self {
        Self { x: 0, y: 0 }
    }
    
    /// Returns the position as a tuple (x, y)
    pub fn as_tuple(&self) -> (u16, u16) {
        (self.x, self.y)
    }
    
    /// Moves the cursor position by the given offsets
    pub fn offset(&self, x_offset: i16, y_offset: i16) -> Self {
        let new_x = if x_offset >= 0 {
            self.x.saturating_add(x_offset as u16)
        } else {
            self.x.saturating_sub((-x_offset) as u16)
        };
        
        let new_y = if y_offset >= 0 {
            self.y.saturating_add(y_offset as u16)
        } else {
            self.y.saturating_sub((-y_offset) as u16)
        };
        
        Self { x: new_x, y: new_y }
    }
    
    /// Moves the cursor position right by the given amount
    pub fn right(&self, amount: u16) -> Self {
        self.offset(amount as i16, 0)
    }
    
    /// Moves the cursor position left by the given amount
    pub fn left(&self, amount: u16) -> Self {
        self.offset(-(amount as i16), 0)
    }
    
    /// Moves the cursor position down by the given amount
    pub fn down(&self, amount: u16) -> Self {
        self.offset(0, amount as i16)
    }
    
    /// Moves the cursor position up by the given amount
    pub fn up(&self, amount: u16) -> Self {
        self.offset(0, -(amount as i16))
    }
    
    /// Applies the cursor position to the terminal
    /// 
    /// This uses crossterm to move the cursor to this position
    pub fn apply(&self) -> Result<()> {
        use crossterm::cursor::MoveTo;
        use crossterm::QueueableCommand;
        
        let mut stdout = std::io::stdout();
        stdout.queue(MoveTo(self.x, self.y))?;
        stdout.flush()?;
        
        Ok(())
    }
    
    /// Gets the current cursor position from the terminal
    pub fn get_current() -> Result<Self> {
        use crossterm::cursor::position;
        
        // Get the current terminal position (returns 1-indexed values)
        let (x, y) = position()?;
        
        // Convert to zero-indexed
        Ok(Self {
            x: x.saturating_sub(1),
            y: y.saturating_sub(1),
        })
    }
}

impl fmt::Display for CursorPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl From<(u16, u16)> for CursorPosition {
    fn from(tuple: (u16, u16)) -> Self {
        Self { x: tuple.0, y: tuple.1 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cursor_position_creation() {
        let pos = CursorPosition::new(5, 10);
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 10);
        
        let origin = CursorPosition::origin();
        assert_eq!(origin.x, 0);
        assert_eq!(origin.y, 0);
    }
    
    #[test]
    fn test_cursor_position_movement() {
        let pos = CursorPosition::new(5, 10);
        
        let right = pos.right(3);
        assert_eq!(right.x, 8);
        assert_eq!(right.y, 10);
        
        let left = pos.left(2);
        assert_eq!(left.x, 3);
        assert_eq!(left.y, 10);
        
        let down = pos.down(4);
        assert_eq!(down.x, 5);
        assert_eq!(down.y, 14);
        
        let up = pos.up(3);
        assert_eq!(up.x, 5);
        assert_eq!(up.y, 7);
    }
    
    #[test]
    fn test_cursor_position_offset() {
        let pos = CursorPosition::new(5, 10);
        
        let offset = pos.offset(2, -3);
        assert_eq!(offset.x, 7);
        assert_eq!(offset.y, 7);
        
        // Test underflow protection
        let underflow = pos.offset(-10, -20);
        assert_eq!(underflow.x, 0);
        assert_eq!(underflow.y, 0);
    }
    
    #[test]
    fn test_cursor_position_conversion() {
        let pos = CursorPosition::from((8, 12));
        assert_eq!(pos.x, 8);
        assert_eq!(pos.y, 12);
        
        let tuple = pos.as_tuple();
        assert_eq!(tuple, (8, 12));
    }
    
    #[test]
    fn test_cursor_position_display() {
        let pos = CursorPosition::new(5, 10);
        assert_eq!(format!("{}", pos), "(5, 10)");
    }
} 