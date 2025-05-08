#![deny(unused_imports)]
#![feature(internal_output_capture)]

use std::{
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
use crate::modes::Config;
use crate::modes::factory::ModeFactory;
use crate::thread::TaskHandle;
use crate::renderer::Renderer;
use crate::progress_manager::ProgressManager;
pub mod io;

pub mod modes;
pub mod errors;
pub mod formatter;
pub mod terminal;
pub mod thread;
pub mod renderer;
pub mod progress_manager;
pub mod progress_bar;
#[cfg(test)]
pub mod tests;

pub use modes::{ModeRegistry, ModeCreator, ThreadMode};
pub use errors::ModeCreationError;
pub use formatter::{ProgressTemplate, TemplateContext, TemplateVar, TemplatePreset, ColorName, ProgressIndicator};
pub use io::{ProgressWriter, OutputBuffer, TeeWriter};
pub use io::custom::{CustomWriter, WriterCapabilities, WriterRegistry};

thread_local! {
    static CURRENT_THREAD_ID: AtomicUsize = const { AtomicUsize::new(0) };
    static CURRENT_WRITER: RefCell<Option<ThreadLogger>> = const { RefCell::new(None) };
}

/// Message sent from a thread to the progress display
#[derive(Debug, Clone)]
pub struct ThreadMessage {
    /// The ID of the thread that sent the message
    pub thread_id: usize,
    /// The lines of output from the thread
    pub lines: Vec<String>,
    /// The configuration for the thread
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
    /// Renderer responsible for UI display
    renderer: Arc<Renderer>,
    /// Progress manager for business logic
    progress_manager: Arc<ProgressManager>,
    /// Message receiving channel for thread messages
    message_rx: Arc<Mutex<mpsc::Receiver<ThreadMessage>>>,
    /// Flag to control if the display is running
    running: Arc<AtomicBool>,
    /// Background processing task
    processing_task: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl std::fmt::Debug for ProgressDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProgressDisplay")
            .field("renderer", &"Arc<Renderer>")
            .field("progress_manager", &"Arc<ProgressManager>")
            .field("message_rx", &"Arc<Mutex<mpsc::Receiver<ThreadMessage>>>")
            .field("running", &self.running)
            .field("processing_task", &self.processing_task)
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
        let (message_tx, message_rx) = mpsc::channel(1000);
        let renderer = Arc::new(Renderer::new());
        let progress_manager = Arc::new(ProgressManager::new(factory.clone(), message_tx));
        
        let display = Self {
            renderer,
            progress_manager,
            message_rx: Arc::new(Mutex::new(message_rx)),
            running: Arc::new(AtomicBool::new(true)),
            processing_task: Arc::new(Mutex::new(None)),
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
        
        self.progress_manager.create_task(mode, total_jobs).await
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

        self.progress_manager.spawn(f).await
    }

    /// Create a new task with the specified mode and title
    pub async fn spawn_with_mode<F, R>(&self, mode: ThreadMode, f: F) -> Result<TaskHandle>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Into<String> + Send + 'static,
    {
        if !self.running.load(Ordering::SeqCst) {
            let ctx = ErrorContext::new("spawning task with mode", "ProgressDisplay")
                .with_details("Display is not running");
            return Err(anyhow::Error::from(ProgressError::DisplayOperation("Display is not running".to_string()).into_context(ctx)));
        }
        
        let title = f().into();
        self.progress_manager.create_task_with_title(mode, title).await
    }

    pub async fn display(&self) -> std::io::Result<()> {
        let outputs = self.progress_manager.outputs().lock().await;
        self.renderer.render(&outputs).await
    }

    /// Stop the display and clean up all resources
    pub async fn stop(&self) -> Result<()> {
        // First, signal that we're shutting down
        self.running.store(false, Ordering::SeqCst);
        
        // Cancel all tasks first
        self.progress_manager.cancel_all().await?;
        
        // Join all tasks to ensure they're properly cleaned up
        self.progress_manager.join_all().await?;
        
        // Stop the terminal event detection
        if let Err(e) = self.renderer.stop().await {
            let ctx = ErrorContext::new("stopping terminal event detection", "ProgressDisplay")
                .with_details(format!("Failed to stop terminal event detection: {}", e));
            return Err(anyhow::Error::from(ProgressError::DisplayOperation(e.to_string()).into_context(ctx)));
        }
        
        // Stop the processing task last
        let mut guard = self.processing_task.lock().await;
        if let Some(task) = guard.take() {
            task.abort();
        }
        Ok(())
    }

    /// Set the title for a specific thread (if it supports titles)
    pub async fn set_title(&self, thread_id: usize, title: String) -> Result<()> {
        self.progress_manager.set_title(thread_id, title).await
    }

    /// Add an emoji to the display of a specific thread (if it supports emojis)
    pub async fn add_emoji(&self, thread_id: usize, emoji: &str) -> Result<()> {
        self.progress_manager.add_emoji(thread_id, emoji).await
    }

    /// Set the total number of jobs for a specific thread or all threads
    pub async fn set_total_jobs(&self, thread_id: Option<usize>, total: usize) -> Result<()> {
        self.progress_manager.set_total_jobs(thread_id, total).await
    }

    /// Update the progress for a specific thread
    pub async fn update_progress(&self, thread_id: usize) -> Result<f64> {
        self.progress_manager.update_progress(thread_id).await
    }

    /// Join all tasks to ensure they're properly cleaned up
    pub async fn join_all(&self) -> Result<()> {
        self.progress_manager.join_all().await
    }

    /// Cancel all tasks (abort execution)
    pub async fn cancel_all(&self) -> Result<()> {
        self.progress_manager.cancel_all().await
    }

    /// Get the number of active threads
    pub async fn thread_count(&self) -> usize {
        self.progress_manager.thread_count().await
    }

    /// Get a task handle by its thread ID
    pub async fn get_task(&self, thread_id: usize) -> Option<TaskHandle> {
        self.progress_manager.get_task(thread_id).await
    }

    /// Get access to the progress manager
    pub fn progress_manager(&self) -> &Arc<ProgressManager> {
        &self.progress_manager
    }
    
    /// Background thread that receives messages and processes them
    async fn start_display_thread(&self) {
        let mut rx = self.message_rx.lock().await;
        
        // Process messages in batches for better performance
        let mut batch_size = 0;
        const MAX_BATCH_SIZE: usize = 50;
        
        while self.running.load(Ordering::SeqCst) {
            tokio::select! {
                // Try to receive a message with a small timeout
                msg_option = tokio::time::timeout(
                    tokio::time::Duration::from_millis(10), 
                    rx.recv()
                ) => {
                    match msg_option {
                        Ok(Some(msg)) => {
                            // Process the message
                            self.progress_manager.handle_message(msg).await;
                            batch_size += 1;
                            
                            // If we've processed enough messages, update the display
                            if batch_size >= MAX_BATCH_SIZE {
                                if let Err(e) = self.display().await {
                                    eprintln!("Error displaying progress: {}", e);
                                }
                                batch_size = 0;
                            }
                            
                            // Try to process any pending messages without delay
                            while batch_size < MAX_BATCH_SIZE {
                                match rx.try_recv() {
                                    Ok(msg) => {
                                        self.progress_manager.handle_message(msg).await;
                                        batch_size += 1;
                                    },
                                    Err(_) => break,
                                }
                            }
                        },
                        Ok(None) => {
                            // Channel is closed, exit
                            return;
                        },
                        Err(_) => {
                            // Timeout, update display with current state
                            if batch_size > 0 {
                                if let Err(e) = self.display().await {
                                    eprintln!("Error displaying progress: {}", e);
                                }
                                batch_size = 0;
                            }
                        }
                    }
                }
            }
        }
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

