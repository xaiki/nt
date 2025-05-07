#![deny(unused_imports)]
#![feature(internal_output_capture)]

use std::{
    collections::HashMap,
    io::Write,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
};
use anyhow::{Result, anyhow};
use std::fmt::Debug;
use std::cell::RefCell;
use crate::errors::{ContextExt, ErrorContext, ProgressError};
use crate::terminal::{Terminal, TerminalEvent};

pub mod modes;
pub mod errors;
pub mod formatter;
pub mod terminal;
#[cfg(test)]
pub mod tests;

pub use modes::{ThreadMode, ThreadConfig, Config, JobTracker, HasBaseConfig};
pub use modes::{ModeRegistry, ModeCreator, get_registry, create_thread_config};
pub use errors::ModeCreationError;
pub use formatter::{ProgressTemplate, TemplateContext, TemplateVar, TemplatePreset};

thread_local! {
    static CURRENT_THREAD_ID: AtomicUsize = AtomicUsize::new(0);
    static CURRENT_WRITER: RefCell<Option<ThreadLogger>> = RefCell::new(None);
}

#[derive(Debug, Clone)]
pub struct ThreadMessage {
    pub thread_id: usize,
    pub lines: Vec<String>,
    pub config: Config,
}

/// A display for tracking progress of multiple threads or tasks.
///
/// ProgressDisplay provides a central point for aggregating outputs from multiple
/// tasks and rendering them in different display modes.
///
/// # Safety and Resource Cleanup
///
/// Although ProgressDisplay implements Drop for safety, it's strongly recommended to 
/// call `stop()` explicitly when you're done with it, especially in tests:
///
/// ```rust
/// # async fn example() {
/// let display = ProgressDisplay::new().await;
///
/// // Use the display...
///
/// // Always call stop() explicitly when done
/// display.stop().await.unwrap();
/// # }
/// ```
///
/// # Test Best Practices
///
/// In tests, it's vital to follow this pattern to avoid hangs:
///
/// 1. Create ProgressDisplay OUTSIDE any timeout block
/// 2. Run test logic INSIDE a timeout block
/// 3. Call display.stop() OUTSIDE the timeout (to ensure cleanup even if timeout occurs)
///
/// ```rust
/// # use anyhow::Result;
/// # async fn test_example() -> Result<()> {
/// // 1. Create outside timeout
/// let display = ProgressDisplay::new().await;
///
/// // 2. Test inside timeout
/// with_timeout(async {
///     // Test logic here...
///     Ok(())
/// }, 3).await?;
///
/// // 3. Clean up outside timeout
/// display.stop().await?;
/// Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct ProgressDisplay {
    outputs: Arc<Mutex<HashMap<usize, Vec<String>>>>,
    spinner_index: Arc<AtomicUsize>,
    pub terminal: Arc<Terminal>,
    processing_task: Arc<Mutex<Option<JoinHandle<()>>>>,
    thread_handles: Arc<Mutex<HashMap<usize, JoinHandle<()>>>>,
    next_thread_id: Arc<AtomicUsize>,
    running: Arc<AtomicBool>,
    message_tx: mpsc::Sender<ThreadMessage>,
    message_rx: Arc<Mutex<mpsc::Receiver<ThreadMessage>>>,
    thread_configs: Arc<Mutex<HashMap<usize, Config>>>,
    writer: Arc<Mutex<Box<dyn Write + Send + 'static>>>,
}

impl Debug for ProgressDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProgressDisplay")
            .field("outputs", &self.outputs)
            .field("spinner_index", &self.spinner_index)
            .field("terminal", &"Terminal")
            .field("processing_task", &self.processing_task)
            .field("thread_handles", &self.thread_handles)
            .field("next_thread_id", &self.next_thread_id)
            .field("running", &self.running)
            .field("message_tx", &self.message_tx)
            .field("message_rx", &self.message_rx)
            .field("thread_configs", &self.thread_configs)
            .finish()
    }
}

impl ProgressDisplay {
    pub async fn new() -> Self {
        Self::new_with_mode_and_writer(ThreadMode::Limited, Box::new(std::io::stderr())).await
    }

    pub async fn new_with_mode(mode: ThreadMode) -> Self {
        Self::new_with_mode_and_writer(mode, Box::new(std::io::stderr())).await
    }

    pub async fn new_with_mode_and_writer<W: Write + Send + 'static>(mode: ThreadMode, writer: W) -> Self {
        let (message_tx, message_rx) = mpsc::channel(100);
        
        // Create a new Terminal instance
        let terminal = Arc::new(Terminal::new());
        
        let display = Self {
            outputs: Arc::new(Mutex::new(HashMap::new())),
            spinner_index: Arc::new(AtomicUsize::new(0)),
            terminal,
            processing_task: Arc::new(Mutex::new(None)),
            thread_handles: Arc::new(Mutex::new(HashMap::new())),
            next_thread_id: Arc::new(AtomicUsize::new(0)),
            running: Arc::new(AtomicBool::new(true)),
            message_tx,
            message_rx: Arc::new(Mutex::new(message_rx)),
            thread_configs: Arc::new(Mutex::new(HashMap::new())),
            writer: Arc::new(Mutex::new(Box::new(writer))),
        };

        // Start the terminal event detection
        if let Err(e) = display.terminal.start_event_detection().await {
            eprintln!("Warning: Failed to start terminal event detection: {}", e);
        }
        
        // Register a handler for terminal resize events to trigger display refresh
        let display_clone = display.clone();
        if let Err(e) = display.terminal.event_manager().register_handler(move |event| {
            let display = display_clone.clone();
            
            async move {
                match event {
                    TerminalEvent::Resize { width: _, height: _ } => {
                        // Resize happened, refresh the display
                        if let Err(e) = display.display().await {
                            eprintln!("Error refreshing display after resize: {}", e);
                        }
                    },
                    _ => {}, // Ignore other events for now
                }
                Ok(())
            }
        }).await {
            eprintln!("Warning: Failed to register resize event handler: {}", e);
        }

        // Start the display thread with a reference to the terminal
        let processing_task = tokio::spawn(Self::start_display_thread(
            Arc::clone(&display.outputs),
            Arc::clone(&display.terminal),
            Arc::clone(&display.spinner_index),
            Arc::clone(&display.message_rx),
            Arc::clone(&display.running),
            Arc::clone(&display.writer),
        ));
        *display.processing_task.lock().await = Some(processing_task);

        display
    }

    pub async fn spawn<F, R>(&self, f: F) -> Result<TaskHandle>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + std::fmt::Debug + 'static,
    {
        self.spawn_with_mode(ThreadMode::Limited, f).await
    }

    pub async fn spawn_with_mode<F, R>(&self, mode: ThreadMode, f: F) -> Result<TaskHandle>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + std::fmt::Debug + 'static,
    {
        let thread_id = self.next_thread_id.fetch_add(1, Ordering::SeqCst);
        let message_tx = self.message_tx.clone();
        let progress = Arc::new(self.clone());
        
        // Create the thread's config with specified mode
        let mut configs = self.thread_configs.lock().await;
        
        // Use context-aware error handling
        let config = Config::new(mode, 1)
            .with_context("creating thread config", "ProgressDisplay::spawn_with_mode")
            .map_err(|e: ProgressError| {
                let ctx = ErrorContext::new("spawning task", "ProgressDisplay")
                    .with_thread_id(thread_id)
                    .with_details(format!("with mode {:?}", mode));
                let err: anyhow::Error = ProgressError::WithContext(Box::new(e), ctx).into();
                err
            })?;
            
        configs.insert(thread_id, config.clone());
        
        let handle = tokio::spawn(async move {
            let output = f();
            let lines = vec![format!("{:?}", output)];
            if let Err(e) = message_tx.send(ThreadMessage { 
                thread_id, 
                lines,
                config,
            }).await {
                eprintln!("[Error] Failed to send message for thread {}: {}", thread_id, e);
            }
        });
        
        let mut handles = self.thread_handles.lock().await;
        handles.insert(thread_id, handle);
        
        Ok(TaskHandle::new(thread_id, progress))
    }

    pub async fn display(&self) -> std::io::Result<()> {
        Self::render_display(
            &self.outputs,
            &self.terminal,
            &self.spinner_index,
            &self.writer,
        ).await
    }

    pub async fn stop(&self) -> Result<()> {
        self.running.store(false, Ordering::SeqCst);
        self.join_all().await?;
        
        // Stop the terminal event detection
        if let Err(e) = self.terminal.stop_event_detection().await {
            eprintln!("Warning: Failed to stop terminal event detection: {}", e);
        }
        
        if let Some(handle) = self.processing_task.lock().await.take() {
            handle.await.map_err(|e| anyhow!("Failed to join processing task: {}", e))?;
        }
        Ok(())
    }

    pub async fn set_title(&self, thread_id: usize, title: String) -> Result<()> {
        let mut configs = self.thread_configs.lock().await;
        
        // Check if the thread exists
        if let Some(config) = configs.get_mut(&thread_id) {
            // Try to set the title using the capability trait
            match config.set_title(title) {
                Ok(()) => Ok(()),
                Err(_) => {
                    let ctx = ErrorContext::new("setting title", "ProgressDisplay")
                        .with_thread_id(thread_id)
                        .with_details("Thread is not in a mode that supports titles");
                    
                    let error_msg = format!("Thread {} is not in a mode that supports titles", thread_id);
                    let error = ProgressError::TaskOperation(error_msg).into_context(ctx);
                    Err(anyhow::Error::from(error))
                }
            }
        } else {
            let ctx = ErrorContext::new("setting title", "ProgressDisplay")
                .with_thread_id(thread_id)
                .with_details("Thread not found");
            
            let error_msg = format!("Thread {} not found", thread_id);
            let error = ProgressError::TaskOperation(error_msg).into_context(ctx);
            Err(anyhow::Error::from(error))
        }
    }

    pub async fn add_emoji(&self, thread_id: usize, emoji: &str) -> Result<()> {
        let mut configs = self.thread_configs.lock().await;
        
        // Check if the thread exists
        if let Some(config) = configs.get_mut(&thread_id) {
            // Try to add the emoji using the capability trait
            match config.add_emoji(emoji) {
                Ok(()) => Ok(()),
                Err(_) => {
                    let ctx = ErrorContext::new("adding emoji", "ProgressDisplay")
                        .with_thread_id(thread_id)
                        .with_details("Thread is not in a mode that supports emojis");
                    
                    let error_msg = format!("Thread {} is not in a mode that supports emojis", thread_id);
                    let error = ProgressError::TaskOperation(error_msg).into_context(ctx);
                    Err(anyhow::Error::from(error))
                }
            }
        } else {
            let ctx = ErrorContext::new("adding emoji", "ProgressDisplay")
                .with_thread_id(thread_id)
                .with_details("Thread not found");
            
            let error_msg = format!("Thread {} not found", thread_id);
            let error = ProgressError::TaskOperation(error_msg).into_context(ctx);
            Err(anyhow::Error::from(error))
        }
    }

    pub async fn set_total_jobs(&self, thread_id: Option<usize>, total: usize) -> Result<()> {
        if total == 0 {
            let ctx = ErrorContext::new("setting total jobs", "ProgressDisplay")
                .with_details("Total jobs cannot be zero");
            let error = ProgressError::DisplayOperation("Total jobs cannot be zero".to_string())
                .into_context(ctx);
            return Err(anyhow::Error::from(error));
        }
        
        let mut configs = self.thread_configs.lock().await;
        
        if let Some(thread_id) = thread_id {
            // Update a specific thread's total jobs
            if let Some(config) = configs.get_mut(&thread_id) {
                config.set_total_jobs(total);
                Ok(())
            } else {
                let ctx = ErrorContext::new("setting total jobs", "ProgressDisplay")
                    .with_thread_id(thread_id)
                    .with_details("Thread not found");
                
                let error_msg = format!("Thread {} not found", thread_id);
                let error = ProgressError::TaskOperation(error_msg).into_context(ctx);
                Err(anyhow::Error::from(error))
            }
        } else {
            // Update total jobs for all threads
            for (_, config) in configs.iter_mut() {
                config.set_total_jobs(total);
            }
            Ok(())
        }
    }

    pub async fn update_progress(&self, thread_id: usize, current: usize, total: usize, prefix: &str) -> Result<()> {
        if total == 0 {
            return Err(ProgressError::DisplayOperation("Total jobs cannot be zero".to_string()).into());
        }
        
        let progress_percent = ((current * 100) / total).min(100);
        let bar_width = 50;
        let filled = (progress_percent * bar_width) / 100;
        let bar = "▉".repeat(filled) + &"▏".repeat(bar_width - filled);
        let message = format!("{:<12} {}%|{}| {}/{}", prefix, progress_percent, bar, current, total);
        
        let mut outputs = self.outputs.lock().await;
        if let Some(lines) = outputs.get_mut(&thread_id) {
            if lines.is_empty() {
                lines.push(message);
            } else {
                lines[0] = message;
            }
        } else {
            outputs.insert(thread_id, vec![message]);
        }
        Ok(())
    }

    pub async fn join_all(&self) -> Result<()> {
        let handles = {
            let mut handles = self.thread_handles.lock().await;
            handles.drain().collect::<Vec<_>>()
        };
        
        for (thread_id, handle) in handles {
            handle.await.map_err(|e| anyhow!("Failed to join thread {}: {}", thread_id, e))?;
        }
        Ok(())
    }

    pub async fn cancel_all(&self) -> Result<()> {
        let handles = {
            let mut handles = self.thread_handles.lock().await;
            handles.drain().collect::<Vec<_>>()
        };
        
        for (thread_id, handle) in handles {
            handle.abort();
            eprintln!("[Info] Cancelled thread {}", thread_id);
        }
        Ok(())
    }

    pub async fn thread_count(&self) -> usize {
        let handles = self.thread_handles.lock().await;
        handles.len()
    }

    pub async fn get_task(&self, thread_id: usize) -> Option<TaskHandle> {
        let handles = self.thread_handles.lock().await;
        handles.get(&thread_id).map(|_handle| TaskHandle {
            thread_id,
            progress: Arc::new(self.clone()),
            message_tx: self.message_tx.clone(),
            thread_config: Arc::new(Mutex::new(Config::new(ThreadMode::Limited, 1).unwrap())),
            writer: self.writer.clone(),
        })
    }

    async fn start_display_thread(
        outputs: Arc<Mutex<HashMap<usize, Vec<String>>>>,
        terminal: Arc<Terminal>,
        spinner_index: Arc<AtomicUsize>,
        message_rx: Arc<Mutex<mpsc::Receiver<ThreadMessage>>>,
        running: Arc<AtomicBool>,
        writer: Arc<Mutex<Box<dyn Write + Send + 'static>>>,
    ) {
        // Detect the terminal size at startup
        if let Err(e) = terminal.detect_size().await {
            eprintln!("Warning: Failed to detect terminal size, using default: {}", e);
        }
        
        while running.load(Ordering::SeqCst) {
            // Process any pending messages
            let mut message_rx = message_rx.lock().await;
            while let Ok(message) = message_rx.try_recv() {
                let mut outputs = outputs.lock().await;
                outputs.entry(message.thread_id)
                    .or_insert_with(Vec::new)
                    .extend(message.lines);
            }

            // Update display
            let result = Self::render_display(
                &outputs,
                &terminal,
                &spinner_index,
                &writer,
            ).await;

            if let Err(e) = result {
                eprintln!("Error updating display: {}", e);
            }

            // Sleep a bit to prevent CPU spinning
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    // Private method for rendering the display
    async fn render_display(
        outputs: &Arc<Mutex<HashMap<usize, Vec<String>>>>,
        terminal: &Arc<Terminal>,
        _spinner_index: &Arc<AtomicUsize>,
        writer: &Arc<Mutex<Box<dyn Write + Send + 'static>>>,
    ) -> std::io::Result<()> {
        // Get all thread outputs and prepare for rendering
        let (thread_ids, lines_by_thread, line_count, terminal_height) = {
            let outputs_guard = outputs.lock().await;
            if outputs_guard.is_empty() {
                return Ok(());
            }
            
            // Use terminal to get the height
            let (_width, height) = terminal.size().await;
            
            // Get all thread IDs and sort them
            let mut thread_ids: Vec<usize> = outputs_guard.keys().copied().collect();
            thread_ids.sort();
            
            // Calculate total lines and adjust if needed
            let mut total_lines = 0;
            
            // First pass: calculate total lines
            for thread_id in &thread_ids {
                if let Some(lines) = outputs_guard.get(thread_id) {
                    total_lines += lines.len();
                }
            }
            
            // Clone the necessary data to avoid holding the lock
            let mut lines_by_thread = HashMap::new();
            for (thread_id, lines) in outputs_guard.iter() {
                lines_by_thread.insert(*thread_id, lines.clone());
            }
            
            (thread_ids, lines_by_thread, total_lines, height)
        };
        
        // Get a clone of the writer to avoid holding a lock for too long
        let mut writer_guard = writer.lock().await;
        
        // Clear the screen and format output
        let mut adjusted_lines_by_thread = lines_by_thread.clone();
        
        // Adjust line count if needed based on terminal height
        if line_count > terminal_height as usize && !adjusted_lines_by_thread.is_empty() {
            let available_lines = terminal_height as usize;
            let mut used_lines = 0;
            
            // Allocate at least one line per thread
            for thread_id in thread_ids.iter() {
                if let Some(lines) = adjusted_lines_by_thread.get_mut(thread_id) {
                    if !lines.is_empty() {
                        lines.truncate(1);
                        used_lines += 1;
                    }
                    
                    if used_lines >= available_lines {
                        break;
                    }
                }
            }
            
            // Allocate remaining lines proportionally
            if used_lines < available_lines {
                let remaining_lines = available_lines - used_lines;
                let lines_per_thread = remaining_lines / thread_ids.len();
                
                if lines_per_thread > 0 {
                    for thread_id in thread_ids.iter() {
                        if let Some(original_lines) = lines_by_thread.get(thread_id) {
                            if let Some(adjusted_lines) = adjusted_lines_by_thread.get_mut(thread_id) {
                                let original_count = original_lines.len();
                                
                                if original_count > 1 {
                                    let additional_lines = (lines_per_thread).min(original_count - 1);
                                    adjusted_lines.extend(original_lines.iter().skip(1).take(additional_lines).cloned());
                                    used_lines += additional_lines;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Clear previous output
        write!(writer_guard, "\x1B[2J\x1B[1;1H")?;
        
        // Format and display each thread's output
        for thread_id in thread_ids {
            if let Some(lines) = adjusted_lines_by_thread.get(&thread_id) {
                for line in lines {
                    writeln!(writer_guard, "{}", line)?;
                }
            }
        }
        
        writer_guard.flush()?;
        
        Ok(())
    }
}

impl Drop for ProgressDisplay {
    fn drop(&mut self) {
        // Signal that we're shutting down
        self.running.store(false, Ordering::SeqCst);
        
        // We can't run async code directly in drop, so we just do our best with synchronous cleanup
        
        // Try to abort all tasks we have handles for
        let _handles = {
            if let Ok(mut handles_lock) = self.thread_handles.try_lock() {
                // Collect and abort all handles
                for (_, handle) in handles_lock.drain() {
                    handle.abort();
                }
            }
        };
        
        // Try to abort the processing task
        if let Ok(mut guard) = self.processing_task.try_lock() {
            if let Some(handle) = guard.take() {
                handle.abort();
            }
        }
        
        // We can't stop the terminal event detection properly in drop
        // This is a last-ditch effort to warn users if they didn't call stop() explicitly
        eprintln!("Warning: ProgressDisplay dropped without calling stop() - some resources may not be fully cleaned up");
    }
}

#[derive(Debug, Clone)]
pub struct ThreadLogger {
    thread_id: usize,
    message_tx: mpsc::Sender<ThreadMessage>,
    config: Config,
}

impl ThreadLogger {
    pub fn new(thread_id: usize, message_tx: mpsc::Sender<ThreadMessage>, config: Config) -> Self {
        Self {
            thread_id,
            message_tx,
            config,
        }
    }

    pub async fn log(&mut self, message: String) -> Result<()> {
        let lines = self.config.handle_message(message);
        
        let message = ThreadMessage {
            thread_id: self.thread_id,
            lines,
            config: self.config.clone(),
        };
        
        self.message_tx.send(message).await.map_err(|e| anyhow!("Failed to send message: {}", e))
    }

    pub fn update_config(&mut self, config: Config) {
        self.config = config;
    }
}

pub struct TaskHandle {
    thread_id: usize,
    progress: Arc<ProgressDisplay>,
    message_tx: mpsc::Sender<ThreadMessage>,
    thread_config: Arc<Mutex<Config>>,
    writer: Arc<Mutex<Box<dyn Write + Send + 'static>>>,
}

impl Debug for TaskHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskHandle")
            .field("thread_id", &self.thread_id)
            .field("progress", &self.progress)
            .field("message_tx", &self.message_tx)
            .field("thread_config", &self.thread_config)
            .finish()
    }
}

impl TaskHandle {
    pub fn new(thread_id: usize, progress: Arc<ProgressDisplay>) -> Self {
        Self {
            thread_id,
            progress: progress.clone(),
            message_tx: progress.message_tx.clone(),
            thread_config: Arc::new(Mutex::new(Config::new(ThreadMode::Limited, 1).unwrap())),
            writer: progress.writer.clone(),
        }
    }

    pub async fn join(self) -> Result<()> {
        let handle = {
            let mut handles = self.progress.thread_handles.lock().await;
            handles.remove(&self.thread_id)
        };
        
        if let Some(handle) = handle {
            handle.await?;
        }
        Ok(())
    }

    pub async fn cancel(self) -> Result<()> {
        let handle = {
            let mut handles = self.progress.thread_handles.lock().await;
            handles.remove(&self.thread_id)
        };
        
        if let Some(handle) = handle {
            handle.abort();
        }
        Ok(())
    }

    pub fn thread_id(&self) -> usize {
        self.thread_id
    }

    pub async fn set_mode(&mut self, mode: ThreadMode) -> Result<()> {
        let mut config = self.thread_config.lock().await;
        *config = Config::new(mode, 1).unwrap();
        Ok(())
    }

    pub async fn capture_stdout(&mut self, line: String) -> Result<()> {
        let config = self.thread_config.lock().await.clone();
        self.message_tx.send(ThreadMessage {
            thread_id: self.thread_id,
            lines: vec![line],
            config,
        }).await.map_err(|e| anyhow!("Failed to send message: {}", e))?;
        Ok(())
    }

    pub async fn capture_stderr(&mut self, line: String) -> Result<()> {
        let config = self.thread_config.lock().await.clone();
        self.message_tx.send(ThreadMessage {
            thread_id: self.thread_id,
            lines: vec![line],
            config,
        }).await.map_err(|e| anyhow!("Failed to send message: {}", e))?;
        Ok(())
    }

    /// Set the title for a task when using WindowWithTitle mode.
    ///
    /// # Parameters
    /// * `title` - The new title to set
    ///
    /// # Returns
    /// Ok(()) if the title was set successfully, or an error if the task is not in WindowWithTitle mode
    ///
    /// # Errors
    /// Returns a ProgressError::TaskOperation error if the task is not in WindowWithTitle mode
    pub async fn set_title(&self, title: String) -> Result<()> {
        self.progress.set_title(self.thread_id, title).await
    }
    
    /// Add an emoji to the task's display when using WindowWithTitle mode.
    ///
    /// The emoji will be shown at the beginning of the title line.
    /// Multiple emojis can be added and they will be displayed in the order they were added.
    ///
    /// # Parameters
    /// * `emoji` - The emoji character or string to add
    ///
    /// # Returns
    /// Ok(()) if the emoji was added successfully, or an error if the task doesn't support emojis
    ///
    /// # Errors
    /// Returns a ProgressError::TaskOperation error if the task doesn't support emojis
    pub async fn add_emoji(&self, emoji: &str) -> Result<()> {
        self.progress.add_emoji(self.thread_id, emoji).await
    }
    
    /// Set the total number of jobs for this task.
    ///
    /// # Parameters
    /// * `total` - The new total number of jobs
    ///
    /// # Returns
    /// Ok(()) if the total jobs was set successfully, or an error if the task doesn't exist
    ///
    /// # Errors
    /// Returns a ProgressError::TaskOperation error if the task doesn't exist or if total is zero
    pub async fn set_total_jobs(&self, total: usize) -> Result<()> {
        self.progress.set_total_jobs(Some(self.thread_id), total).await
    }
}
