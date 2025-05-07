#![deny(unused_imports)]
#![feature(internal_output_capture)]

use std::{
    collections::HashMap,
    io::Write,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    future::Future,
};
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
};
use anyhow::{Result, anyhow};
use std::fmt::Debug;
use std::cell::RefCell;
use crate::errors::{ErrorContext, ProgressError};
use crate::terminal::Terminal;
use crate::modes::Config;
use crate::modes::factory::ModeFactory;
use crate::thread::{ThreadManager, TaskHandle};
pub mod io;

pub mod modes;
pub mod errors;
pub mod formatter;
pub mod terminal;
pub mod thread;
#[cfg(test)]
pub mod tests;

pub use modes::{ModeRegistry, ModeCreator, ThreadMode};
pub use errors::ModeCreationError;
pub use formatter::{ProgressTemplate, TemplateContext, TemplateVar, TemplatePreset};
pub use io::{ProgressWriter, OutputBuffer, TeeWriter};
pub use io::custom::{CustomWriter, WriterCapabilities, WriterRegistry};

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
    terminal: Arc<Terminal>,
    spinner_index: Arc<AtomicUsize>,
    message_tx: mpsc::Sender<ThreadMessage>,
    message_rx: Arc<Mutex<mpsc::Receiver<ThreadMessage>>>,
    running: Arc<AtomicBool>,
    processing_task: Arc<Mutex<Option<JoinHandle<()>>>>,
    thread_manager: Arc<ThreadManager>,
    writer: Arc<Mutex<Box<dyn Write + Send + 'static>>>,
    factory: Arc<ModeFactory>,
}

impl std::fmt::Debug for ProgressDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProgressDisplay")
            .field("outputs", &"Arc<Mutex<HashMap<usize, Vec<String>>>>")
            .field("terminal", &"Arc<Terminal>")
            .field("spinner_index", &self.spinner_index)
            .field("message_tx", &"mpsc::Sender<ThreadMessage>")
            .field("message_rx", &"Arc<Mutex<mpsc::Receiver<ThreadMessage>>>")
            .field("running", &self.running)
            .field("processing_task", &self.processing_task)
            .field("thread_manager", &self.thread_manager)
            .field("writer", &"Arc<Mutex<Box<dyn Write + Send>>>")
            .field("factory", &self.factory)
            .finish()
    }
}

impl ProgressDisplay {
    /// Create a new ProgressDisplay with default settings
    pub async fn new() -> Result<Self> {
        Self::new_with_factory(Arc::new(ModeFactory::new())).await
    }

    /// Create a new ProgressDisplay with a specific mode
    pub async fn new_with_mode(mode: ThreadMode) -> Result<Self> {
        let mut factory = ModeFactory::new();
        factory.set_default_mode(mode);
        Self::new_with_factory(Arc::new(factory)).await
    }

    /// Create a new ProgressDisplay with a specific factory
    pub async fn new_with_factory(factory: Arc<ModeFactory>) -> Result<Self> {
        let (message_tx, message_rx) = mpsc::channel(100);
        let terminal: Arc<Terminal> = Arc::new(Terminal::new());
        let display = Self {
            outputs: Arc::new(Mutex::new(HashMap::new())),
            terminal: terminal.clone(),
            spinner_index: Arc::new(AtomicUsize::new(0)),
            message_tx: message_tx.clone(),
            message_rx: Arc::new(Mutex::new(message_rx)),
            running: Arc::new(AtomicBool::new(true)),
            processing_task: Arc::new(Mutex::new(None)),
            thread_manager: Arc::new(ThreadManager::new()),
            writer: Arc::new(Mutex::new(Box::new(std::io::stdout()))),
            factory: factory.clone(),
        };

        // Create a weak reference for the processing task
        let display_arc = Arc::new(display);
        let display_weak = Arc::downgrade(&display_arc);

        let processing_task = tokio::spawn({
            let display_weak = display_weak.clone();
            async move {
                if let Some(display) = display_weak.upgrade() {
                    display.start_display_thread().await;
                }
            }
        });

        let mut guard = display_arc.processing_task.lock().await;
        *guard = Some(processing_task);
        drop(guard);

        Ok(Arc::try_unwrap(display_arc).expect("Failed to unwrap Arc - this should never happen"))
    }

    /// Create a new task with the specified mode
    pub async fn create_task(&self, mode: ThreadMode, total_jobs: usize) -> Result<TaskHandle> {
        // Check if the display is running
        if !self.running.load(Ordering::SeqCst) {
            let ctx = ErrorContext::new("creating task", "ProgressDisplay")
                .with_details("Display is not running");
            return Err(anyhow::Error::from(ProgressError::DisplayOperation("Display is not running".to_string()).into_context(ctx)));
        }
        
        let thread_id = self.thread_manager.next_thread_id();
        let config = Config::from(self.factory.create_mode(mode, total_jobs)?);
        let task_handle = TaskHandle::new(thread_id, config);
        let join_handle = tokio::spawn(async move {
            Ok(())
        });
        self.thread_manager.register_thread(thread_id, task_handle.clone(), join_handle).await;
        Ok(task_handle)
    }

    pub async fn spawn<F, R>(&self, f: F) -> Result<TaskHandle>
    where
        F: FnOnce(TaskHandle) -> R + Send + 'static,
        R: Future<Output = Result<()>> + Send + 'static,
    {
        if !self.running.load(Ordering::SeqCst) {
            let ctx = ErrorContext::new("spawning task", "ProgressDisplay")
                .with_details("Display is not running");
            return Err(anyhow::Error::from(ProgressError::DisplayOperation("Display is not running".to_string()).into_context(ctx)));
        }

        let handle = self.create_task(ThreadMode::Limited, 1).await?;
        
        // Spawn the task
        let task_handle: JoinHandle<Result<()>> = tokio::spawn(f(handle.clone()));
        
        // Store the handle
        self.thread_manager.register_thread(handle.thread_id(), handle.clone(), task_handle).await;
        
        Ok(handle)
    }

    /// Create a new task with the specified mode and title
    pub async fn spawn_with_mode<F, R>(&self, mode: ThreadMode, f: F) -> Result<TaskHandle>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Into<String> + Send + 'static,
    {
        // Create the task directly with the specified mode instead of Limited mode
        let mut handle = self.create_task(mode, 1).await?;
        
        let title = f().into();
        handle.capture_stdout(title).await?;
        
        Ok(handle)
    }

    pub async fn display(&self) -> std::io::Result<()> {
        self.render_display().await
    }

    /// Stop the display and clean up all resources
    pub async fn stop(&self) -> Result<()> {
        // First, signal that we're shutting down
        self.running.store(false, Ordering::SeqCst);
        
        // Cancel all tasks first
        self.cancel_all().await?;
        
        // Join all tasks to ensure they're properly cleaned up
        self.join_all().await?;
        
        // Stop the terminal event detection
        if let Err(e) = self.terminal.stop_event_detection().await {
            let ctx = ErrorContext::new("stopping terminal event detection", "ProgressDisplay")
                .with_details(format!("Failed to stop terminal event detection: {}", e));
            return Err(anyhow::Error::from(ProgressError::DisplayOperation(e.to_string()).into_context(ctx)));
        }
        
        // Stop the processing task last
        if let Some(handle) = self.processing_task.lock().await.take() {
            // Give the task a chance to finish gracefully
            tokio::select! {
                result = handle => {
                    if let Err(e) = result {
                        let ctx = ErrorContext::new("stopping processing task", "ProgressDisplay")
                            .with_details(format!("Processing task failed during shutdown: {}", e));
                        return Err(anyhow::Error::from(ProgressError::DisplayOperation(e.to_string()).into_context(ctx)));
                    }
                }
                _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
                    // If we timeout, we can't abort the handle since it's moved into the select
                    // But the process will be cleaned up when the program exits
                    let ctx = ErrorContext::new("stopping processing task", "ProgressDisplay")
                        .with_details("Processing task did not finish gracefully during shutdown");
                    return Err(anyhow::Error::from(ProgressError::DisplayOperation("Processing task timeout".to_string()).into_context(ctx)));
                }
            }
        }
        
        Ok(())
    }

    pub async fn set_title(&self, thread_id: usize, title: String) -> Result<()> {
        if let Some(handle) = self.thread_manager.get_task(thread_id).await {
            let mut config = handle.config().lock().await;
            config.set_title(title)?;
            Ok(())
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
        if let Some(handle) = self.thread_manager.get_task(thread_id).await {
            let mut config = handle.config().lock().await;
            config.add_emoji(emoji)?;
            Ok(())
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
        
        if let Some(thread_id) = thread_id {
            // Update a specific thread's total jobs
            if let Some(handle) = self.thread_manager.get_task(thread_id).await {
                let mut config = handle.config().lock().await;
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
            let active_threads = self.thread_manager.get_active_threads().await;
            for thread_id in active_threads {
                if let Some(handle) = self.thread_manager.get_task(thread_id).await {
                    let mut config = handle.config().lock().await;
                    config.set_total_jobs(total);
                }
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
        self.thread_manager.join_all().await
    }

    pub async fn cancel_all(&self) -> Result<()> {
        self.thread_manager.cancel_all().await
    }

    pub async fn thread_count(&self) -> usize {
        self.thread_manager.thread_count().await
    }

    pub async fn get_task(&self, thread_id: usize) -> Option<TaskHandle> {
        self.thread_manager.get_task(thread_id).await
    }

    async fn start_display_thread(&self) {
        let mut rx = self.message_rx.lock().await;
        while self.running.load(Ordering::SeqCst) {
            if let Some(msg) = rx.recv().await {
                self.handle_message(msg).await;
            }
        }
    }

    async fn handle_message(&self, msg: ThreadMessage) {
        let mut outputs = self.outputs.lock().await;
        let thread_outputs = outputs.entry(msg.thread_id).or_insert_with(Vec::new);
        
        // Add new messages
        thread_outputs.extend(msg.lines);
    }

    async fn render_display(&self) -> std::io::Result<()> {
        let outputs = self.outputs.lock().await;
        if outputs.is_empty() {
            return Ok(());
        }

        let mut writer = self.writer.lock().await;
        write!(writer, "\x1B[2J\x1B[1H")?;

        // First, collect all messages for each thread
        let mut thread_messages = Vec::new();
        for (thread_id, lines) in outputs.iter() {
            for line in lines {
                thread_messages.push((*thread_id, line.clone()));
            }
        }

        // Sort messages by thread ID and content to ensure consistent ordering
        thread_messages.sort_by(|a, b| {
            a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1))
        });

        // Group messages by thread ID and ensure proper spacing
        let mut current_thread = None;
        for (thread_id, message) in thread_messages {
            if current_thread != Some(thread_id) {
                if current_thread.is_some() {
                    writeln!(writer)?;
                }
                current_thread = Some(thread_id);
            }
            writeln!(writer, "{}", message)?;
        }

        writer.flush()?;
        Ok(())
    }
}

impl Drop for ProgressDisplay {
    fn drop(&mut self) {
        // Signal that we're shutting down
        self.running.store(false, Ordering::SeqCst);
        
        // Log a warning about cleanup
        eprintln!("Warning: ProgressDisplay dropped - cleanup should be handled by calling stop() explicitly");
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

