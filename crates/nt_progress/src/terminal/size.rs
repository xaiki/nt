use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;

/// Represents a terminal with its properties and capabilities
/// 
/// The Terminal struct is responsible for detecting and tracking terminal size,
/// as well as providing methods to check for terminal features and capabilities.
pub struct Terminal {
    /// Current terminal size (width, height) in columns and rows
    size: Arc<Mutex<(u16, u16)>>,
    /// Whether this terminal supports ANSI color codes
    supports_color: bool,
    /// Whether this terminal supports cursor movement
    supports_cursor_movement: bool,
}

impl Terminal {
    /// Creates a new Terminal instance with default size and capabilities
    pub fn new() -> Self {
        Self {
            size: Arc::new(Mutex::new((80, 24))), // Default terminal size
            supports_color: true,                 // Assume ANSI color support
            supports_cursor_movement: true,       // Assume cursor movement support
        }
    }
    
    /// Creates a new Terminal instance with the specified size
    pub fn with_size(width: u16, height: u16) -> Self {
        Self {
            size: Arc::new(Mutex::new((width, height))),
            supports_color: true,
            supports_cursor_movement: true,
        }
    }
    
    /// Gets a cloneable reference to the terminal size
    pub fn size_ref(&self) -> Arc<Mutex<(u16, u16)>> {
        Arc::clone(&self.size)
    }
    
    /// Gets the current terminal size
    /// 
    /// This method acquires a lock on the size field, so it should be used
    /// sparingly in performance-critical code.
    pub async fn size(&self) -> (u16, u16) {
        *self.size.lock().await
    }
    
    /// Sets the terminal size
    /// 
    /// This can be used to manually update the terminal size or for testing.
    pub async fn set_size(&self, width: u16, height: u16) -> Result<()> {
        let mut size = self.size.lock().await;
        *size = (width, height);
        Ok(())
    }
    
    /// Detects the current terminal size using crossterm
    /// 
    /// This method updates the internal size field with the actual terminal size.
    pub async fn detect_size(&self) -> Result<(u16, u16)> {
        let (width, height) = crossterm::terminal::size()?;
        let mut size = self.size.lock().await;
        *size = (width, height);
        Ok((width, height))
    }
    
    /// Checks if the terminal supports ANSI colors
    pub fn supports_color(&self) -> bool {
        self.supports_color
    }
    
    /// Checks if the terminal supports cursor movement
    pub fn supports_cursor_movement(&self) -> bool {
        self.supports_cursor_movement
    }
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new()
    }
}

/// Unit tests for the Terminal struct
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_terminal_size() {
        let terminal = Terminal::new();
        let (width, height) = terminal.size().await;
        assert_eq!(width, 80);
        assert_eq!(height, 24);
        
        terminal.set_size(100, 40).await.unwrap();
        let (width, height) = terminal.size().await;
        assert_eq!(width, 100);
        assert_eq!(height, 40);
    }
    
    #[tokio::test]
    async fn test_size_ref() {
        let terminal = Terminal::new();
        let size_ref = terminal.size_ref();
        
        // Modify through size_ref
        {
            let mut size = size_ref.lock().await;
            *size = (120, 50);
        }
        
        // Verify the change is reflected in the terminal
        let (width, height) = terminal.size().await;
        assert_eq!(width, 120);
        assert_eq!(height, 50);
    }
} 