use crate::modes::ThreadMode;
use crate::ProgressDisplay;
use crate::terminal::TestEnv;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;

/// TestBuilder provides a fluent interface for creating and configuring
/// test environments for different display modes with reduced boilerplate.
pub struct TestBuilder {
    /// The test environment for output verification
    env: TestEnv,
    /// Display mode to test
    mode: Option<ThreadMode>,
    /// Whether to create a ProgressDisplay instance
    create_display: bool,
}

impl TestBuilder {
    /// Create a new TestBuilder with default terminal size (80x24)
    pub fn new() -> Self {
        Self::with_size(80, 24)
    }

    /// Create a new TestBuilder with specified terminal size
    pub fn with_size(width: u16, height: u16) -> Self {
        Self {
            env: TestEnv::new(),
            mode: None,
            create_display: false,
        }
    }

    /// Set the display mode for testing
    pub fn mode(mut self, mode: ThreadMode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Configure to create a ProgressDisplay instance
    pub fn with_display(mut self) -> Self {
        self.create_display = true;
        self
    }

    /// Get the test environment
    pub fn env(&mut self) -> &mut TestEnv {
        &mut self.env
    }

    /// Set up for window mode with specified height
    pub fn window_mode(self, height: usize) -> Self {
        self.mode(ThreadMode::Window(height))
    }

    /// Set up for window with title mode with specified height
    pub fn window_with_title_mode(self, height: usize) -> Self {
        self.mode(ThreadMode::WindowWithTitle(height))
    }

    /// Set up for limited mode
    pub fn limited_mode(self) -> Self {
        self.mode(ThreadMode::Limited)
    }

    /// Set up for capturing mode
    pub fn capturing_mode(self) -> Self {
        self.mode(ThreadMode::Capturing)
    }

    /// Create a ProgressDisplay with the configured mode
    pub async fn build_display(&self) -> ProgressDisplay {
        match self.mode {
            Some(mode) => ProgressDisplay::new_with_mode(mode).await.expect("Failed to create display"),
            None => ProgressDisplay::new().await.expect("Failed to create display"),
        }
    }

    /// Run a simple message test with the specified message
    pub async fn test_message(&mut self, message: &str) -> Result<ProgressDisplay> {
        let display = self.build_display().await;
        let mode = self.mode.unwrap_or(ThreadMode::Limited);
        let mut task = display.spawn_with_mode(mode, || "test").await?;
        
        // Send the message using the task handle
        task.capture_stdout(message.to_string()).await?;
        task.join().await?;
        
        Ok(display)
    }

    /// Run a test with multiple messages
    pub async fn test_messages(&mut self, messages: &[&str]) -> Result<ProgressDisplay> {
        let display = self.build_display().await;
        let mode = self.mode.unwrap_or(ThreadMode::Limited);
        let mut task = display.spawn_with_mode(mode, || "test").await?;
        
        for message in messages {
            task.capture_stdout(message.to_string()).await?;
        }
        task.join().await?;
        
        Ok(display)
    }

    /// Run a concurrent task test with the given number of tasks
    pub async fn test_concurrent_tasks(&mut self, task_count: usize, message_template: &str) -> Result<ProgressDisplay> {
        let display = self.build_display().await;
        let display_arc = Arc::new(Mutex::new(display));
        
        // Create tasks that will send messages
        let mut handles = Vec::with_capacity(task_count);
        
        for i in 0..task_count {
            let display_clone = Arc::clone(&display_arc);
            let message = message_template.replace("{}", &i.to_string());
            
            let handle = tokio::spawn(async move {
                let display = display_clone.lock().await;
                let mut task = display.spawn_with_mode(ThreadMode::Capturing, || "test".to_string()).await.unwrap();
                task.capture_stdout(message).await.unwrap();
                task
            });
            
            handles.push(handle);
        }
        
        // Wait for all tasks to complete and join them
        for handle in handles {
            let task = handle.await.unwrap();
            task.join().await.unwrap();
        }
        
        // For concurrent tests, we need to add expected messages in the same way
        for i in 0..task_count {
            let message = message_template.replace("{}", &i.to_string());
            self.env.writeln(&message);
        }
        
        display_arc.lock().await.display().await?;
        self.env.verify();
        
        Ok(Arc::try_unwrap(display_arc).unwrap().into_inner())
    }

    /// Run a concurrent task test with an existing display
    pub async fn test_concurrent_tasks_with_display(&mut self, display: &ProgressDisplay, task_count: usize, message_template: &str) -> Result<()> {
        let display_arc = Arc::new(Mutex::new(display.clone()));
        
        // Create tasks that will send messages
        let mut handles = Vec::with_capacity(task_count);
        
        for i in 0..task_count {
            let display_clone = Arc::clone(&display_arc);
            let message = message_template.replace("{}", &i.to_string());
            
            let handle = tokio::spawn(async move {
                let display = display_clone.lock().await;
                let mut task = display.spawn_with_mode(ThreadMode::Capturing, || "test".to_string()).await.unwrap();
                task.capture_stdout(message).await.unwrap();
                task
            });
            
            handles.push(handle);
        }
        
        // Wait for all tasks to complete and join them
        for handle in handles {
            let task = handle.await.unwrap();
            task.join().await.unwrap();
        }
        
        // For concurrent tests, we need to add expected messages in the same way
        for i in 0..task_count {
            let message = message_template.replace("{}", &i.to_string());
            self.env.writeln(&message);
        }
        
        display.display().await?;
        self.env.verify();
        
        Ok(())
    }

    /// Test error handling by sending an error message
    pub async fn test_error(&mut self, error_message: &str) -> Result<ProgressDisplay> {
        let display = self.build_display().await;
        let mut task = display.spawn_with_mode(self.mode.unwrap(), || "test").await?;
        
        // Format and send as error message
        task.capture_stderr(error_message.to_string()).await?;
        task.join().await?;
        
        Ok(display)
    }

    /// Test edge cases specific to different modes using an existing display
    pub async fn test_edge_case_with_display(&mut self, display: &ProgressDisplay, case_type: EdgeCaseType) -> Result<()> {
        // Ensure we have a valid mode with appropriate window size for window modes
        let mode = match self.mode {
            Some(ThreadMode::Window(_)) => ThreadMode::Window(3),
            Some(ThreadMode::WindowWithTitle(_)) => ThreadMode::WindowWithTitle(3),
            Some(mode) => mode,
            None => ThreadMode::Limited,
        };
        
        let mut task = display.spawn_with_mode(mode, || "test").await?;
        
        match case_type {
            EdgeCaseType::EmptyMessage => {
                task.capture_stdout("".to_string()).await?;
            }
            EdgeCaseType::LongMessage(length) => {
                task.capture_stdout("x".repeat(length)).await?;
            }
            EdgeCaseType::SpecialChars => {
                task.capture_stdout("!@#$%^&*()".to_string()).await?;
            }
            EdgeCaseType::UnicodeCharacters => {
                task.capture_stdout("你好, こんにちは, 안녕하세요 🚀🔥🌈".to_string()).await?;
            }
        }
        task.join().await?;
        
        Ok(())
    }

    /// Test window-specific features
    pub async fn test_window_features(&mut self, lines: &[&str]) -> Result<ProgressDisplay> {
        // Ensure we're in Window mode
        if self.mode.is_none() {
            self.mode = Some(ThreadMode::Window(lines.len()));
        }
        
        let display = self.build_display().await;
        let mut task = display.spawn_with_mode(self.mode.unwrap(), || "test").await?;
        
        // Send messages to fill the window
        for line in lines {
            task.capture_stdout(line.to_string()).await?;
            self.env.writeln(line);
        }
        
        task.join().await?;
        Ok(display)
    }

    /// Test window with title specific features
    pub async fn test_window_with_title_features(&mut self, title: &str, lines: &[&str]) -> Result<ProgressDisplay> {
        // Ensure we're in WindowWithTitle mode
        if self.mode.is_none() {
            self.mode = Some(ThreadMode::WindowWithTitle(lines.len() + 1)); // +1 for title
        }
        
        let display = self.build_display().await;
        let mut task = display.spawn_with_mode(self.mode.unwrap(), || "test").await?;
        let thread_id = task.thread_id();
        
        // Set the title
        display.set_title(thread_id, title.to_string()).await?;
        self.env.writeln(title);
        
        // Send messages to fill the window
        for line in lines {
            task.capture_stdout(line.to_string()).await?;
            self.env.writeln(line);
        }
        
        task.join().await?;
        Ok(display)
    }

    /// Test limited mode features
    pub async fn test_limited_features(&mut self, messages: &[&str]) -> Result<ProgressDisplay> {
        // Ensure we're in Limited mode
        if self.mode.is_none() {
            self.mode = Some(ThreadMode::Limited);
        }
        
        let display = self.build_display().await;
        let mut task = display.spawn_with_mode(self.mode.unwrap(), || "test").await?;
        
        // Send all messages, but only the last one should be displayed
        for (i, message) in messages.iter().enumerate() {
            let message_str = message.to_string();
            task.capture_stdout(message_str.clone()).await?;
            
            // Clear expected output if not the last message
            if i < messages.len() - 1 {
                self.env.clear();
            } else {
                self.env.writeln(&message_str);
            }
        }
        
        self.env.verify();
        task.join().await?;
        Ok(display)
    }

    /// Test capturing mode features
    pub async fn test_capturing_features(&mut self, messages: &[&str]) -> Result<ProgressDisplay> {
        // Ensure we're in Capturing mode
        if self.mode.is_none() {
            self.mode = Some(ThreadMode::Capturing);
        }
        
        let display = self.build_display().await;
        let mut task = display.spawn_with_mode(ThreadMode::Capturing, || "test").await?;
        
        // Send all messages, they should be captured but not displayed
        for message in messages {
            task.capture_stdout(message.to_string()).await?;
        }
        
        task.join().await?;
        
        // In capturing mode, nothing should be displayed immediately
        self.env.verify();
        
        Ok(display)
    }

    /// Test progress updating (for all modes)
    pub async fn test_progress_update(&mut self, total_jobs: usize, messages_per_job: usize) -> Result<ProgressDisplay> {
        let display = self.build_display().await;
        
        let mut task = display.spawn_with_mode(self.mode.unwrap(), || "test").await?;
        let thread_id = task.thread_id();
        
        // Set total jobs in the display
        display.set_total_jobs(None, total_jobs).await?;
        
        // Send messages and increment completed jobs
        for job in 0..total_jobs {
            for msg_idx in 0..messages_per_job {
                let message = format!("Job {} - Message {}", job, msg_idx);
                // Use the same task handle for all messages
                task.capture_stdout(message.clone()).await?;
                self.env.writeln(&message);
            }
            
            display.update_progress(thread_id, job + 1, total_jobs, "Progress").await?;
        }
        
        task.join().await?;
        Ok(display)
    }

    pub async fn test_concurrent_messages(&mut self, messages: &[&str]) -> Result<ProgressDisplay> {
        let display = Arc::new(Mutex::new(self.build_display().await));
        let mut handles = vec![];

        for message in messages {
            let display_clone = Arc::clone(&display);
            let message = message.to_string();

            let handle = tokio::spawn(async move {
                let display = display_clone.lock().await;
                let mut task = display.spawn_with_mode(ThreadMode::Capturing, || "test").await.unwrap();
                task.capture_stdout(message).await.unwrap();
                task
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await?.join().await?;
        }

        Ok(Arc::try_unwrap(display).unwrap().into_inner())
    }

    pub async fn test_window_overflow(&mut self) -> Result<ProgressDisplay> {
        let display = self.build_display().await;
        let mut task = display.spawn_with_mode(self.mode.unwrap(), || "test").await?;
        
        // Send messages to fill the window
        for i in 0..10 {
            task.capture_stdout(format!("Message {}", i)).await?;
        }
        task.join().await?;
        
        Ok(display)
    }

    pub async fn test_thread_id(&mut self) -> Result<ProgressDisplay> {
        let display = self.build_display().await;
        let mut task = display.spawn_with_mode(self.mode.unwrap(), || "test").await?;
        let thread_id = task.thread_id();
        
        // Send a message with the thread ID
        task.capture_stdout(format!("Thread ID: {}", thread_id)).await?;
        task.join().await?;
        
        Ok(display)
    }

    pub async fn test_mode_specific(&mut self) -> Result<ProgressDisplay> {
        let display = self.build_display().await;
        let mut task = display.spawn_with_mode(self.mode.unwrap(), || "test").await?;
        
        // Send all messages, but only the last one should be displayed
        for i in 0..5 {
            task.capture_stdout(format!("Message {}", i)).await?;
        }
        task.join().await?;
        
        Ok(display)
    }

    pub async fn test_capturing(&mut self) -> Result<ProgressDisplay> {
        let display = self.build_display().await;
        let mut task = display.spawn_with_mode(ThreadMode::Capturing, || "test").await?;
        
        // Send all messages, they should be captured but not displayed
        for i in 0..5 {
            task.capture_stdout(format!("Message {}", i)).await?;
        }
        task.join().await?;
        
        Ok(display)
    }

    pub async fn test_thread_management(&mut self) -> Result<ProgressDisplay> {
        let display = self.build_display().await;
        let mut task = display.spawn_with_mode(self.mode.unwrap(), || "test").await?;
        let thread_id = task.thread_id();
        
        // Send messages to test thread management
        task.capture_stdout(format!("Thread {} starting", thread_id)).await?;
        task.capture_stdout(format!("Thread {} running", thread_id)).await?;
        task.capture_stdout(format!("Thread {} stopping", thread_id)).await?;
        task.join().await?;
        
        Ok(display)
    }

    pub async fn test_spawn(&mut self) -> Result<ProgressDisplay> {
        let display = self.build_display().await;
        let mut task = display.spawn(|_| async move { Ok(()) }).await?;
        
        // Basic spawn test
        task.capture_stdout("Spawned task".to_string()).await?;
        task.join().await?;
        
        Ok(display)
    }
}

/// Types of edge cases that can be tested
pub enum EdgeCaseType {
    /// Test with an empty message
    EmptyMessage,
    /// Test with a very long message
    LongMessage(usize),
    /// Test with special characters
    SpecialChars,
    /// Test with Unicode characters
    UnicodeCharacters,
} 