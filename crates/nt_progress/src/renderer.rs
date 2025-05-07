use std::io::{self, Write};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::terminal::Terminal;
use std::collections::HashMap;

/// Responsible for rendering terminal output
pub struct Renderer {
    terminal: Arc<Terminal>,
    writer: Arc<Mutex<Box<dyn Write + Send + 'static>>>,
}

impl Renderer {
    /// Create a new renderer with default settings
    pub fn new() -> Self {
        Self {
            terminal: Arc::new(Terminal::new()),
            writer: Arc::new(Mutex::new(Box::new(std::io::stdout()))),
        }
    }
    
    /// Create a new renderer with a custom writer
    pub fn with_writer(writer: Box<dyn Write + Send + 'static>) -> Self {
        Self {
            terminal: Arc::new(Terminal::new()),
            writer: Arc::new(Mutex::new(writer)),
        }
    }
    
    /// Get a reference to the terminal
    pub fn terminal(&self) -> &Arc<Terminal> {
        &self.terminal
    }
    
    /// Render the provided thread outputs to the terminal
    pub async fn render(&self, outputs: &HashMap<usize, Vec<String>>) -> io::Result<()> {
        if outputs.is_empty() {
            return Ok(());
        }

        let mut writer = self.writer.lock().await;
        // Clear screen and move cursor to home position
        write!(writer, "\x1B[2J\x1B[1H")?;

        // Optimize for high concurrency by building the output in a single pass
        let mut sorted_threads: Vec<usize> = outputs.keys().cloned().collect();
        sorted_threads.sort(); // Sort by thread ID for consistent order
        
        // Pre-allocate a buffer for the output
        let mut buffer = String::with_capacity(outputs.len() * 50); // Reasonable initial capacity
        
        for thread_id in sorted_threads {
            if let Some(lines) = outputs.get(&thread_id) {
                for line in lines {
                    buffer.push_str(line);
                    buffer.push('\n');
                }
                // Add a blank line between thread outputs
                buffer.push('\n');
            }
        }
        
        // Write the entire buffer at once to minimize syscalls
        write!(writer, "{}", buffer)?;
        writer.flush()?;
        Ok(())
    }
    
    /// Stop the terminal event detection
    pub async fn stop(&self) -> anyhow::Result<()> {
        self.terminal.stop_event_detection().await
    }
} 