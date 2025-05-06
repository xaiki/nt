use crate::modes::ThreadMode;
use crate::ProgressDisplay;
use super::common::TestEnv;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;

/// TestBuilder provides a fluent interface for creating and configuring
/// test environments for different display modes with reduced boilerplate.
pub struct TestBuilder {
    /// The test environment for output verification
    env: TestEnv,
    /// Terminal width
    width: u16,
    /// Terminal height
    height: u16,
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
            env: TestEnv::new(width, height),
            width,
            height,
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
            Some(mode) => ProgressDisplay::new_with_mode(mode).await,
            None => ProgressDisplay::new().await,
        }
    }

    /// Run a simple message test with the specified message
    pub async fn test_message(&mut self, message: &str) -> Result<ProgressDisplay> {
        let display = self.build_display().await;
        let mut task = display.spawn(|| {}).await?;
        
        // Send the message using the task handle
        task.capture_stdout(message.to_string()).await?;
        display.display().await?;
        
        self.env.writeln(message);
        self.env.verify();
        
        Ok(display)
    }

    /// Run a test with multiple messages
    pub async fn test_messages(&mut self, messages: &[&str]) -> Result<ProgressDisplay> {
        let display = self.build_display().await;
        let mut task = display.spawn(|| {}).await?;
        
        for message in messages {
            task.capture_stdout(message.to_string()).await?;
            display.display().await?;
            self.env.writeln(message);
            self.env.verify();
        }
        
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
                let mut task = display.spawn(|| {}).await.unwrap();
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
        
        // Get the display back and verify
        let display = Arc::try_unwrap(display_arc)
            .expect("There are still references to the display")
            .into_inner();
        
        display.display().await?;
        
        // For concurrent tests, we need to add expected messages in the same way
        for i in 0..task_count {
            let message = message_template.replace("{}", &i.to_string());
            self.env.writeln(&message);
        }
        
        self.env.verify();
        
        Ok(display)
    }

    /// Test error handling by sending an error message
    pub async fn test_error(&mut self, error_message: &str) -> Result<ProgressDisplay> {
        let display = self.build_display().await;
        let mut task = display.spawn(|| {}).await?;
        
        // Format and send as error message
        task.capture_stderr(format!("ERROR: {}", error_message)).await?;
        display.display().await?;
        
        // Format expected error output
        self.env.writeln(&format!("ERROR: {}", error_message));
        self.env.verify();
        
        Ok(display)
    }

    /// Test edge cases specific to different modes
    pub async fn test_edge_case(&mut self, case_type: EdgeCaseType) -> Result<ProgressDisplay> {
        let display = self.build_display().await;
        let mut task = display.spawn(|| {}).await?;
        
        match case_type {
            EdgeCaseType::EmptyMessage => {
                task.capture_stdout("".to_string()).await?;
                display.display().await?;
                // Empty message usually results in no output
                self.env.verify();
            },
            EdgeCaseType::LongMessage(length) => {
                let long_message = "A".repeat(length);
                task.capture_stdout(long_message.clone()).await?;
                display.display().await?;
                
                // Long messages might be truncated depending on the mode
                self.env.writeln(&long_message);
                self.env.verify();
            },
            EdgeCaseType::SpecialCharacters => {
                let special_message = "Special chars: !@#$%^&*()_+{}|:<>?~`-=[]\\;',./";
                task.capture_stdout(special_message.to_string()).await?;
                display.display().await?;
                
                self.env.writeln(special_message);
                self.env.verify();
            },
            EdgeCaseType::UnicodeCharacters => {
                let unicode_message = "Unicode: 😀 🚀 👍 ❤️ 🔥 🌟 🎉 🙏 🌈 ✨";
                task.capture_stdout(unicode_message.to_string()).await?;
                display.display().await?;
                
                self.env.writeln(unicode_message);
                self.env.verify();
            },
        }
        
        Ok(display)
    }

    /// Test window-specific features
    pub async fn test_window_features(&mut self, lines: &[&str]) -> Result<ProgressDisplay> {
        // Ensure we're in Window mode
        if self.mode.is_none() {
            self.mode = Some(ThreadMode::Window(lines.len()));
        }
        
        let display = self.build_display().await;
        let mut task = display.spawn_with_mode(self.mode.unwrap(), || {}).await?;
        
        // Send messages to fill the window
        for line in lines {
            task.capture_stdout(line.to_string()).await?;
            self.env.writeln(line);
        }
        
        display.display().await?;
        self.env.verify();
        
        Ok(display)
    }

    /// Test window with title specific features
    pub async fn test_window_with_title_features(&mut self, title: &str, lines: &[&str]) -> Result<ProgressDisplay> {
        // Ensure we're in WindowWithTitle mode
        if self.mode.is_none() {
            self.mode = Some(ThreadMode::WindowWithTitle(lines.len() + 1)); // +1 for title
        }
        
        let display = self.build_display().await;
        let mut task = display.spawn_with_mode(self.mode.unwrap(), || {}).await?;
        let thread_id = task.thread_id();
        
        // Set the title
        display.set_title(thread_id, title.to_string()).await?;
        self.env.writeln(title);
        
        // Send messages to fill the window
        for line in lines {
            task.capture_stdout(line.to_string()).await?;
            self.env.writeln(line);
        }
        
        display.display().await?;
        self.env.verify();
        
        Ok(display)
    }

    /// Test limited mode features
    pub async fn test_limited_features(&mut self, messages: &[&str]) -> Result<ProgressDisplay> {
        // Ensure we're in Limited mode
        if self.mode.is_none() {
            self.mode = Some(ThreadMode::Limited);
        }
        
        let display = self.build_display().await;
        let mut task = display.spawn_with_mode(self.mode.unwrap(), || {}).await?;
        
        // Send all messages, but only the last one should be displayed
        for (i, message) in messages.iter().enumerate() {
            task.capture_stdout(message.to_string()).await?;
            display.display().await?;
            
            // Clear expected output if not the last message
            if i < messages.len() - 1 {
                self.env.clear();
            } else {
                self.env.writeln(message);
            }
        }
        
        self.env.verify();
        Ok(display)
    }

    /// Test capturing mode features
    pub async fn test_capturing_features(&mut self, messages: &[&str]) -> Result<ProgressDisplay> {
        // Ensure we're in Capturing mode
        if self.mode.is_none() {
            self.mode = Some(ThreadMode::Capturing);
        }
        
        let display = self.build_display().await;
        let mut task = display.spawn_with_mode(ThreadMode::Capturing, || {}).await?;
        
        // Send all messages, they should be captured but not displayed
        for message in messages {
            task.capture_stdout(message.to_string()).await?;
        }
        
        display.display().await?;
        
        // In capturing mode, nothing should be displayed immediately
        self.env.verify();
        
        // Now we'd want to get the captured output, but we don't have direct access
        // to that method. In a real test, we'd use task.get_captured_output() if available.
        
        task.join().await?;
        Ok(display)
    }

    /// Test progress updating (for all modes)
    pub async fn test_progress_update(&mut self, total_jobs: usize, messages_per_job: usize) -> Result<ProgressDisplay> {
        let display = self.build_display().await;
        let mut task = display.spawn(|| {}).await?;
        let thread_id = task.thread_id();
        
        // Set total jobs in the display
        display.set_total_jobs(total_jobs).await?;
        
        // Send messages and increment completed jobs
        for job in 0..total_jobs {
            for msg_idx in 0..messages_per_job {
                let message = format!("Job {} - Message {}", job, msg_idx);
                // Use the same task handle for all messages
                task.capture_stdout(message.clone()).await?;
                self.env.writeln(&message);
                display.display().await?;
                self.env.verify();
            }
            
            display.update_progress(thread_id, job + 1, total_jobs, "Progress").await?;
        }
        
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
    SpecialCharacters,
    /// Test with Unicode characters
    UnicodeCharacters,
} 