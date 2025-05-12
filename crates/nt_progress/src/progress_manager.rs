use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;

use crate::errors::{ErrorContext, ProgressError};
use crate::thread::{ThreadManager, TaskHandle, ThreadState};
use crate::config::Config;
use crate::config::ThreadMode;
use crate::modes::factory::ModeFactory;
use crate::ThreadMessage;
use tokio::task::JoinHandle;
use tokio::sync::mpsc;
use crate::ui::progress_bar::{ProgressBar, ProgressBarConfig, ProgressBarStyle, MultiProgressBar};

/// Manages progress tracking and state across multiple threads/tasks
pub struct ProgressManager {
    /// Map of thread IDs to their output lines
    outputs: Arc<Mutex<HashMap<usize, Vec<String>>>>,
    /// Thread manager for handling thread lifecycle
    thread_manager: Arc<ThreadManager>,
    /// Factory for creating thread config modes
    factory: Arc<ModeFactory>,
    /// Sender for ThreadMessage channel
    message_tx: mpsc::Sender<ThreadMessage>,
    /// Collection of multi-progress bars for grouped display
    multi_bars: Arc<Mutex<HashMap<String, MultiProgressBar>>>,
}

impl ProgressManager {
    /// Create a new progress manager with the given factory and message sender
    pub fn new(factory: Arc<ModeFactory>, message_tx: mpsc::Sender<ThreadMessage>) -> Self {
        Self {
            outputs: Arc::new(Mutex::new(HashMap::new())),
            thread_manager: Arc::new(ThreadManager::new()),
            factory,
            message_tx,
            multi_bars: Arc::new(Mutex::new(HashMap::new())),
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
    
    /// Update the progress for a specific thread
    ///
    /// # Parameters
    /// * `thread_id` - The ID of the thread to update progress for
    ///
    /// # Returns
    /// The updated progress percentage between 0.0 and 100.0
    pub async fn update_progress(&self, thread_id: usize) -> Result<f64> {
        if let Some(handle) = self.thread_manager.get_task(thread_id).await {
            handle.update_progress().await
        } else {
            let ctx = ErrorContext::new("updating progress", "ProgressManager")
                .with_thread_id(thread_id)
                .with_details("Thread not found");
            
            let error_msg = format!("Thread {} not found", thread_id);
            let error = ProgressError::TaskOperation(error_msg).into_context(ctx);
            Err(anyhow::anyhow!(error))
        }
    }
    
    /// Set the progress for a specific thread
    ///
    /// # Parameters
    /// * `thread_id` - The ID of the thread to set progress for
    /// * `completed` - The number of completed jobs
    ///
    /// # Returns
    /// The updated progress percentage between 0.0 and 100.0
    pub async fn set_progress(&self, thread_id: usize, completed: usize) -> Result<f64> {
        if let Some(handle) = self.thread_manager.get_task(thread_id).await {
            handle.set_progress(completed).await
        } else {
            let ctx = ErrorContext::new("setting progress", "ProgressManager")
                .with_thread_id(thread_id)
                .with_details("Thread not found");
            
            let error_msg = format!("Thread {} not found", thread_id);
            let error = ProgressError::TaskOperation(error_msg).into_context(ctx);
            Err(anyhow::anyhow!(error))
        }
    }
    
    /// Get the current progress percentage for a specific thread
    ///
    /// # Parameters
    /// * `thread_id` - The ID of the thread to get progress for
    ///
    /// # Returns
    /// The current progress percentage between 0.0 and 100.0
    pub async fn get_progress_percentage(&self, thread_id: usize) -> Result<f64> {
        if let Some(handle) = self.thread_manager.get_task(thread_id).await {
            handle.get_progress_percentage().await
        } else {
            let ctx = ErrorContext::new("getting progress", "ProgressManager")
                .with_thread_id(thread_id)
                .with_details("Thread not found");
            
            let error_msg = format!("Thread {} not found", thread_id);
            let error = ProgressError::TaskOperation(error_msg).into_context(ctx);
            Err(anyhow::anyhow!(error))
        }
    }
    
    /// Set the progress format for a specific thread
    ///
    /// # Parameters
    /// * `thread_id` - The ID of the thread to set the format for
    /// * `format` - The format string for progress display
    pub async fn set_progress_format(&self, thread_id: usize, format: &str) -> Result<()> {
        if let Some(handle) = self.thread_manager.get_task(thread_id).await {
            handle.set_progress_format(format).await
        } else {
            let ctx = ErrorContext::new("setting progress format", "ProgressManager")
                .with_thread_id(thread_id)
                .with_details("Thread not found");
            
            let error_msg = format!("Thread {} not found", thread_id);
            let error = ProgressError::TaskOperation(error_msg).into_context(ctx);
            Err(anyhow::anyhow!(error))
        }
    }
    
    /// Update the progress bar display for a specific thread.
    ///
    /// This method renders a customizable progress bar based on the provided
    /// configuration and current progress values.
    ///
    /// # Parameters
    /// * `thread_id` - The ID of the thread to update the progress bar for
    /// * `current` - The current number of completed items
    /// * `total` - The total number of items
    /// * `config` - The progress bar configuration to use
    ///
    /// # Returns
    /// Result with () if successful
    pub async fn update_progress_bar_with_config(&self, thread_id: usize, current: usize, total: usize, config: &ProgressBarConfig) -> Result<()> {
        if total == 0 {
            return Err(ProgressError::DisplayOperation("Total jobs cannot be zero".to_string()).into());
        }
        
        // Set the total jobs and current progress
        if let Some(mut handle) = self.thread_manager.get_task(thread_id).await {
            handle.set_total_jobs(total).await?;
            handle.set_progress(current).await?;
            
            // Create and update a progress bar
            let mut progress_bar = ProgressBar::new(config.clone());
            progress_bar.update_with_values(current, total);
            
            // Get the template to use for formatting
            let template = progress_bar.template();
            
            // Set the progress display format
            handle.set_progress_format(&template).await?;
            
            // Generate a progress display message
            let mut ctx = crate::ui::formatter::TemplateContext::new();
            ctx.set("progress", progress_bar.progress())
               .set("completed", current)
               .set("total", total)
               .set("percent", format!("{}%", progress_bar.percentage()));
            
            if let Some(prefix) = &config.prefix {
                ctx.set("prefix", prefix.clone());
            }
            
            // Create a template for rendering
            let template = crate::ui::formatter::ProgressTemplate::new(template);
            let message = template.render(&ctx)?;
            
            // Update the display
            handle.capture_stdout(message).await?;
            Ok(())
        } else {
            let ctx = ErrorContext::new("updating progress bar", "ProgressManager")
                .with_thread_id(thread_id)
                .with_details("Thread not found");
            
            let error_msg = format!("Thread {} not found", thread_id);
            let error = ProgressError::TaskOperation(error_msg).into_context(ctx);
            Err(anyhow::anyhow!(error))
        }
    }
    
    /// Update the progress bar with default configuration.
    ///
    /// This is a convenience method that uses a standard progress bar configuration.
    ///
    /// # Parameters
    /// * `thread_id` - The ID of the thread to update the progress bar for
    /// * `current` - The current number of completed items
    /// * `total` - The total number of items
    /// * `prefix` - Optional prefix to display before the progress bar
    ///
    /// # Returns
    /// Result with () if successful
    pub async fn update_progress_bar(&self, thread_id: usize, current: usize, total: usize, prefix: &str) -> Result<()> {
        let config = ProgressBarConfig::new()
            .width(50)
            .style(ProgressBarStyle::Block);
        
        let config = if prefix.is_empty() {
            config
        } else {
            config.prefix(prefix)
        };
        
        self.update_progress_bar_with_config(thread_id, current, total, &config).await
    }
    
    /// Create a progress bar with the specified configuration.
    ///
    /// # Parameters
    /// * `thread_id` - The ID of the thread to create the progress bar for
    /// * `total` - The total number of items
    /// * `config` - The progress bar configuration
    ///
    /// # Returns
    /// Result with () if successful
    pub async fn create_progress_bar(&self, thread_id: usize, total: usize, config: &ProgressBarConfig) -> Result<()> {
        // Initial update with 0 progress
        self.update_progress_bar_with_config(thread_id, 0, total, config).await
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
    
    /// Create a child task that is linked to a parent task.
    ///
    /// This method creates a new task that is a child of the specified parent task.
    /// The child task's progress will be included in the parent task's cumulative progress.
    ///
    /// # Parameters
    /// * `parent_id` - The ID of the parent task
    /// * `mode` - The display mode for the child task
    /// * `total_jobs` - The total number of jobs for the child task
    ///
    /// # Returns
    /// A Result containing the new TaskHandle, or an error if the operation failed
    pub async fn create_child_task(&self, parent_id: usize, mode: ThreadMode, total_jobs: usize) -> Result<TaskHandle> {
        // Verify parent exists
        if let Some(parent_handle) = self.thread_manager.get_task(parent_id).await {
            // Create new task
            let child_handle = self.create_task(mode, total_jobs).await?;
            let child_id = child_handle.thread_id();
            
            // Set parent-child relationship
            child_handle.set_parent_job_id(parent_id).await?;
            parent_handle.add_child_job(child_id).await?;
            
            Ok(child_handle)
        } else {
            let ctx = ErrorContext::new("creating child task", "ProgressManager")
                .with_thread_id(parent_id)
                .with_details("Parent thread not found");
            let error = ProgressError::TaskOperation(format!("Parent thread {} not found", parent_id))
                .into_context(ctx);
            Err(anyhow::anyhow!(error))
        }
    }
    
    /// Create a child task with a title that is linked to a parent task.
    ///
    /// This method creates a new task with the specified title that is a child of the specified parent task.
    /// The child task's progress will be included in the parent task's cumulative progress.
    ///
    /// # Parameters
    /// * `parent_id` - The ID of the parent task
    /// * `mode` - The display mode for the child task
    /// * `title` - The title for the child task
    /// * `total_jobs` - The total number of jobs for the child task (defaults to 1 if not specified)
    ///
    /// # Returns
    /// A Result containing the new TaskHandle, or an error if the operation failed
    pub async fn create_child_task_with_title(&self, parent_id: usize, mode: ThreadMode, title: String, total_jobs: Option<usize>) -> Result<TaskHandle> {
        // Verify parent exists
        if let Some(parent_handle) = self.thread_manager.get_task(parent_id).await {
            // Create new task with title
            let child_handle = self.create_task_with_title(mode, title).await?;
            let child_id = child_handle.thread_id();
            
            // Set total jobs if specified
            if let Some(total) = total_jobs {
                child_handle.set_total_jobs(total).await?;
            }
            
            // Set parent-child relationship
            child_handle.set_parent_job_id(parent_id).await?;
            parent_handle.add_child_job(child_id).await?;
            
            Ok(child_handle)
        } else {
            let ctx = ErrorContext::new("creating child task with title", "ProgressManager")
                .with_thread_id(parent_id)
                .with_details("Parent thread not found");
            let error = ProgressError::TaskOperation(format!("Parent thread {} not found", parent_id))
                .into_context(ctx);
            Err(anyhow::anyhow!(error))
        }
    }
    
    /// Get the child tasks of a parent task.
    ///
    /// # Parameters
    /// * `parent_id` - The ID of the parent task
    ///
    /// # Returns
    /// A Result containing a vector of child TaskHandles, or an error if the operation failed
    pub async fn get_child_tasks(&self, parent_id: usize) -> Result<Vec<TaskHandle>> {
        if let Some(parent_handle) = self.thread_manager.get_task(parent_id).await {
            let child_ids = parent_handle.get_child_job_ids().await?;
            let mut child_tasks = Vec::new();
            
            for child_id in child_ids {
                if let Some(child_handle) = self.thread_manager.get_task(child_id).await {
                    child_tasks.push(child_handle);
                }
            }
            
            Ok(child_tasks)
        } else {
            let ctx = ErrorContext::new("getting child tasks", "ProgressManager")
                .with_thread_id(parent_id)
                .with_details("Parent thread not found");
            let error = ProgressError::TaskOperation(format!("Parent thread {} not found", parent_id))
                .into_context(ctx);
            Err(anyhow::anyhow!(error))
        }
    }
    
    /// Calculate the cumulative progress of a task including all its child tasks.
    ///
    /// # Parameters
    /// * `thread_id` - The ID of the task
    ///
    /// # Returns
    /// A Result containing the cumulative progress as a percentage between 0.0 and 100.0
    pub async fn get_cumulative_progress(&self, thread_id: usize) -> Result<f64> {
        if let Some(handle) = self.thread_manager.get_task(thread_id).await {
            // Get child IDs
            let child_ids = handle.get_child_job_ids().await?;
            
            if child_ids.is_empty() {
                // No children, just return this task's progress
                return handle.get_progress_percentage().await;
            }
            
            // Get all child tasks that exist
            let mut child_task_ids = Vec::new();
            for child_id in child_ids {
                if self.thread_manager.get_task(child_id).await.is_some() {
                    child_task_ids.push(child_id);
                }
            }
            
            if child_task_ids.is_empty() {
                // No active children, just return this task's progress
                return handle.get_progress_percentage().await;
            }
            
            // Calculate weighted progress
            let parent_progress = handle.get_progress_percentage().await?;
            let mut total_progress = parent_progress;
            let num_children = child_task_ids.len() as f64;
            
            // Add each child's progress (including their children recursively)
            // Using boxed future to address recursion in async fn
            for child_id in child_task_ids {
                let child_progress_future = Box::pin(self.get_cumulative_progress(child_id));
                let child_progress = child_progress_future.await?;
                total_progress += child_progress;
            }
            
            // Average the progress (parent + all children)
            let average_progress = total_progress / (1.0 + num_children);
            Ok(average_progress)
        } else {
            let ctx = ErrorContext::new("getting cumulative progress", "ProgressManager")
                .with_thread_id(thread_id)
                .with_details("Thread not found");
            let error = ProgressError::TaskOperation(format!("Thread {} not found", thread_id))
                .into_context(ctx);
            Err(anyhow::anyhow!(error))
        }
    }
    
    /// Pause a specific thread.
    ///
    /// # Parameters
    /// * `thread_id` - The ID of the thread to pause
    ///
    /// # Returns
    /// A Result indicating success or an error
    pub async fn pause_thread(&self, thread_id: usize) -> Result<()> {
        if let Some(handle) = self.thread_manager.get_task(thread_id).await {
            // Update the thread state
            self.thread_manager.update_thread_state(thread_id, ThreadState::Paused).await?;
            // Pause the task itself
            handle.pause().await
        } else {
            let ctx = ErrorContext::new("pausing thread", "ProgressManager")
                .with_thread_id(thread_id)
                .with_details("Thread not found");
            
            let error_msg = format!("Thread {} not found", thread_id);
            let error = ProgressError::TaskOperation(error_msg).into_context(ctx);
            Err(anyhow::anyhow!(error))
        }
    }
    
    /// Resume a specific thread.
    ///
    /// # Parameters
    /// * `thread_id` - The ID of the thread to resume
    ///
    /// # Returns
    /// A Result indicating success or an error
    pub async fn resume_thread(&self, thread_id: usize) -> Result<()> {
        if let Some(handle) = self.thread_manager.get_task(thread_id).await {
            // Update the thread state
            self.thread_manager.update_thread_state(thread_id, ThreadState::Running).await?;
            // Resume the task itself
            handle.resume().await
        } else {
            let ctx = ErrorContext::new("resuming thread", "ProgressManager")
                .with_thread_id(thread_id)
                .with_details("Thread not found");
            
            let error_msg = format!("Thread {} not found", thread_id);
            let error = ProgressError::TaskOperation(error_msg).into_context(ctx);
            Err(anyhow::anyhow!(error))
        }
    }
    
    /// Check if a specific thread is paused.
    ///
    /// # Parameters
    /// * `thread_id` - The ID of the thread to check
    ///
    /// # Returns
    /// A Result containing a boolean indicating whether the thread is paused
    pub async fn is_thread_paused(&self, thread_id: usize) -> Result<bool> {
        if let Some(handle) = self.thread_manager.get_task(thread_id).await {
            handle.is_paused().await
        } else {
            let ctx = ErrorContext::new("checking thread pause state", "ProgressManager")
                .with_thread_id(thread_id)
                .with_details("Thread not found");
            
            let error_msg = format!("Thread {} not found", thread_id);
            let error = ProgressError::TaskOperation(error_msg).into_context(ctx);
            Err(anyhow::anyhow!(error))
        }
    }
    
    /// Pause all threads.
    ///
    /// # Returns
    /// A Result indicating success or an error
    pub async fn pause_all(&self) -> Result<()> {
        // Get all active threads
        let active_threads = self.thread_manager.get_active_threads().await;
        
        // Pause each thread
        for thread_id in active_threads {
            if let Err(e) = self.pause_thread(thread_id).await {
                eprintln!("Warning: Failed to pause thread {}: {}", thread_id, e);
            }
        }
        
        Ok(())
    }
    
    /// Resume all threads.
    ///
    /// # Returns
    /// A Result indicating success or an error
    pub async fn resume_all(&self) -> Result<()> {
        // Get all paused threads
        let paused_threads = self.thread_manager.get_threads_by_state(ThreadState::Paused).await;
        
        // Resume each thread
        for thread_id in paused_threads {
            if let Err(e) = self.resume_thread(thread_id).await {
                eprintln!("Warning: Failed to resume thread {}: {}", thread_id, e);
            }
        }
        
        Ok(())
    }
    
    /// Create a new multi-progress bar group
    ///
    /// # Parameters
    /// * `group_id` - The ID for the new multi-progress bar group
    ///
    /// # Returns
    /// Ok(()) if the group was created successfully
    pub async fn create_multi_progress_bar_group(&self, group_id: impl Into<String>) -> Result<()> {
        let group_id = group_id.into();
        let mut multi_bars = self.multi_bars.lock().await;
        
        if multi_bars.contains_key(&group_id) {
            let ctx = ErrorContext::new("creating multi-progress bar group", "ProgressManager")
                .with_details(format!("Group ID '{}' already exists", &group_id));
            
            let error_msg = format!("Multi-progress bar group '{}' already exists", group_id);
            let error = ProgressError::DisplayOperation(error_msg).into_context(ctx);
            return Err(anyhow::anyhow!(error));
        }
        
        multi_bars.insert(group_id, MultiProgressBar::new());
        Ok(())
    }
    
    /// Add a progress bar to a multi-progress bar group
    ///
    /// # Parameters
    /// * `group_id` - The ID of the multi-progress bar group
    /// * `bar_id` - The ID for the new progress bar
    /// * `config` - Configuration for the progress bar
    ///
    /// # Returns
    /// Ok(()) if the progress bar was added successfully
    pub async fn add_progress_bar(&self, group_id: &str, bar_id: impl Into<String>, config: ProgressBarConfig) -> Result<()> {
        let bar_id = bar_id.into();
        let mut multi_bars = self.multi_bars.lock().await;
        
        if let Some(group) = multi_bars.get_mut(group_id) {
            let bar = ProgressBar::new(config);
            group.add(bar_id, bar);
            Ok(())
        } else {
            let ctx = ErrorContext::new("adding progress bar", "ProgressManager")
                .with_details(format!("Group ID '{}' does not exist", group_id));
            
            let error_msg = format!("Multi-progress bar group '{}' not found", group_id);
            let error = ProgressError::DisplayOperation(error_msg).into_context(ctx);
            Err(anyhow::anyhow!(error))
        }
    }
    
    /// Update a progress bar in a multi-progress bar group
    ///
    /// # Parameters
    /// * `group_id` - The ID of the multi-progress bar group
    /// * `bar_id` - The ID of the progress bar to update
    /// * `current` - The current value
    /// * `total` - The total value
    ///
    /// # Returns
    /// Ok(()) if the progress bar was updated successfully
    pub async fn update_multi_progress_bar(&self, group_id: &str, bar_id: &str, current: usize, total: usize) -> Result<()> {
        let mut multi_bars = self.multi_bars.lock().await;
        
        if let Some(group) = multi_bars.get_mut(group_id) {
            group.update_with_values(bar_id, current, total);
            Ok(())
        } else {
            let ctx = ErrorContext::new("updating multi-progress bar", "ProgressManager")
                .with_details(format!("Group ID '{}' does not exist", group_id));
            
            let error_msg = format!("Multi-progress bar group '{}' not found", group_id);
            let error = ProgressError::DisplayOperation(error_msg).into_context(ctx);
            Err(anyhow::anyhow!(error))
        }
    }
    
    /// Remove a progress bar from a multi-progress bar group
    ///
    /// # Parameters
    /// * `group_id` - The ID of the multi-progress bar group
    /// * `bar_id` - The ID of the progress bar to remove
    ///
    /// # Returns
    /// Ok(()) if the progress bar was removed successfully
    pub async fn remove_progress_bar(&self, group_id: &str, bar_id: &str) -> Result<()> {
        let mut multi_bars = self.multi_bars.lock().await;
        
        if let Some(group) = multi_bars.get_mut(group_id) {
            group.remove(bar_id);
            Ok(())
        } else {
            let ctx = ErrorContext::new("removing progress bar", "ProgressManager")
                .with_details(format!("Group ID '{}' does not exist", group_id));
            
            let error_msg = format!("Multi-progress bar group '{}' not found", group_id);
            let error = ProgressError::DisplayOperation(error_msg).into_context(ctx);
            Err(anyhow::anyhow!(error))
        }
    }
    
    /// Remove a multi-progress bar group
    ///
    /// # Parameters
    /// * `group_id` - The ID of the multi-progress bar group to remove
    ///
    /// # Returns
    /// Ok(()) if the group was removed successfully
    pub async fn remove_multi_progress_bar_group(&self, group_id: &str) -> Result<()> {
        let mut multi_bars = self.multi_bars.lock().await;
        
        if multi_bars.remove(group_id).is_some() {
            Ok(())
        } else {
            let ctx = ErrorContext::new("removing multi-progress bar group", "ProgressManager")
                .with_details(format!("Group ID '{}' does not exist", group_id));
            
            let error_msg = format!("Multi-progress bar group '{}' not found", group_id);
            let error = ProgressError::DisplayOperation(error_msg).into_context(ctx);
            Err(anyhow::anyhow!(error))
        }
    }
    
    /// Get the rendered output of a multi-progress bar group
    ///
    /// # Parameters
    /// * `group_id` - The ID of the multi-progress bar group
    ///
    /// # Returns
    /// The rendered output of the multi-progress bar group
    pub async fn render_multi_progress_bar_group(&self, group_id: &str) -> Result<String> {
        let multi_bars = self.multi_bars.lock().await;
        
        if let Some(group) = multi_bars.get(group_id) {
            Ok(group.render())
        } else {
            let ctx = ErrorContext::new("rendering multi-progress bar group", "ProgressManager")
                .with_details(format!("Group ID '{}' does not exist", group_id));
            
            let error_msg = format!("Multi-progress bar group '{}' not found", group_id);
            let error = ProgressError::DisplayOperation(error_msg).into_context(ctx);
            Err(anyhow::anyhow!(error))
        }
    }
    
    /// Display a multi-progress bar group on a specific task
    ///
    /// # Parameters
    /// * `thread_id` - The ID of the thread to display on
    /// * `group_id` - The ID of the multi-progress bar group
    ///
    /// # Returns
    /// Ok(()) if the multi-progress bar group was displayed successfully
    pub async fn display_multi_progress_bar_group(&self, thread_id: usize, group_id: &str) -> Result<()> {
        let rendered = self.render_multi_progress_bar_group(group_id).await?;
        
        if let Some(mut handle) = self.thread_manager.get_task(thread_id).await {
            handle.capture_stdout(rendered).await?;
            Ok(())
        } else {
            let ctx = ErrorContext::new("displaying multi-progress bar group", "ProgressManager")
                .with_thread_id(thread_id)
                .with_details("Thread not found");
            
            let error_msg = format!("Thread {} not found", thread_id);
            let error = ProgressError::TaskOperation(error_msg).into_context(ctx);
            Err(anyhow::anyhow!(error))
        }
    }
    
    /// Get a reference to the multi-progress bar map
    pub fn multi_bars(&self) -> &Arc<Mutex<HashMap<String, MultiProgressBar>>> {
        &self.multi_bars
    }
} 