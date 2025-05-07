use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use std::sync::atomic::AtomicUsize;

use crate::errors::{ErrorContext, ProgressError};
use crate::thread::{ThreadManager, TaskHandle};
use crate::modes::Config;
use crate::modes::ThreadMode;
use crate::modes::factory::ModeFactory;
use crate::ThreadMessage;
use tokio::task::JoinHandle;
use tokio::sync::mpsc;

/// Manages progress tracking and state across multiple threads/tasks
pub struct ProgressManager {
    /// Map of thread IDs to their output lines
    outputs: Arc<Mutex<HashMap<usize, Vec<String>>>>,
    /// Thread manager for handling thread lifecycle
    thread_manager: Arc<ThreadManager>,
    /// Factory for creating thread config modes
    factory: Arc<ModeFactory>,
    /// Spinner state for animated indicators
    spinner_index: Arc<AtomicUsize>,
    /// Sender for ThreadMessage channel
    message_tx: mpsc::Sender<ThreadMessage>,
}

impl ProgressManager {
    /// Create a new progress manager with the given factory and message sender
    pub fn new(factory: Arc<ModeFactory>, message_tx: mpsc::Sender<ThreadMessage>) -> Self {
        Self {
            outputs: Arc::new(Mutex::new(HashMap::new())),
            thread_manager: Arc::new(ThreadManager::new()),
            factory,
            spinner_index: Arc::new(AtomicUsize::new(0)),
            message_tx,
        }
    }
    
    /// Create a new task with the specified mode
    pub async fn create_task(&self, mode: ThreadMode, total_jobs: usize) -> Result<TaskHandle> {       
        let thread_id = self.thread_manager.next_thread_id();
        let config = Config::from(self.factory.create_mode(mode, total_jobs)?);
        let task_handle = TaskHandle::new(thread_id, config, self.message_tx.clone());
        let join_handle = tokio::spawn(async move {
            Ok(())
        });
        self.thread_manager.register_thread(thread_id, task_handle.clone(), join_handle).await;
        Ok(task_handle)
    }
    
    /// Create a new task with the specified mode and title
    pub async fn create_task_with_title(&self, mode: ThreadMode, title: String) -> Result<TaskHandle> {
        let mut handle = self.create_task(mode, 1).await?;
        handle.capture_stdout(title).await?;
        Ok(handle)
    }
    
    /// Get a task by its thread ID
    pub async fn get_task(&self, thread_id: usize) -> Option<TaskHandle> {
        self.thread_manager.get_task(thread_id).await
    }
    
    /// Get the number of active threads
    pub async fn thread_count(&self) -> usize {
        self.thread_manager.thread_count().await
    }
    
    /// Join all threads (wait for completion)
    pub async fn join_all(&self) -> Result<()> {
        self.thread_manager.join_all().await
    }
    
    /// Cancel all threads (abort execution)
    pub async fn cancel_all(&self) -> Result<()> {
        self.thread_manager.cancel_all().await
    }
    
    /// Set the title for a specific thread
    pub async fn set_title(&self, thread_id: usize, title: String) -> Result<()> {
        if let Some(handle) = self.thread_manager.get_task(thread_id).await {
            handle.set_title(title).await?;
            Ok(())
        } else {
            let ctx = ErrorContext::new("setting title", "ProgressManager")
                .with_thread_id(thread_id)
                .with_details("Thread not found");
            
            let error_msg = format!("Thread {} not found", thread_id);
            let error = ProgressError::TaskOperation(error_msg).into_context(ctx);
            Err(anyhow::anyhow!(error))
        }
    }
    
    /// Add an emoji to a specific thread
    pub async fn add_emoji(&self, thread_id: usize, emoji: &str) -> Result<()> {
        if let Some(handle) = self.thread_manager.get_task(thread_id).await {
            let mut config = handle.config().lock().await;
            config.add_emoji(emoji)?;
            Ok(())
        } else {
            let ctx = ErrorContext::new("adding emoji", "ProgressManager")
                .with_thread_id(thread_id)
                .with_details("Thread not found");
            
            let error_msg = format!("Thread {} not found", thread_id);
            let error = ProgressError::TaskOperation(error_msg).into_context(ctx);
            Err(anyhow::anyhow!(error))
        }
    }
    
    /// Set the total number of jobs for a thread or all threads
    pub async fn set_total_jobs(&self, thread_id: Option<usize>, total: usize) -> Result<()> {
        if total == 0 {
            let ctx = ErrorContext::new("setting total jobs", "ProgressManager")
                .with_details("Total jobs cannot be zero");
            let error = ProgressError::DisplayOperation("Total jobs cannot be zero".to_string())
                .into_context(ctx);
            return Err(anyhow::anyhow!(error));
        }
        
        match thread_id {
            Some(thread_id) => {
                if let Some(handle) = self.thread_manager.get_task(thread_id).await {
                    handle.set_total_jobs(total).await?;
                    Ok(())
                } else {
                    let ctx = ErrorContext::new("setting total jobs", "ProgressManager")
                        .with_thread_id(thread_id)
                        .with_details("Thread not found");
                    
                    let error_msg = format!("Thread {} not found", thread_id);
                    let error = ProgressError::TaskOperation(error_msg).into_context(ctx);
                    Err(anyhow::anyhow!(error))
                }
            },
            None => {
                // Apply to all threads
                let active_threads = self.thread_manager.get_active_threads().await;
                
                // Update each thread
                for thread_id in active_threads {
                    if let Some(handle) = self.thread_manager.get_task(thread_id).await {
                        handle.set_total_jobs(total).await?;
                    }
                }
                
                Ok(())
            }
        }
    }
    
    /// Update the progress bar for a specific thread
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
    
    /// Handle a message from a thread
    pub async fn handle_message(&self, msg: ThreadMessage) {
        let mut outputs = self.outputs.lock().await;
        let thread_outputs = outputs.entry(msg.thread_id).or_insert_with(Vec::new);
        
        // Add new messages
        thread_outputs.extend(msg.lines);
    }
    
    /// Get a reference to the outputs
    pub fn outputs(&self) -> &Arc<Mutex<HashMap<usize, Vec<String>>>> {
        &self.outputs
    }
    
    /// Get the thread manager
    pub fn thread_manager(&self) -> &Arc<ThreadManager> {
        &self.thread_manager
    }
    
    /// Create a new spawn task that runs the given function
    pub async fn spawn<F, R>(&self, f: F) -> Result<TaskHandle>
    where
        F: FnOnce(TaskHandle) -> R + Send + 'static,
        R: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        let handle = self.create_task(ThreadMode::Limited, 1).await?;
        
        // Spawn the task
        let task_handle: JoinHandle<Result<()>> = tokio::spawn(f(handle.clone()));
        
        // Store the handle
        self.thread_manager.register_thread(handle.thread_id(), handle.clone(), task_handle).await;
        
        Ok(handle)
    }
} 