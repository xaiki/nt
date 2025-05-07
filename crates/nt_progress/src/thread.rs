use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::Mutex;
use std::collections::HashMap;
use tokio::task::JoinHandle;
use anyhow::Result;
use crate::errors::{ErrorContext, ProgressError};
use crate::modes::{Config, ThreadMode};
use tokio::sync::mpsc;
use std::io::Write;

/// Represents the state of a thread in the system
#[derive(Debug, Clone, PartialEq)]
pub enum ThreadState {
    /// Thread is running normally
    Running,
    /// Thread is paused
    Paused,
    /// Thread has completed successfully
    Completed,
    /// Thread has failed with an error
    Failed(String),
}

/// Represents a thread's context and state
#[derive(Debug)]
pub struct ThreadContext {
    /// The current state of the thread
    state: ThreadState,
    /// The task handle for the thread
    handle: TaskHandle,
    /// The join handle for the thread
    join_handle: Option<JoinHandle<Result<()>>>,
    /// The time when the thread was created
    created_at: std::time::Instant,
    /// The time when the thread was last updated
    last_updated: std::time::Instant,
}

impl ThreadContext {
    /// Create a new thread context
    pub fn new(handle: TaskHandle, join_handle: JoinHandle<Result<()>>) -> Self {
        let now = std::time::Instant::now();
        Self {
            state: ThreadState::Running,
            handle,
            join_handle: Some(join_handle),
            created_at: now,
            last_updated: now,
        }
    }

    /// Get the current state of the thread
    pub fn state(&self) -> &ThreadState {
        &self.state
    }

    /// Get the task handle
    pub fn handle(&self) -> &TaskHandle {
        &self.handle
    }

    /// Get the join handle
    pub fn join_handle(&self) -> Option<&JoinHandle<Result<()>>> {
        self.join_handle.as_ref()
    }

    /// Take ownership of the join handle
    pub fn take_join_handle(&mut self) -> Option<JoinHandle<Result<()>>> {
        self.join_handle.take()
    }

    /// Get the time when the thread was created
    pub fn created_at(&self) -> std::time::Instant {
        self.created_at
    }

    /// Get the time when the thread was last updated
    pub fn last_updated(&self) -> std::time::Instant {
        self.last_updated
    }

    /// Update the thread's state
    pub fn update_state(&mut self, state: ThreadState) {
        self.state = state;
        self.last_updated = std::time::Instant::now();
    }
}

/// Manages the lifecycle of threads in the system.
///
/// The `ThreadManager` provides a centralized way to manage thread lifecycles, including:
/// - Thread creation and registration
/// - Thread state tracking
/// - Resource cleanup
/// - Thread pool management
/// - Thread synchronization
///
/// # Thread States
/// Threads can be in one of the following states:
/// - `Running`: Thread is executing normally
/// - `Paused`: Thread execution is temporarily suspended
/// - `Completed`: Thread has finished successfully
/// - `Failed`: Thread encountered an error
///
/// # Thread Pool Management
/// The manager maintains a pool of threads and provides methods to:
/// - Track active threads
/// - Limit concurrent thread count
/// - Clean up completed threads
/// - Handle thread failures
///
/// # Example
/// ```rust
/// # use nt_progress::thread::ThreadManager;
/// # async fn example() -> anyhow::Result<()> {
/// let manager = ThreadManager::new();
/// 
/// // Register a new thread
/// let thread_id = manager.next_thread_id();
/// let task_handle = TaskHandle::new(thread_id, config);
/// let join_handle = tokio::spawn(async move { Ok(()) });
/// manager.register_thread(thread_id, task_handle, join_handle).await;
/// 
/// // Get thread state
/// if let Some(state) = manager.get_thread_state(thread_id).await {
///     println!("Thread {} is in state {:?}", thread_id, state);
/// }
/// 
/// // Clean up when done
/// manager.join_all().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct ThreadManager {
    /// The next available thread ID
    next_id: AtomicUsize,
    /// Map of thread IDs to their contexts
    threads: Arc<Mutex<HashMap<usize, ThreadContext>>>,
    /// Maximum number of concurrent threads
    max_threads: AtomicUsize,
}

impl ThreadManager {
    /// Create a new ThreadManager instance with default settings.
    ///
    /// By default, there is no limit on the number of concurrent threads.
    pub fn new() -> Self {
        Self {
            next_id: AtomicUsize::new(0),
            threads: Arc::new(Mutex::new(HashMap::new())),
            max_threads: AtomicUsize::new(usize::MAX),
        }
    }

    /// Create a new ThreadManager with a specified maximum number of concurrent threads.
    pub fn with_thread_limit(max_threads: usize) -> Self {
        Self {
            next_id: AtomicUsize::new(0),
            threads: Arc::new(Mutex::new(HashMap::new())),
            max_threads: AtomicUsize::new(max_threads),
        }
    }

    /// Set the maximum number of concurrent threads.
    pub fn set_thread_limit(&self, max_threads: usize) {
        self.max_threads.store(max_threads, Ordering::SeqCst);
    }

    /// Get the current thread limit.
    pub fn get_thread_limit(&self) -> usize {
        self.max_threads.load(Ordering::SeqCst)
    }

    /// Generate a new unique thread ID.
    pub fn next_thread_id(&self) -> usize {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Register a new thread with its handle and configuration.
    ///
    /// This method will wait if the thread limit has been reached.
    pub async fn register_thread(&self, thread_id: usize, handle: TaskHandle, join_handle: JoinHandle<Result<()>>) {
        let mut threads = self.threads.lock().await;
        while threads.len() >= self.max_threads.load(Ordering::SeqCst) {
            // Drop the lock and wait for a thread to complete
            drop(threads);
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            threads = self.threads.lock().await;
        }
        threads.insert(thread_id, ThreadContext::new(handle, join_handle));
    }

    /// Clean up completed threads from the pool.
    pub async fn cleanup_completed(&self) -> Result<()> {
        let mut threads = self.threads.lock().await;
        let mut to_remove = Vec::new();
        
        for (thread_id, ctx) in threads.iter() {
            match ctx.state() {
                ThreadState::Completed | ThreadState::Failed(_) => {
                    to_remove.push(*thread_id);
                }
                _ => {}
            }
        }
        
        for thread_id in to_remove {
            threads.remove(&thread_id);
        }
        
        Ok(())
    }

    /// Get the number of active threads.
    pub async fn thread_count(&self) -> usize {
        let threads = self.threads.lock().await;
        threads.len()
    }

    /// Get a task handle for a specific thread ID.
    pub async fn get_task(&self, thread_id: usize) -> Option<TaskHandle> {
        let threads = self.threads.lock().await;
        threads.get(&thread_id).map(|ctx| ctx.handle().clone())
    }

    /// Get the state of a specific thread.
    pub async fn get_thread_state(&self, thread_id: usize) -> Option<ThreadState> {
        let threads = self.threads.lock().await;
        threads.get(&thread_id).map(|ctx| ctx.state().clone())
    }

    /// Update the state of a specific thread.
    pub async fn update_thread_state(&self, thread_id: usize, state: ThreadState) -> Result<()> {
        let mut threads = self.threads.lock().await;
        if let Some(ctx) = threads.get_mut(&thread_id) {
            ctx.update_state(state);
            Ok(())
        } else {
            let ctx = ErrorContext::new("updating thread state", "ThreadManager")
                .with_thread_id(thread_id)
                .with_details("Thread not found");
            Err(ProgressError::TaskOperation(
                format!("Failed to update state for thread {}: Thread not found", thread_id)
            ).into_context(ctx).into())
        }
    }

    /// Join all threads and wait for their completion.
    pub async fn join_all(&self) -> Result<()> {
        let mut threads = self.threads.lock().await;
        let mut thread_contexts = Vec::new();
        
        // Drain the threads map into our vector
        for (thread_id, ctx) in threads.drain() {
            thread_contexts.push((thread_id, ctx));
        }
        
        // Now process each thread
        for (thread_id, mut ctx) in thread_contexts {
            if let Some(join_handle) = ctx.take_join_handle() {
                if let Err(e) = join_handle.await {
                    let ctx = ErrorContext::new("joining thread", "ThreadManager")
                        .with_thread_id(thread_id)
                        .with_details(&e.to_string());
                    return Err(ProgressError::TaskOperation(
                        format!("Failed to join thread {}: {}", thread_id, e)
                    ).into_context(ctx).into());
                }
            }
        }
        Ok(())
    }

    /// Cancel all threads and clean up resources.
    pub async fn cancel_all(&self) -> Result<()> {
        let mut threads = self.threads.lock().await;
        for ctx in threads.values_mut() {
            ctx.update_state(ThreadState::Failed("Cancelled".to_string()));
            if let Some(join_handle) = ctx.take_join_handle() {
                join_handle.abort();
            }
        }
        threads.clear();
        Ok(())
    }

    /// Get all active thread IDs.
    pub async fn get_active_threads(&self) -> Vec<usize> {
        let threads = self.threads.lock().await;
        threads.keys().cloned().collect()
    }

    /// Get all threads that are in a specific state.
    pub async fn get_threads_by_state(&self, state: ThreadState) -> Vec<usize> {
        let threads = self.threads.lock().await;
        threads.iter()
            .filter(|(_, ctx)| ctx.state() == &state)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get the number of threads in a specific state.
    pub async fn count_threads_by_state(&self, state: ThreadState) -> usize {
        let threads = self.threads.lock().await;
        threads.iter()
            .filter(|(_, ctx)| ctx.state() == &state)
            .count()
    }
}

/// A handle to a task that can be used to interact with it.
#[derive(Clone)]
pub struct TaskHandle {
    thread_id: usize,
    #[cfg(test)]
    pub thread_config: Arc<Mutex<Config>>,
    #[cfg(not(test))]
    thread_config: Arc<Mutex<Config>>,
    message_tx: mpsc::Sender<crate::ThreadMessage>,
    writer: Arc<Mutex<Box<dyn Write + Send + 'static>>>,
    join_handle: Arc<Mutex<Option<JoinHandle<Result<()>>>>>,
}

impl std::fmt::Debug for TaskHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskHandle")
            .field("thread_id", &self.thread_id)
            .field("thread_config", &"Arc<Mutex<Config>>")
            .field("message_tx", &"mpsc::Sender<ThreadMessage>")
            .field("writer", &"Arc<Mutex<Box<dyn Write + Send>>>")
            .field("join_handle", &"Arc<Mutex<Option<JoinHandle<Result<()>>>>>")
            .finish()
    }
}

impl TaskHandle {
    /// Create a new TaskHandle with the specified thread ID and configuration.
    pub fn new(thread_id: usize, config: Config) -> Self {
        Self {
            thread_id,
            thread_config: Arc::new(Mutex::new(config)),
            message_tx: mpsc::channel(100).0,
            writer: Arc::new(Mutex::new(Box::new(std::io::stdout()))),
            join_handle: Arc::new(Mutex::new(None)),
        }
    }

    /// Get the thread ID associated with this handle.
    pub fn thread_id(&self) -> usize {
        self.thread_id
    }

    /// Get a reference to the thread configuration.
    pub fn config(&self) -> &Arc<Mutex<Config>> {
        &self.thread_config
    }

    /// Set the mode for this task.
    pub async fn set_mode(&mut self, mode: ThreadMode) -> Result<()> {
        // Create a new config with the specified mode
        let config = Config::new(mode, 1)?;
        
        // Now that we have successfully created the config, update our thread_config
        *self.thread_config.lock().await = config;
        Ok(())
    }

    /// Capture stdout output for this task.
    pub async fn capture_stdout(&mut self, line: String) -> Result<()> {
        let config = self.thread_config.lock().await.clone();
        self.message_tx.send(crate::ThreadMessage {
            thread_id: self.thread_id,
            lines: vec![line],
            config,
        }).await.map_err(|e| anyhow::anyhow!("Failed to send message: {}", e))?;
        Ok(())
    }

    /// Capture stderr output for this task.
    pub async fn capture_stderr(&mut self, line: String) -> Result<()> {
        let config = self.thread_config.lock().await.clone();
        self.message_tx.send(crate::ThreadMessage {
            thread_id: self.thread_id,
            lines: vec![line],
            config,
        }).await.map_err(|e| anyhow::anyhow!("Failed to send message: {}", e))?;
        Ok(())
    }

    /// Set the title for this task.
    pub async fn set_title(&self, title: String) -> Result<()> {
        let mut config = self.thread_config.lock().await;
        if !config.supports_title() {
            let ctx = ErrorContext::new("setting title", "TaskHandle")
                .with_thread_id(self.thread_id)
                .with_details("Current mode does not support titles");
            
            let error = ProgressError::TaskOperation(
                "Task is not in a mode that supports titles".to_string()
            ).into_context(ctx);
            return Err(anyhow::anyhow!(error));
        }
        
        // Set the title using the underlying config
        if let Err(e) = config.set_title(title) {
            let ctx = ErrorContext::new("setting title", "TaskHandle")
                .with_thread_id(self.thread_id)
                .with_details(&e.to_string());
            
            let error = ProgressError::TaskOperation(
                format!("Failed to set title: {}", e)
            ).into_context(ctx);
            return Err(anyhow::anyhow!(error));
        }
        Ok(())
    }
    
    /// Add an emoji to this task.
    pub async fn add_emoji(&self, emoji: &str) -> Result<()> {
        let mut config = self.thread_config.lock().await;
        
        if !config.supports_emoji() {
            let ctx = ErrorContext::new("adding emoji", "TaskHandle")
                .with_thread_id(self.thread_id)
                .with_details("Current mode does not support emojis");
            
            let error = ProgressError::TaskOperation(
                "Task is not in a mode that supports emojis".to_string()
            ).into_context(ctx);
            return Err(anyhow::anyhow!(error));
        }
        
        // Add the emoji using the underlying config
        if let Err(e) = config.add_emoji(emoji) {
            let ctx = ErrorContext::new("adding emoji", "TaskHandle")
                .with_thread_id(self.thread_id)
                .with_details(&e.to_string());
            
            let error = ProgressError::TaskOperation(
                format!("Failed to add emoji: {}", e)
            ).into_context(ctx);
            return Err(anyhow::anyhow!(error));
        }
        Ok(())
    }
    
    /// Set the total number of jobs for this task.
    pub async fn set_total_jobs(&self, total: usize) -> Result<()> {
        let mut config = self.thread_config.lock().await;
        config.set_total_jobs(total);
        Ok(())
    }

    /// Join this task and wait for its completion.
    pub async fn join(self) -> Result<()> {
        if let Some(handle) = self.join_handle.lock().await.take() {
            handle.await??;
        }
        Ok(())
    }

    /// Cancel this task.
    pub async fn cancel(self) -> Result<()> {
        if let Some(handle) = self.join_handle.lock().await.take() {
            handle.abort();
        }
        Ok(())
    }

    /// Abort this task without waiting.
    pub fn abort(&self) {
        if let Ok(mut handle) = self.join_handle.try_lock() {
            if let Some(task_handle) = handle.take() {
                task_handle.abort();
            }
        }
    }
} 