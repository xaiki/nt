use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use crate::terminal::event::{EventManager, TerminalEvent};

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
    /// Whether this terminal supports keyboard input
    supports_keyboard_input: bool,
    /// Whether this terminal supports mouse input
    supports_mouse: bool,
    /// Whether this terminal supports focus events
    supports_focus_events: bool,
    /// Whether this terminal is in raw mode (for interactive input)
    is_raw_mode: Arc<Mutex<bool>>,
    /// Event manager for handling terminal events
    event_manager: Arc<EventManager>,
}

impl Terminal {
    /// Creates a new Terminal instance with default size and capabilities
    pub fn new() -> Self {
        Self {
            size: Arc::new(Mutex::new((80, 24))), // Default terminal size
            supports_color: detect_color_support(),
            supports_cursor_movement: detect_cursor_support(),
            supports_keyboard_input: true,  // Most terminals support keyboard input
            supports_mouse: false,  // Conservative default
            supports_focus_events: false,  // Conservative default
            is_raw_mode: Arc::new(Mutex::new(false)),
            event_manager: Arc::new(EventManager::new()),
        }
    }
    
    /// Creates a new Terminal instance with the specified size
    pub fn with_size(width: u16, height: u16) -> Self {
        Self {
            size: Arc::new(Mutex::new((width, height))),
            supports_color: detect_color_support(),
            supports_cursor_movement: detect_cursor_support(),
            supports_keyboard_input: true,
            supports_mouse: false,
            supports_focus_events: false,
            is_raw_mode: Arc::new(Mutex::new(false)),
            event_manager: Arc::new(EventManager::new()),
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
        
        // Only emit resize event if the size is actually changing
        let old_size = *size;
        if old_size.0 != width || old_size.1 != height {
            *size = (width, height);
            
            // Emit a resize event
            self.event_manager.emit_event(TerminalEvent::Resize { width, height }).await?;
        } else {
            *size = (width, height);
        }
        
        Ok(())
    }
    
    /// Detects the current terminal size using crossterm
    /// 
    /// This method updates the internal size field with the actual terminal size.
    pub async fn detect_size(&self) -> Result<(u16, u16)> {
        let (width, height) = crossterm::terminal::size()?;
        
        // Get the old size first
        let old_size = *self.size.lock().await;
        
        // Update the size
        let mut size = self.size.lock().await;
        *size = (width, height);
        
        // Emit resize event if the size changed
        if old_size.0 != width || old_size.1 != height {
            self.event_manager.emit_event(TerminalEvent::Resize { width, height }).await?;
        }
        
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
    
    /// Checks if the terminal supports keyboard input
    pub fn supports_keyboard_input(&self) -> bool {
        self.supports_keyboard_input
    }
    
    /// Checks if the terminal supports mouse input
    pub fn supports_mouse(&self) -> bool {
        self.supports_mouse
    }
    
    /// Checks if the terminal supports focus events
    pub fn supports_focus_events(&self) -> bool {
        self.supports_focus_events
    }
    
    /// Checks if the terminal is currently in raw mode
    pub async fn is_raw_mode(&self) -> bool {
        *self.is_raw_mode.lock().await
    }
    
    /// Enables raw mode for the terminal (needed for interactive input)
    /// 
    /// This method puts the terminal in raw mode, which allows capturing
    /// individual keypresses without waiting for Enter to be pressed.
    pub async fn enable_raw_mode(&self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        let mut raw_mode = self.is_raw_mode.lock().await;
        *raw_mode = true;
        Ok(())
    }
    
    /// Disables raw mode for the terminal
    /// 
    /// This method returns the terminal to its normal mode.
    pub async fn disable_raw_mode(&self) -> Result<()> {
        crossterm::terminal::disable_raw_mode()?;
        let mut raw_mode = self.is_raw_mode.lock().await;
        *raw_mode = false;
        Ok(())
    }
    
    /// Enables mouse capture for mouse event detection
    pub async fn enable_mouse_capture(&self) -> Result<()> {
        crossterm::execute!(
            std::io::stdout(),
            crossterm::event::EnableMouseCapture
        )?;
        Ok(())
    }
    
    /// Disables mouse capture
    pub async fn disable_mouse_capture(&self) -> Result<()> {
        crossterm::execute!(
            std::io::stdout(),
            crossterm::event::DisableMouseCapture
        )?;
        Ok(())
    }
    
    /// Detects terminal capabilities by checking environment and features
    /// 
    /// This method updates internal capability flags based on actual terminal features.
    pub fn detect_capabilities(&mut self) -> Result<()> {
        // Detect color support
        self.supports_color = detect_color_support();
        
        // Detect cursor movement support
        self.supports_cursor_movement = detect_cursor_support();
        
        // Most terminals support keyboard input
        self.supports_keyboard_input = true;
        
        // For mouse and focus events, we'll be conservative
        // These can be enabled on demand if needed
        self.supports_mouse = false;
        self.supports_focus_events = false;
        
        Ok(())
    }
    
    /// Get a reference to the event manager
    pub fn event_manager(&self) -> &Arc<EventManager> {
        &self.event_manager
    }
    
    /// Start listening for terminal events
    pub async fn start_event_detection(&self) -> Result<()> {
        // Start the event manager
        self.event_manager.start_event_loop().await?;
        
        // Register a handler for resize events to update our size
        let size_ref = self.size_ref();
        
        self.event_manager.register_handler(move |event| {
            let size_ref = size_ref.clone();
            
            async move {
                if let TerminalEvent::Resize { width, height } = event {
                    // Update the size
                    let mut size = size_ref.lock().await;
                    *size = (width, height);
                }
                Ok(())
            }
        }).await?;
        
        Ok(())
    }
    
    /// Stop listening for terminal events
    pub async fn stop_event_detection(&self) -> Result<()> {
        self.event_manager.stop_event_loop().await
    }
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new()
    }
}

/// Detect if the terminal supports ANSI color codes
fn detect_color_support() -> bool {
    // Check for NO_COLOR environment variable (https://no-color.org/)
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }
    
    // Check for COLORTERM environment variable
    if let Ok(colorterm) = std::env::var("COLORTERM") {
        if colorterm == "truecolor" || colorterm == "24bit" {
            return true;
        }
    }
    
    // Check for common terminals that support color
    if let Ok(term) = std::env::var("TERM") {
        if term.contains("color") || term.contains("xterm") || term.contains("256") {
            return true;
        }
    }
    
    // Default to true for most modern terminals
    true
}

/// Detect if the terminal supports cursor movement
fn detect_cursor_support() -> bool {
    // Check for TERM environment variable
    if let Ok(term) = std::env::var("TERM") {
        // Most terminal types support cursor movement
        if term.contains("xterm") || term.contains("rxvt") || term.contains("screen") || term.contains("tmux") {
            return true;
        }
    }
    
    // Default to true for most modern terminals
    true
}

/// Unit tests for the Terminal struct
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;
    use crate::terminal::test_helpers::with_timeout;
    
    #[tokio::test]
    async fn test_terminal_size() {
        with_timeout(async {
            let terminal = Terminal::new();
            
            // Start event detection
            terminal.start_event_detection().await.unwrap();
            
            let (width, height) = terminal.size().await;
            assert_eq!(width, 80);
            assert_eq!(height, 24);
            
            terminal.set_size(100, 40).await.unwrap();
            let (width, height) = terminal.size().await;
            assert_eq!(width, 100);
            assert_eq!(height, 40);
            
            // Stop event detection
            terminal.stop_event_detection().await.unwrap();
            tokio::time::sleep(Duration::from_millis(50)).await;
        }, 30).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_size_ref() {
        with_timeout(async {
            let terminal = Terminal::new();
            
            // Start event detection
            terminal.start_event_detection().await.unwrap();
            
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
            
            // Stop event detection
            terminal.stop_event_detection().await.unwrap();
            tokio::time::sleep(Duration::from_millis(50)).await;
        }, 30).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_resize_event() {
        with_timeout(async {
            let terminal = Terminal::new();
            
            // Create a flag to track if the event was received
            let resize_received = Arc::new(AtomicBool::new(false));
            
            // Start event detection
            terminal.start_event_detection().await.unwrap();
            
            // Register a handler for resize events
            {
                let resize_received_clone = Arc::clone(&resize_received);
                terminal.event_manager().register_handler(move |event| {
                    let resize_flag = Arc::clone(&resize_received_clone);
                    async move {
                        if let TerminalEvent::Resize { width, height } = event {
                            assert_eq!(width, 100);
                            assert_eq!(height, 50);
                            resize_flag.store(true, Ordering::SeqCst);
                        }
                        Ok(())
                    }
                }).await.unwrap();
            }
            
            // Trigger a resize event
            terminal.set_size(100, 50).await.unwrap();
            
            // Wait a bit for the event to be processed
            tokio::time::sleep(Duration::from_millis(50)).await;
            
            // Stop event detection and wait for cleanup
            terminal.stop_event_detection().await.unwrap();
            tokio::time::sleep(Duration::from_millis(50)).await;
            
            // Verify the event was received
            assert!(resize_received.load(Ordering::SeqCst));
        }, 30).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_raw_mode() {
        with_timeout(async {
            let terminal = Terminal::new();
            
            // Check initial state
            assert!(!terminal.is_raw_mode().await);
            
            // We'll just test the flag toggling without actually enabling raw mode
            // since that would affect the test environment
            {
                let mut raw_mode = terminal.is_raw_mode.lock().await;
                *raw_mode = true;
            }
            
            assert!(terminal.is_raw_mode().await);
            
            {
                let mut raw_mode = terminal.is_raw_mode.lock().await;
                *raw_mode = false;
            }
            
            assert!(!terminal.is_raw_mode().await);
        }, 30).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_capability_detection() {
        with_timeout(async {
            let mut terminal = Terminal::new();
            
            // Just make sure the detection doesn't error
            let _ = terminal.detect_capabilities();
            
            // Check that the capabilities are set
            assert!(terminal.supports_keyboard_input());
        }, 30).await.unwrap();
    }
} 