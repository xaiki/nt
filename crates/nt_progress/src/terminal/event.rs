use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use anyhow::Result;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyModifiers, KeyEvent, MouseEvent, MouseEventKind};
use futures::StreamExt; // Add this for EventStream support

#[cfg(test)]
use crate::terminal::test_helpers::with_timeout;

/// Represents various terminal events that can be handled
#[derive(Debug, Clone)]
pub enum TerminalEvent {
    /// Terminal window has been resized
    Resize {
        /// New width in columns
        width: u16,
        /// New height in rows
        height: u16,
    },
    /// A key has been pressed
    KeyPress(KeyData),
    /// A mouse event has occurred
    MouseEvent(MouseData),
    /// Terminal has lost focus
    FocusLost,
    /// Terminal has gained focus
    FocusGained,
    /// A control code or special sequence was received
    ControlCode(String),
}

/// Key press data with information about the key and modifiers
#[derive(Debug, Clone)]
pub struct KeyData {
    /// The key code that was pressed
    pub code: KeyCode,
    /// Any modifier keys held during the press
    pub modifiers: KeyModifiers,
    /// The character represented, if applicable
    pub char: Option<char>,
    /// Whether this is a key release event (false means key press)
    pub is_release: bool,
}

/// Mouse event data
#[derive(Debug, Clone)]
pub struct MouseData {
    /// Type of mouse event (press, release, drag)
    pub kind: MouseEventKind,
    /// Column position (x-coordinate)
    pub column: u16,
    /// Row position (y-coordinate)
    pub row: u16,
    /// Any modifier keys held during the mouse event
    pub modifiers: KeyModifiers,
}

impl KeyData {
    /// Creates a new KeyData from a crossterm KeyEvent
    pub fn from_key_event(event: KeyEvent) -> Self {
        let char = match event.code {
            KeyCode::Char(c) => Some(c),
            _ => None,
        };
        
        Self {
            code: event.code,
            modifiers: event.modifiers,
            char,
            is_release: false, // Crossterm only provides press events
        }
    }
    
    /// Checks if the key is a control key (Ctrl+key)
    pub fn is_control(&self) -> bool {
        self.modifiers.contains(KeyModifiers::CONTROL)
    }
    
    /// Checks if the key is an alt key (Alt+key)
    pub fn is_alt(&self) -> bool {
        self.modifiers.contains(KeyModifiers::ALT)
    }
    
    /// Checks if the key is a shift key (Shift+key)
    pub fn is_shift(&self) -> bool {
        self.modifiers.contains(KeyModifiers::SHIFT)
    }
    
    /// Checks if the key is the Escape key
    pub fn is_escape(&self) -> bool {
        matches!(self.code, KeyCode::Esc)
    }
    
    /// Checks if the key is a function key (F1-F12)
    pub fn is_function_key(&self) -> bool {
        matches!(self.code,
            KeyCode::F(1) | KeyCode::F(2) | KeyCode::F(3) | KeyCode::F(4) |
            KeyCode::F(5) | KeyCode::F(6) | KeyCode::F(7) | KeyCode::F(8) |
            KeyCode::F(9) | KeyCode::F(10) | KeyCode::F(11) | KeyCode::F(12)
        )
    }
    
    /// Checks if the key is an arrow key
    pub fn is_arrow_key(&self) -> bool {
        matches!(self.code,
            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right
        )
    }
    
    /// Checks if the key is a special key
    pub fn is_special_key(&self) -> bool {
        matches!(self.code,
            KeyCode::Backspace | KeyCode::Enter | KeyCode::Home | KeyCode::End |
            KeyCode::PageUp | KeyCode::PageDown | KeyCode::Tab | KeyCode::BackTab |
            KeyCode::Delete | KeyCode::Insert
        )
    }
}

impl MouseData {
    /// Creates a new MouseData from a crossterm MouseEvent
    pub fn from_mouse_event(event: MouseEvent) -> Self {
        Self {
            kind: event.kind,
            column: event.column,
            row: event.row,
            modifiers: event.modifiers,
        }
    }
    
    /// Checks if this is a mouse press event
    pub fn is_press(&self) -> bool {
        matches!(self.kind, 
            MouseEventKind::Down(_)
        )
    }
    
    /// Checks if this is a mouse release event
    pub fn is_release(&self) -> bool {
        matches!(self.kind, 
            MouseEventKind::Up(_)
        )
    }
    
    /// Checks if this is a mouse drag event
    pub fn is_drag(&self) -> bool {
        matches!(self.kind, 
            MouseEventKind::Drag(_)
        )
    }
    
    /// Checks if this is a scroll event
    pub fn is_scroll(&self) -> bool {
        matches!(self.kind, 
            MouseEventKind::ScrollDown | 
            MouseEventKind::ScrollUp
        )
    }
}

impl fmt::Display for TerminalEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TerminalEvent::Resize { width, height } => write!(f, "Resize({}, {})", width, height),
            TerminalEvent::KeyPress(key) => {
                let char_str = key.char.map(|c| c.to_string()).unwrap_or_default();
                write!(f, "KeyPress({:?}, {:?}, {})", key.code, key.modifiers, char_str)
            },
            TerminalEvent::MouseEvent(mouse) => {
                write!(f, "MouseEvent({:?} at {},{}, {:?})", mouse.kind, mouse.column, mouse.row, mouse.modifiers)
            },
            TerminalEvent::FocusLost => write!(f, "FocusLost"),
            TerminalEvent::FocusGained => write!(f, "FocusGained"),
            TerminalEvent::ControlCode(code) => write!(f, "ControlCode({})", code),
        }
    }
}

/// Type for event handler callbacks
pub type EventHandler = Box<dyn Fn(TerminalEvent) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync>;

/// Manager for terminal events
/// 
/// Handles event detection, dispatching, and listener registration
pub struct EventManager {
    /// Channel for sending events
    event_tx: mpsc::Sender<TerminalEvent>,
    /// Channel for receiving events
    event_rx: Arc<Mutex<mpsc::Receiver<TerminalEvent>>>,
    /// Registered event handlers
    handlers: Arc<Mutex<Vec<EventHandler>>>,
    /// Whether the event manager is currently running
    running: Arc<Mutex<bool>>,
    /// Polling interval in milliseconds
    poll_interval_ms: u64,
    /// Whether mouse events are enabled
    mouse_events_enabled: Arc<Mutex<bool>>,
}

impl EventManager {
    /// Creates a new event manager
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            event_tx: tx,
            event_rx: Arc::new(Mutex::new(rx)),
            handlers: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(Mutex::new(false)),
            poll_interval_ms: 100,
            mouse_events_enabled: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Creates a new event manager with a custom polling interval
    pub fn with_poll_interval(poll_interval_ms: u64) -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            event_tx: tx,
            event_rx: Arc::new(Mutex::new(rx)),
            handlers: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(Mutex::new(false)),
            poll_interval_ms,
            mouse_events_enabled: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Enables or disables mouse event handling
    pub async fn set_mouse_events_enabled(&self, enabled: bool) -> Result<()> {
        let mut mouse_enabled = self.mouse_events_enabled.lock().await;
        *mouse_enabled = enabled;
        
        // Enable or disable mouse capture in crossterm
        if enabled {
            crossterm::execute!(
                std::io::stdout(),
                crossterm::event::EnableMouseCapture
            )?;
        } else {
            crossterm::execute!(
                std::io::stdout(),
                crossterm::event::DisableMouseCapture
            )?;
        }
        
        Ok(())
    }
    
    /// Gets whether mouse events are enabled
    pub async fn mouse_events_enabled(&self) -> bool {
        *self.mouse_events_enabled.lock().await
    }
    
    /// Sets the polling interval for the event loop
    pub fn set_poll_interval(&mut self, interval_ms: u64) {
        self.poll_interval_ms = interval_ms;
    }
    
    /// Registers an event handler function
    pub async fn register_handler<F, Fut>(&self, handler: F) -> Result<()>
    where
        F: Fn(TerminalEvent) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let boxed_handler: EventHandler = Box::new(move |event| {
            Box::pin(handler(event))
        });
        
        let mut handlers = self.handlers.lock().await;
        handlers.push(boxed_handler);
        
        Ok(())
    }
    
    /// Starts the event detection loop
    pub async fn start_event_loop(&self) -> Result<()> {
        let mut running = self.running.lock().await;
        if *running {
            return Ok(());  // Already running
        }
        
        *running = true;
        drop(running); // Release the lock before spawning tasks
        
        // Clone what we need for the event loop
        let event_tx = self.event_tx.clone();
        let running_arc = Arc::clone(&self.running);
        let poll_interval = self.poll_interval_ms;
        let mouse_events_enabled = Arc::clone(&self.mouse_events_enabled);
        
        // Spawn a task to listen for terminal events
        let event_loop_handle = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            
            loop {
                // Check if we should stop
                {
                    let running = running_arc.lock().await;
                    if !*running {
                        break;
                    }
                }
                
                // Wait for the next event with a timeout
                if let Ok(Some(event)) = tokio::time::timeout(
                    tokio::time::Duration::from_millis(poll_interval),
                    reader.next()
                ).await {
                    if let Ok(crossterm_event) = event {
                        // Convert crossterm event to our TerminalEvent type
                        if let Some(terminal_event) = convert_event(crossterm_event, *mouse_events_enabled.lock().await).await {
                            // Send the event to our channel
                            if let Err(e) = event_tx.send(terminal_event).await {
                                if !*running_arc.lock().await {
                                    break; // Normal shutdown
                                }
                                eprintln!("Error sending terminal event: {}", e);
                                break;
                            }
                        }
                    }
                }
            }
        });
        
        // Spawn a task to process and dispatch events
        let event_rx = Arc::clone(&self.event_rx);
        let handlers = Arc::clone(&self.handlers);
        let running_arc = Arc::clone(&self.running);
        
        let dispatch_handle = tokio::spawn(async move {
            let mut event_rx = event_rx.lock().await;
            
            loop {
                // Check if we should stop
                {
                    let running = running_arc.lock().await;
                    if !*running {
                        break;
                    }
                }
                
                // Wait for the next event with a timeout
                if let Ok(Some(event)) = tokio::time::timeout(
                    tokio::time::Duration::from_millis(poll_interval),
                    event_rx.recv()
                ).await {
                    // Process the event with all handlers
                    let handlers_lock = handlers.lock().await;
                    
                    // Call each handler with the event
                    for handler in handlers_lock.iter() {
                        let event_clone = event.clone();
                        if let Err(e) = handler(event_clone).await {
                            eprintln!("Error in event handler: {}", e);
                        }
                    }
                }
            }
        });
        
        // Wait for both tasks to complete when stopping
        tokio::spawn(async move {
            let _ = tokio::join!(event_loop_handle, dispatch_handle);
        });
        
        Ok(())
    }
    
    /// Stops the event detection loop
    pub async fn stop_event_loop(&self) -> Result<()> {
        let mut running = self.running.lock().await;
        *running = false;
        
        // Wait a bit for tasks to clean up
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        Ok(())
    }
    
    /// Directly emit an event for testing purposes
    pub async fn emit_event(&self, event: TerminalEvent) -> Result<()> {
        self.event_tx.send(event).await?;
        Ok(())
    }
    
    /// Get a sender that can be used to emit events
    pub fn event_sender(&self) -> mpsc::Sender<TerminalEvent> {
        self.event_tx.clone()
    }
}

impl Default for EventManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Converts a crossterm event to our TerminalEvent type
async fn convert_event(event: CrosstermEvent, mouse_events_enabled: bool) -> Option<TerminalEvent> {
    match event {
        CrosstermEvent::Resize(width, height) => {
            Some(TerminalEvent::Resize { width, height })
        },
        CrosstermEvent::Key(key_event) => {
            Some(TerminalEvent::KeyPress(KeyData::from_key_event(key_event)))
        },
        CrosstermEvent::Mouse(mouse_event) => {
            if mouse_events_enabled {
                Some(TerminalEvent::MouseEvent(MouseData::from_mouse_event(mouse_event)))
            } else {
                None // Ignore mouse events if they're not enabled
            }
        },
        CrosstermEvent::FocusGained => {
            Some(TerminalEvent::FocusGained)
        },
        CrosstermEvent::FocusLost => {
            Some(TerminalEvent::FocusLost)
        },
        CrosstermEvent::Paste(text) => {
            // Paste events are converted to ControlCode events
            Some(TerminalEvent::ControlCode(format!("PASTE:{}", text)))
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_event_manager_creation() {
        with_timeout(async {
            let manager = EventManager::new();
            assert!(!*manager.running.lock().await);
        }, 30).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_event_emission() {
        with_timeout(async {
            let manager = EventManager::new();
            let event_handled = Arc::new(AtomicBool::new(false));
            
            // Register a handler
            {
                let event_handled_clone = Arc::clone(&event_handled);
                manager.register_handler(move |event| {
                    let event_handled = Arc::clone(&event_handled_clone);
                    async move {
                        if let TerminalEvent::Resize { width, height } = event {
                            assert_eq!(width, 100);
                            assert_eq!(height, 50);
                            event_handled.store(true, Ordering::SeqCst);
                        }
                        Ok(())
                    }
                }).await.unwrap();
            }
            
            // Start the event loop
            manager.start_event_loop().await.unwrap();
            
            // Emit a test event
            manager.emit_event(TerminalEvent::Resize { width: 100, height: 50 }).await.unwrap();
            
            // Wait a bit for the event to be processed
            tokio::time::sleep(Duration::from_millis(50)).await;
            
            // Stop the event loop and wait for it to complete
            manager.stop_event_loop().await.unwrap();
            tokio::time::sleep(Duration::from_millis(50)).await;
            
            // Verify the event was handled
            assert!(event_handled.load(Ordering::SeqCst));
        }, 30).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_multiple_handlers() {
        with_timeout(async {
            let manager = EventManager::new();
            let counter1 = Arc::new(AtomicBool::new(false));
            let counter2 = Arc::new(AtomicBool::new(false));
            
            // Register two handlers
            {
                let counter = Arc::clone(&counter1);
                manager.register_handler(move |_| {
                    let counter = Arc::clone(&counter);
                    async move {
                        counter.store(true, Ordering::SeqCst);
                        Ok(())
                    }
                }).await.unwrap();
            }
            
            {
                let counter = Arc::clone(&counter2);
                manager.register_handler(move |_| {
                    let counter = Arc::clone(&counter);
                    async move {
                        counter.store(true, Ordering::SeqCst);
                        Ok(())
                    }
                }).await.unwrap();
            }
            
            // Start the event loop
            manager.start_event_loop().await.unwrap();
            
            // Emit a test event
            manager.emit_event(TerminalEvent::FocusGained).await.unwrap();
            
            // Wait a bit for the event to be processed
            tokio::time::sleep(Duration::from_millis(50)).await;
            
            // Stop the event loop and wait for it to complete
            manager.stop_event_loop().await.unwrap();
            tokio::time::sleep(Duration::from_millis(50)).await;
            
            // Verify both handlers were called
            assert!(counter1.load(Ordering::SeqCst));
            assert!(counter2.load(Ordering::SeqCst));
        }, 30).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_key_data_methods() {
        with_timeout(async {
            // Test various key data methods
            let key_data = KeyData {
                code: KeyCode::Char('a'),
                modifiers: KeyModifiers::CONTROL,
                char: Some('a'),
                is_release: false,
            };
            
            assert!(key_data.is_control());
            assert!(!key_data.is_alt());
            assert!(!key_data.is_shift());
            assert!(!key_data.is_escape());
            assert!(!key_data.is_function_key());
            assert!(!key_data.is_arrow_key());
            assert!(!key_data.is_special_key());
            
            // Test various key types
            let esc_key = KeyData {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::empty(),
                char: None,
                is_release: false,
            };
            assert!(esc_key.is_escape());
            
            let f1_key = KeyData {
                code: KeyCode::F(1),
                modifiers: KeyModifiers::empty(),
                char: None,
                is_release: false,
            };
            assert!(f1_key.is_function_key());
            
            let arrow_key = KeyData {
                code: KeyCode::Up,
                modifiers: KeyModifiers::empty(),
                char: None,
                is_release: false,
            };
            assert!(arrow_key.is_arrow_key());
            
            let special_key = KeyData {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::empty(),
                char: None,
                is_release: false,
            };
            assert!(special_key.is_special_key());
        }, 30).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_mouse_data_methods() {
        with_timeout(async {
            // Test press event
            let press_event = MouseData {
                kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
                column: 10,
                row: 20,
                modifiers: KeyModifiers::empty(),
            };
            
            assert!(press_event.is_press());
            assert!(!press_event.is_release());
            assert!(!press_event.is_drag());
            assert!(!press_event.is_scroll());
            
            // Test release event
            let release_event = MouseData {
                kind: MouseEventKind::Up(crossterm::event::MouseButton::Left),
                column: 10,
                row: 20,
                modifiers: KeyModifiers::empty(),
            };
            
            assert!(!release_event.is_press());
            assert!(release_event.is_release());
            assert!(!release_event.is_drag());
            assert!(!release_event.is_scroll());
            
            // Test drag event
            let drag_event = MouseData {
                kind: MouseEventKind::Drag(crossterm::event::MouseButton::Left),
                column: 10,
                row: 20,
                modifiers: KeyModifiers::empty(),
            };
            
            assert!(!drag_event.is_press());
            assert!(!drag_event.is_release());
            assert!(drag_event.is_drag());
            assert!(!drag_event.is_scroll());
            
            // Test scroll event
            let scroll_event = MouseData {
                kind: MouseEventKind::ScrollDown,
                column: 10,
                row: 20,
                modifiers: KeyModifiers::empty(),
            };
            
            assert!(!scroll_event.is_press());
            assert!(!scroll_event.is_release());
            assert!(!scroll_event.is_drag());
            assert!(scroll_event.is_scroll());
        }, 30).await.unwrap();
    }
} 