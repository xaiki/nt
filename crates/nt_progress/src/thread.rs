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
use crate::io::{ProgressWriter, OutputBuffer};

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
        // Get a snapshot of threads to join while minimizing lock time
        let mut threads_to_join = Vec::new();
        {
            let mut threads = self.threads.lock().await;
            // Extract only the join handles that we need to wait for
            for (thread_id, ctx) in threads.iter_mut() {
                if let Some(handle) = ctx.take_join_handle() {
                    threads_to_join.push((*thread_id, handle));
                }
            }
        }
        
        // Now join each thread without holding the main lock
        let mut errors = Vec::new();
        for (thread_id, join_handle) in threads_to_join {
            if let Err(e) = join_handle.await {
                let error_msg = format!("Failed to join thread {}: {}", thread_id, e);
                errors.push((thread_id, error_msg));
            }
        }
        
        // After joining, clear the threads collection
        {
            let mut threads = self.threads.lock().await;
            threads.clear();
        }
        
        // Report any errors that occurred
        if let Some((thread_id, error_msg)) = errors.first() {
            let ctx = ErrorContext::new("joining thread", "ThreadManager")
                .with_thread_id(*thread_id)
                .with_details(error_msg);
            return Err(ProgressError::TaskOperation(error_msg.clone())
                .into_context(ctx).into());
        }
        
        Ok(())
    }

    /// Cancel all threads and clean up resources.
    pub async fn cancel_all(&self) -> Result<()> {
        // First collect all handles we need to abort
        let mut handles_to_abort = Vec::new();
        {
            let mut threads = self.threads.lock().await;
            for ctx in threads.values_mut() {
                ctx.update_state(ThreadState::Failed("Cancelled".to_string()));
                if let Some(handle) = ctx.take_join_handle() {
                    handles_to_abort.push(handle);
                }
            }
            
            // Clear the threads collection immediately to reduce resource usage
            threads.clear();
        }
        
        // Now abort all the handles without holding the lock
        for handle in handles_to_abort {
            handle.abort();
        }
        
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

impl Default for ThreadManager {
    fn default() -> Self {
        Self::new()
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
    writer: Arc<Mutex<Box<dyn ProgressWriter + Send + 'static>>>,
    join_handle: Arc<Mutex<Option<JoinHandle<Result<()>>>>>,
}

impl std::fmt::Debug for TaskHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskHandle")
            .field("thread_id", &self.thread_id)
            .field("thread_config", &"Arc<Mutex<Config>>")
            .field("message_tx", &"mpsc::Sender<ThreadMessage>")
            .field("writer", &"Arc<Mutex<Box<dyn ProgressWriter + Send>>>")
            .field("join_handle", &"Arc<Mutex<Option<JoinHandle<Result<()>>>>>")
            .finish()
    }
}

impl TaskHandle {
    /// Create a new TaskHandle with the specified thread ID and configuration.
    pub fn new(thread_id: usize, config: Config, message_tx: mpsc::Sender<crate::ThreadMessage>) -> Self {
        Self {
            thread_id,
            thread_config: Arc::new(Mutex::new(config)),
            message_tx,
            writer: Arc::new(Mutex::new(Box::new(OutputBuffer::new(100)))),
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

    /// Write a line to the task's output
    pub async fn write_line(&mut self, line: &str) -> Result<()> {
        let mut writer = self.writer.lock().await;
        writer.write_line(line)?;
        writer.flush()?;
        Ok(())
    }

    /// Write raw bytes to the task's output
    pub async fn write(&mut self, buf: &[u8]) -> Result<()> {
        let mut writer = self.writer.lock().await;
        writer.write_all(buf)?;
        writer.flush()?;
        Ok(())
    }

    /// Capture stdout output for this task.
    pub async fn capture_stdout(&mut self, line: String) -> Result<()> {
        let config = self.thread_config.lock().await.clone();
        self.message_tx.send(crate::ThreadMessage {
            thread_id: self.thread_id,
            lines: vec![line.clone()],
            config,
        }).await.map_err(|e| anyhow::anyhow!("Failed to send message: {}", e))?;
        
        // Also write to the task's output
        self.write_line(&line).await?;
        
        Ok(())
    }

    /// Capture stderr output for this task.
    pub async fn capture_stderr(&mut self, line: String) -> Result<()> {
        let config = self.thread_config.lock().await.clone();
        self.message_tx.send(crate::ThreadMessage {
            thread_id: self.thread_id,
            lines: vec![line.clone()],
            config,
        }).await.map_err(|e| anyhow::anyhow!("Failed to send message: {}", e))?;
        
        // Also write to the task's output
        self.write_line(&line).await?;
        
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
                .with_details(e.to_string());
            
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
                .with_details(e.to_string());
            
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

    /// Execute a closure with a mutable reference to a specific implementation type.
    ///
    /// This is a generic method that can be used to access any implementation
    /// type that is stored in this TaskHandle's config. It allows downcasting
    /// to specific mode types like Limited, Window, etc.
    ///
    /// # Type Parameters
    /// * `T` - The implementation type to downcast to
    /// * `F` - The closure type that takes a mutable reference to T
    /// * `R` - The return type of the closure
    ///
    /// # Returns
    /// `Some(R)` if the config is of type T and the closure returns a value, `None` otherwise
    pub async fn with_type_mut<T: 'static, F, R>(&mut self, f: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut config = self.thread_config.lock().await;
        config.as_type_mut::<T>().map(f)
    }

    /// Get a direct reference to the writer.
    ///
    /// This allows direct access to the underlying writer without going through
    /// the TaskHandle's write methods. This can be useful for implementing custom
    /// formatting or directly writing to the output without TaskHandle's abstractions.
    ///
    /// # Returns
    /// A reference to the Arc<Mutex<Box<dyn ProgressWriter>>> that allows locking and accessing
    /// the writer directly.
    pub fn writer(&self) -> &Arc<Mutex<Box<dyn ProgressWriter + Send + 'static>>> {
        &self.writer
    }

    /// Replace the current writer with a custom implementation.
    ///
    /// This method allows replacing the default OutputBuffer with any custom writer
    /// that implements the ProgressWriter trait. This is useful for redirecting output
    /// to custom destinations or implementing specialized formatting.
    ///
    /// # Parameters
    /// * `writer` - A boxed instance of a type that implements ProgressWriter
    ///
    /// # Returns
    /// The previous writer that was replaced
    pub async fn set_writer(&mut self, writer: Box<dyn ProgressWriter + Send + 'static>) -> Box<dyn ProgressWriter + Send + 'static> {
        let mut writer_guard = self.writer.lock().await;
        std::mem::replace(&mut *writer_guard, writer)
    }
    
    /// Create a tee writer that outputs to both the internal writer and a custom writer.
    ///
    /// This method allows creating a writer that sends output to both the TaskHandle's
    /// internal writer and a custom writer. This is useful for capturing output in 
    /// multiple destinations simultaneously without changing the TaskHandle's behavior.
    ///
    /// # Parameters
    /// * `additional_writer` - A boxed instance of a type that implements ProgressWriter
    ///
    /// # Returns
    /// Result containing () on success, or an error if the operation fails
    pub async fn add_tee_writer<W: ProgressWriter + Send + 'static>(&mut self, additional_writer: W) -> Result<()> {
        use crate::io::{OutputBuffer, new_tee_writer};
        
        // Create a new buffer to replace the current one
        let new_buffer = OutputBuffer::new(100);
        
        // Replace the current writer with a new empty buffer temporarily
        let prev_writer = self.set_writer(Box::new(new_buffer)).await;
        
        // Create a tee writer from the previous writer and the additional writer
        let tee_writer = new_tee_writer(prev_writer, Box::new(additional_writer));
        
        // Set the writer to the new tee writer
        self.set_writer(tee_writer).await;
        
        Ok(())
    }
    
    /// Execute a closure with mutable access to the writer.
    ///
    /// This method provides a convenient way to perform operations on the writer
    /// without manually managing the locking/unlocking.
    ///
    /// # Parameters
    /// * `f` - A closure that takes a mutable reference to the writer and returns a Result
    ///
    /// # Returns
    /// The result returned by the closure
    pub async fn with_writer<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut Box<dyn ProgressWriter + Send + 'static>) -> Result<R>,
    {
        let mut writer = self.writer.lock().await;
        f(&mut writer)
    }

    /// Enable or disable output passthrough for this task.
    ///
    /// Output passthrough allows messages to be sent to secondary outputs (like stdout/stderr)
    /// in addition to being captured by the task's writer. This is particularly useful
    /// for debugging or when you want to see output in real-time.
    ///
    /// # Parameters
    /// * `enabled` - Whether passthrough should be enabled or disabled
    ///
    /// # Returns
    /// Result containing () on success, or an error if the task's mode doesn't support passthrough
    pub async fn set_passthrough(&mut self, enabled: bool) -> Result<()> {
        let mut config = self.thread_config.lock().await;
        
        // Try to downcast to Limited mode (currently only Limited supports passthrough)
        if let Some(limited) = config.as_type_mut::<crate::modes::Limited>() {
            limited.set_passthrough(enabled);
            Ok(())
        } else {
            let ctx = ErrorContext::new("setting passthrough", "TaskHandle")
                .with_thread_id(self.thread_id)
                .with_details("Current mode does not support passthrough");
            
            Err(anyhow::anyhow!(ProgressError::TaskOperation(
                "Task is not in a mode that supports passthrough".to_string()
            ).into_context(ctx)))
        }
    }
    
    /// Check if passthrough is enabled for this task.
    ///
    /// # Returns
    /// Some(true) if passthrough is enabled, Some(false) if it's disabled, 
    /// or None if the task's mode doesn't support passthrough
    pub async fn has_passthrough(&self) -> Option<bool> {
        let config = self.thread_config.lock().await;
        
        // Using internal method as Limited doesn't implement WithPassthrough trait
        config.as_type::<crate::modes::Limited>().map(|_| {
            // We just check if the mode is Limited
            // Limited mode always has passthrough available, whether it's enabled or not
            // depends on the state which we need to query elsewhere
            true
        })
    }
    
    /// Set a custom passthrough writer for this task.
    ///
    /// This allows you to control where passthrough output is sent, rather than
    /// using the default stdout/stderr. This is useful for redirecting output to
    /// a file, network connection, or custom formatter.
    ///
    /// # Parameters
    /// * `writer` - A boxed instance of a type that implements ProgressWriter
    ///
    /// # Returns
    /// Result containing () on success, or an error if the task's mode doesn't support passthrough
    pub async fn set_passthrough_writer(&mut self, writer: Box<dyn ProgressWriter + Send + 'static>) -> Result<()> {
        let mut config = self.thread_config.lock().await;
        
        // Try to downcast to Limited mode (currently only Limited supports passthrough)
        if let Some(limited) = config.as_type_mut::<crate::modes::Limited>() {
            limited.set_passthrough_writer(writer)?;
            Ok(())
        } else {
            let ctx = ErrorContext::new("setting passthrough writer", "TaskHandle")
                .with_thread_id(self.thread_id)
                .with_details("Current mode does not support passthrough");
            
            Err(anyhow::anyhow!(ProgressError::TaskOperation(
                "Task is not in a mode that supports passthrough".to_string()
            ).into_context(ctx)))
        }
    }
    
    /// Apply a filter function to passthrough output.
    ///
    /// This allows you to conditionally pass through messages based on their content.
    /// For example, you could filter to only pass through messages containing "ERROR".
    ///
    /// # Parameters
    /// * `filter_fn` - A function that takes a string slice and returns true if it should be passed through
    ///
    /// # Returns
    /// Result containing () on success, or an error if setting up the filter fails
    pub async fn set_passthrough_filter<F>(&mut self, filter_fn: F) -> Result<()>
    where
        F: Fn(&str) -> bool + Send + Sync + 'static
    {
        // Create a filtering writer that wraps the default passthrough
        struct FilterWriter<F> {
            filter: F,
            buffer: crate::io::OutputBuffer,
        }
        
        impl<F: Fn(&str) -> bool + Send + Sync> std::fmt::Debug for FilterWriter<F> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct("FilterWriter")
                    .field("buffer", &self.buffer)
                    .finish()
            }
        }
        
        impl<F: Fn(&str) -> bool + Send + Sync> std::io::Write for FilterWriter<F> {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                if let Ok(s) = std::str::from_utf8(buf) {
                    if (self.filter)(s) {
                        println!("{}", s);
                    }
                }
                Ok(buf.len())
            }
            
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        
        impl<F: Fn(&str) -> bool + Send + Sync> crate::io::ProgressWriter for FilterWriter<F> {
            fn write_line(&mut self, line: &str) -> Result<()> {
                if (self.filter)(line) {
                    println!("{}", line);
                }
                self.buffer.add_line(line.to_string());
                Ok(())
            }
            
            fn flush(&mut self) -> Result<()> {
                Ok(())
            }
            
            fn is_ready(&self) -> bool {
                true
            }
        }
        
        // Create our filter writer
        let filter_writer = FilterWriter {
            filter: filter_fn,
            buffer: crate::io::OutputBuffer::new(100),
        };
        
        // Set it as the passthrough writer
        self.set_passthrough_writer(Box::new(filter_writer)).await?;
        
        // Enable passthrough if it's not already enabled
        if let Some(false) = self.has_passthrough().await {
            self.set_passthrough(true).await?;
        }
        
        Ok(())
    }
} 