#![allow(unused_imports)]
#![feature(internal_output_capture)]

use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use std::collections::{HashMap, VecDeque};
use std::io::{Write, stderr, stdout, Stdout, Stderr};
use anyhow::{Result, anyhow};
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use crossterm::terminal::size;
use rand::Rng;
use std::time::Duration;
use serial_test::serial;
use std::future::Future;
use tokio::task::LocalSet;
use std::cell::RefCell;
use stdio_override::{StdoutOverride, StderrOverride};
use std::io::{self, Read};
use tokio::task::JoinHandle;
use std::sync::atomic::{AtomicBool};

thread_local! {
    static CURRENT_THREAD_ID: AtomicUsize = AtomicUsize::new(0);
    static CURRENT_WRITER: RefCell<Option<ThreadWriter>> = RefCell::new(None);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThreadMode {
    Capturing,    // All lines output, last one constantly displayed
    Limited,      // Only last line displayed
    Window(usize), // Last n lines displayed
    WindowWithTitle(usize), // Last n lines displayed with title
}

#[derive(Debug, Clone)]
struct ThreadConfig {
    mode: ThreadMode,
    lines_to_display: usize,
    result_emoji_stack: String,
    total_jobs: usize,
    completed_jobs: Arc<AtomicUsize>,
    title: Option<String>,
}

impl ThreadConfig {
    fn new(mode: ThreadMode, total_jobs: usize) -> Self {
        Self {
            mode,
            lines_to_display: match mode {
                ThreadMode::Window(n) | ThreadMode::WindowWithTitle(n) => n,
                _ => 1,
            },
            result_emoji_stack: String::new(),
            total_jobs,
            completed_jobs: Arc::new(AtomicUsize::new(0)),
            title: None,
        }
    }

    fn update_mode(&mut self, mode: ThreadMode) {
        self.mode = mode;
        self.lines_to_display = match mode {
            ThreadMode::Window(n) | ThreadMode::WindowWithTitle(n) => n,
            _ => 1,
        };
    }
}

impl Default for ThreadConfig {
    fn default() -> Self {
        Self::new(ThreadMode::Limited, 1)
    }
}

#[derive(Debug)]
pub struct ThreadMessage {
    pub thread_id: usize,
    pub lines: Vec<String>,
    pub config: ThreadConfig,
}

#[derive(Debug, Clone)]
pub struct ProgressDisplay {
    outputs: Arc<Mutex<HashMap<usize, Vec<String>>>>,
    spinner_index: Arc<AtomicUsize>,
    terminal_size: Arc<Mutex<(u16, u16)>>,
    processing_task: Arc<Mutex<Option<JoinHandle<()>>>>,
    thread_handles: Arc<Mutex<HashMap<usize, JoinHandle<()>>>>,
    next_thread_id: Arc<AtomicUsize>,
    running: Arc<AtomicBool>,
    message_tx: mpsc::Sender<ThreadMessage>,
    message_rx: Arc<Mutex<mpsc::Receiver<ThreadMessage>>>,
}

impl ProgressDisplay {
    pub async fn new() -> Self {
        let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));
        let (message_tx, message_rx) = mpsc::channel(1024);

        let progress = Self {
            outputs: Arc::new(Mutex::new(HashMap::new())),
            spinner_index: Arc::new(AtomicUsize::new(0)),
            terminal_size: Arc::new(Mutex::new((width, height))),
            processing_task: Arc::new(Mutex::new(None)),
            thread_handles: Arc::new(Mutex::new(HashMap::new())),
            next_thread_id: Arc::new(AtomicUsize::new(0)),
            running: Arc::new(AtomicBool::new(true)),
            message_tx,
            message_rx: Arc::new(Mutex::new(message_rx)),
        };

        // Start the display thread
        let handle = Self::start_display_thread(
            progress.outputs.clone(),
            progress.terminal_size.clone(),
            progress.spinner_index.clone(),
            progress.message_rx.clone(),
            progress.running.clone(),
        ).await;
        *progress.processing_task.lock().await = Some(handle);

        progress
    }

    pub async fn spawn<F, R>(&self, f: F) -> Result<TaskHandle>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + std::fmt::Debug + 'static,
    {
        let thread_id = self.next_thread_id.fetch_add(1, Ordering::SeqCst);
        let message_tx = self.message_tx.clone();
        let progress = Arc::new(self.clone());
        
        let handle = tokio::spawn(async move {
            let output = f();
            let lines = vec![format!("{:?}", output)];
            if let Err(e) = message_tx.send(ThreadMessage { 
                thread_id, 
                lines,
                config: ThreadConfig::default(),
            }).await {
                eprintln!("[Error] Failed to send message for thread {}: {}", thread_id, e);
            }
        });
        
        let mut handles = self.thread_handles.lock().await;
        handles.insert(thread_id, handle);
        
        Ok(TaskHandle::new(thread_id, progress))
    }

    async fn start_display_thread(
        outputs: Arc<Mutex<HashMap<usize, Vec<String>>>>,
        terminal_size: Arc<Mutex<(u16, u16)>>,
        spinner_index: Arc<AtomicUsize>,
        message_rx: Arc<Mutex<mpsc::Receiver<ThreadMessage>>>,
        running: Arc<AtomicBool>,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
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
                let result: std::io::Result<()> = async {
                    let mut outputs = outputs.lock().await;
                    let (_width, height) = *terminal_size.lock().await;
                    
                    // Get all thread IDs and sort them
                    let mut thread_ids: Vec<usize> = outputs.keys().copied().collect();
                    thread_ids.sort();
                    
                    // Calculate total lines needed and adjust for terminal height
                    let mut total_lines = 0;
                    let mut display_lines = Vec::new();
                    
                    // First pass: calculate total lines and collect lines to display
                    for thread_id in &thread_ids {
                        if let Some(lines) = outputs.get(thread_id) {
                            total_lines += lines.len();
                            display_lines.extend(lines.iter().cloned());
                        }
                    }
                    
                    // If we need more lines than the terminal can display, adjust
                    if total_lines > height as usize {
                        let available_lines = height as usize;
                        let lines_per_thread = available_lines / outputs.len();
                        
                        // Adjust each thread's window size
                        for (_, lines) in outputs.iter_mut() {
                            if lines.len() > lines_per_thread {
                                lines.drain(0..lines.len() - lines_per_thread);
                            }
                        }
                        
                        // Recalculate total lines after adjustment
                        total_lines = 0;
                        display_lines.clear();
                        for thread_id in &thread_ids {
                            if let Some(lines) = outputs.get(thread_id) {
                                total_lines += lines.len();
                                display_lines.extend(lines.iter().cloned());
                            }
                        }
                    }
                    
                    // Move cursor up by the number of lines we're actually going to display
                    let mut stderr = std::io::stderr().lock();
                    write!(stderr, "\x1B[{}A", total_lines)?;
                    
                    // Update spinner
                    let spinner_chars = ["▏▎▍", "▎▍▌", "▍▌▋", "▌▋▊", "▋▊▉", "▊▉█", "▉█▉", "█▉▊", "▉▊▋", "▊▋▌", "▋▌▍", "▌▍▎", "▍▎▏"];
                    let mut spinner_index = spinner_index.fetch_add(1, Ordering::SeqCst);
                    spinner_index = (spinner_index + 1) % spinner_chars.len();
                    
                    // Print all lines in order
                    for thread_id in thread_ids {
                        if let Some(lines) = outputs.get(&thread_id) {
                            for line in lines {
                                writeln!(stderr, "{} {}", spinner_chars[spinner_index], line)?;
                            }
                        }
                    }
                    
                    stderr.flush()?;
                    Ok(())
                }.await;

                if let Err(e) = result {
                    eprintln!("Error updating display: {}", e);
                }

                // Sleep a bit to prevent CPU spinning
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        })
    }

    pub async fn stop(&self) -> Result<()> {
        self.running.store(false, Ordering::SeqCst);
        self.join_all().await?;
        
        if let Some(handle) = self.processing_task.lock().await.take() {
            handle.await.map_err(|e| anyhow!("Failed to join processing task: {}", e))?;
        }
        Ok(())
    }

    pub async fn set_title(&self, title: String) -> Result<()> {
        let mut outputs = self.outputs.lock().await;
        if let Some(lines) = outputs.get_mut(&0) {
            if lines.is_empty() {
                lines.push(title);
            } else {
                lines[0] = title;
            }
        } else {
            outputs.insert(0, vec![title]);
        }
        Ok(())
    }

    pub async fn set_total_jobs(&self, total: usize) -> Result<()> {
        let mut outputs = self.outputs.lock().await;
        if let Some(lines) = outputs.get_mut(&0) {
            if lines.is_empty() {
                lines.push(format!("Total jobs: {}", total));
            } else {
                lines[0] = format!("Total jobs: {}", total);
            }
        } else {
            outputs.insert(0, vec![format!("Total jobs: {}", total)]);
        }
        Ok(())
    }

    pub async fn capture_stdout(&self, output: String) -> Result<()> {
        let mut outputs = self.outputs.lock().await;
        let thread_outputs = outputs.entry(0).or_insert_with(Vec::new);
        thread_outputs.push(output);
        Ok(())
    }

    pub async fn capture_stderr(&self, output: String) -> Result<()> {
        let mut outputs = self.outputs.lock().await;
        let thread_outputs = outputs.entry(0).or_insert_with(Vec::new);
        thread_outputs.push(output);
        Ok(())
    }

    pub async fn update_progress(&self, thread_id: usize, current: usize, total: usize, prefix: &str) -> Result<()> {
        if total == 0 {
            return Err(anyhow!("Total jobs cannot be zero"));
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

    pub async fn display(&self) -> std::io::Result<()> {
        let mut outputs = self.outputs.lock().await;
        if outputs.is_empty() {
            return Ok(());
        }

        let mut stderr = std::io::stderr().lock();
        let (_width, height) = *self.terminal_size.lock().await;
        
        // Get all thread IDs and sort them
        let mut thread_ids: Vec<usize> = outputs.keys().copied().collect();
        thread_ids.sort();
        
        // Calculate total lines needed and adjust for terminal height
        let mut total_lines = 0;
        let mut display_lines = Vec::new();
        
        // First pass: calculate total lines and collect lines to display
        for thread_id in &thread_ids {
            if let Some(lines) = outputs.get(thread_id) {
                total_lines += lines.len();
                display_lines.extend(lines.iter().cloned());
            }
        }
        
        // If we need more lines than the terminal can display, adjust
        if total_lines > height as usize {
            let available_lines = height as usize;
            let lines_per_thread = available_lines / outputs.len();
            
            // Adjust each thread's window size
            for (_, lines) in outputs.iter_mut() {
                if lines.len() > lines_per_thread {
                    lines.drain(0..lines.len() - lines_per_thread);
                }
            }
            
            // Recalculate total lines after adjustment
            total_lines = 0;
            display_lines.clear();
            for thread_id in &thread_ids {
                if let Some(lines) = outputs.get(thread_id) {
                    total_lines += lines.len();
                    display_lines.extend(lines.iter().cloned());
                }
            }
        }
        
        // Move cursor up by the number of lines we're actually going to display
        write!(stderr, "\x1B[{}A", total_lines)?;
        
        // Update spinner
        let spinner_chars = ["▏▎▍", "▎▍▌", "▍▌▋", "▌▋▊", "▋▊▉", "▊▉█", "▉█▉", "█▉▊", "▉▊▋", "▊▋▌", "▋▌▍", "▌▍▎", "▍▎▏"];
        let mut spinner_index = self.spinner_index.load(Ordering::SeqCst);
        spinner_index = (spinner_index + 1) % spinner_chars.len();
        
        // Print all lines in order
        for thread_id in thread_ids {
            if let Some(lines) = outputs.get(&thread_id) {
                for line in lines {
                    writeln!(stderr, "{} {}", spinner_chars[spinner_index], line)?;
                }
            }
        }
        
        stderr.flush()?;
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
        handles.get(&thread_id).map(|handle| TaskHandle {
            thread_id,
            progress: Arc::new(self.clone()),
        })
    }
}

#[derive(Clone)]
struct ThreadWriter {
    thread_id: usize,
    tx: mpsc::Sender<String>,
}

impl Write for ThreadWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let msg = String::from_utf8_lossy(buf).to_string();
        
        // Try to send the message, but don't block if the channel is full
        if let Err(e) = self.tx.try_send(msg.clone()) {
            eprintln!("[Error] ThreadWriter {} failed to send message: {} (error: {:?})", self.thread_id, msg, e);
            // Return an error if the channel is full
            return Err(std::io::Error::new(std::io::ErrorKind::WouldBlock, "Channel is full"));
        }
        
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Clone)]
pub struct ThreadLogger {
    thread_id: usize,
    message_tx: mpsc::Sender<ThreadMessage>,
    config: ThreadConfig,
    lines: Vec<String>,
    spinner: usize,
}

impl ThreadLogger {
    pub fn new(thread_id: usize, message_tx: mpsc::Sender<ThreadMessage>, config: ThreadConfig) -> Self {
        Self {
            thread_id,
            message_tx,
            config,
            lines: Vec::new(),
            spinner: 0,
        }
    }

    async fn send_message(&self) -> Result<()> {
        let message = ThreadMessage {
            thread_id: self.thread_id,
            lines: self.lines.clone(),
            config: self.config.clone(),
        };
        self.message_tx.send(message).await.map_err(|e| anyhow!("Failed to send message for thread {}: {}", self.thread_id, e))
    }

    pub async fn log(&mut self, message: String) -> Result<()> {
        self.lines.push(message);
        
        // Trim lines based on mode
        match self.config.mode {
            ThreadMode::Capturing => {
                // Keep all lines, but only display last
                self.send_message().await?;
            }
            ThreadMode::Limited => {
                // Keep only last line
                if self.lines.len() > 1 {
                    self.lines.remove(0);
                }
                self.send_message().await?;
            }
            ThreadMode::Window(n) => {
                // Keep last n lines
                if self.lines.len() > n {
                    self.lines.drain(0..self.lines.len() - n);
                }
                self.send_message().await?;
            }
            ThreadMode::WindowWithTitle(n) => {
                // Keep last n lines plus title
                if self.lines.len() > n + 1 {
                    self.lines.drain(1..self.lines.len() - n);
                }
                self.send_message().await?;
            }
        }
        Ok(())
    }

    pub fn update_config(&mut self, config: ThreadConfig) {
        self.config = config;
    }
}

#[derive(Debug)]
pub struct TaskHandle {
    thread_id: usize,
    progress: Arc<ProgressDisplay>,
}

impl TaskHandle {
    pub fn new(thread_id: usize, progress: Arc<ProgressDisplay>) -> Self {
        Self { thread_id, progress }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, timeout};
    use std::time::Duration;
    use rand::Rng;
    use serial_test::serial;

    const THREAD_COUNT: usize = 10;
    const MESSAGES_PER_THREAD: usize = 20;
    const JOBS_PER_THREAD: usize = 5;
    const EMOJIS: &[&str] = &["✅", "❌", "⚠️", "🚀", "💫", "⭐", "🌟", "✨", "💥", "💪"];
    const TEST_TIMEOUT: Duration = Duration::from_secs(20);

    async fn clear_screen() {
        print!("\x1B[2J\x1B[1;1H");
    }

    fn random_message() -> String {
        let mut rng = rand::thread_rng();
        let words = ["processing", "analyzing", "scanning", "checking", "verifying", "testing", "validating", "inspecting"];
        let word = words[rng.gen_range(0..words.len())];
        let number = rng.gen_range(1..1000);
        format!("{} item {}", word, number)
    }

    fn random_emoji() -> String {
        let mut rng = rand::thread_rng();
        EMOJIS[rng.gen_range(0..EMOJIS.len())].to_string()
    }

    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn test_thread_config() {
        clear_screen().await;
        // Test ThreadConfig creation with different modes
        let config1 = ThreadConfig::new(ThreadMode::Limited, 10);
        assert_eq!(config1.lines_to_display, 1);
        assert_eq!(config1.total_jobs, 10);
        assert_eq!(config1.completed_jobs.load(Ordering::SeqCst), 0);

        let config2 = ThreadConfig::new(ThreadMode::Window(5), 10);
        assert_eq!(config2.lines_to_display, 5);

        let config3 = ThreadConfig::new(ThreadMode::WindowWithTitle(3), 10);
        assert_eq!(config3.lines_to_display, 3);

        // Test mode update
        let mut config = ThreadConfig::new(ThreadMode::Limited, 10);
        config.update_mode(ThreadMode::Window(4));
        assert_eq!(config.lines_to_display, 4);
        assert_eq!(config.mode, ThreadMode::Window(4));
    }

    #[tokio::test]
    async fn test_progress_display_basic() -> Result<()> {
        let progress = ProgressDisplay::new().await;
        
        // Spawn a task that takes 100ms
        let handle = progress.spawn(|| {
            std::thread::sleep(Duration::from_millis(100));
            "Task completed"
        }).await?;
        
        // Wait for the task to complete
        handle.join().await?;
        
        // Stop the display
        progress.stop().await?;
        
        Ok(())
    }

    #[tokio::test]
    async fn test_progress_display_multiple_threads() -> Result<()> {
        let progress = ProgressDisplay::new().await;
        
        // Spawn multiple tasks
        let mut handles = Vec::new();
        for i in 0..3 {
            let handle = progress.spawn(move || {
                std::thread::sleep(Duration::from_millis(100));
                format!("Task {} completed", i)
            }).await?;
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        for handle in handles {
            handle.join().await?;
        }
        
        // Stop the display
        progress.stop().await?;
        
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_progress_display_window_mode() {
        timeout(TEST_TIMEOUT, async {
            clear_screen().await;
            let progress = ProgressDisplay::new().await;
            
            // Create multiple progress displays
            for i in 0..THREAD_COUNT {
                // Send multiple messages with emojis
                for _ in 0..MESSAGES_PER_THREAD {
                    let emoji = random_emoji();
                    let message = format!("[test_progress_display_window_mode] [Thread {}] {} {}", i, emoji, random_message());
                    progress.capture_stdout(message).await;
                    sleep(Duration::from_millis(100)).await;
                }
                
                // Complete the jobs
                for j in 0..JOBS_PER_THREAD {
                    progress.update_progress(i, j + 1, JOBS_PER_THREAD, &format!("[test_progress_display_window_mode] [Thread {}]", i)).await;
                    sleep(Duration::from_millis(200)).await;
                }
            }

            // Allow time for final updates
            sleep(Duration::from_millis(200)).await;
            
            // Clean up
            progress.stop().await;
        }).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_progress_display_capturing_mode() {
        timeout(TEST_TIMEOUT, async {
            clear_screen().await;
            let progress = ProgressDisplay::new().await;
            
            // Create multiple progress displays
            for i in 0..THREAD_COUNT {
                // Send multiple messages with emojis
                for _ in 0..MESSAGES_PER_THREAD {
                    let emoji = random_emoji();
                    let message = format!("[test_progress_display_capturing_mode] [Thread {}] {} {}", i, emoji, random_message());
                    progress.capture_stdout(message).await;
                    sleep(Duration::from_millis(100)).await;
                }
                
                // Complete the jobs
                for j in 0..JOBS_PER_THREAD {
                    progress.update_progress(i, j + 1, JOBS_PER_THREAD, &format!("[test_progress_display_capturing_mode] [Thread {}]", i)).await;
                    sleep(Duration::from_millis(200)).await;
                }
            }

            // Allow time for final updates
            sleep(Duration::from_millis(200)).await;
            
            // Verify all threads are present
            let outputs = progress.outputs.lock().await;
            assert_eq!(outputs.len(), THREAD_COUNT, "Expected {} threads, found {}", THREAD_COUNT, outputs.len());
            for i in 0..THREAD_COUNT {
                assert!(outputs.contains_key(&i), "Missing thread {}", i);
            }
            
            // Clean up
            progress.stop().await;
        }).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_progress_display_window_with_title() {
        timeout(TEST_TIMEOUT, async {
            clear_screen().await;
            let progress = ProgressDisplay::new().await;
            
            // Create multiple progress displays
            for i in 0..THREAD_COUNT {
                // Set title
                progress.set_title(format!("[test_progress_display_window_with_title] Thread {} Processing", i)).await;
                sleep(Duration::from_millis(100)).await;
                
                // Send multiple messages with emojis
                for _ in 0..MESSAGES_PER_THREAD {
                    let emoji = random_emoji();
                    let message = format!("[test_progress_display_window_with_title] [Thread {}] {} {}", i, emoji, random_message());
                    progress.capture_stdout(message).await;
                    sleep(Duration::from_millis(100)).await;
                }
                
                // Complete the jobs
                for j in 0..JOBS_PER_THREAD {
                    progress.update_progress(i, j + 1, JOBS_PER_THREAD, &format!("[test_progress_display_window_with_title] [Thread {}]", i)).await;
                    sleep(Duration::from_millis(200)).await;
                }
            }

            // Allow time for final updates
            sleep(Duration::from_millis(200)).await;
            
            // Clean up
            progress.stop().await;
        }).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_progress_display_limited_mode() {
        timeout(TEST_TIMEOUT, async {
            clear_screen().await;
            let progress = ProgressDisplay::new().await;
            
            // Create multiple progress displays
            for i in 0..THREAD_COUNT {
                // Send multiple messages with emojis
                for _ in 0..MESSAGES_PER_THREAD {
                    let emoji = random_emoji();
                    let message = format!("[test_progress_display_limited_mode] [Thread {}] {} {}", i, emoji, random_message());
                    progress.capture_stdout(message).await;
                    sleep(Duration::from_millis(100)).await;
                }
                
                // Complete the jobs
                for j in 0..JOBS_PER_THREAD {
                    progress.update_progress(i, j + 1, JOBS_PER_THREAD, &format!("[test_progress_display_limited_mode] [Thread {}]", i)).await;
                    sleep(Duration::from_millis(200)).await;
                }
            }

            // Allow time for final updates
            sleep(Duration::from_millis(200)).await;
            
            // Clean up
            progress.stop().await;
        }).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_progress_display_terminal_size_handling() {
        timeout(TEST_TIMEOUT, async {
            clear_screen().await;
            let progress = ProgressDisplay::new().await;
            
            // Set a small terminal size
            *progress.terminal_size.lock().await = (80, 3);
            
            // Create multiple progress displays
            for i in 0..THREAD_COUNT {
                // Send multiple messages with emojis
                for j in 0..2 {
                    let emoji = random_emoji();
                    let message = format!("[test_progress_display_terminal_size_handling] [Thread {}] {} Message {}", i, emoji, j);
                    progress.capture_stdout(message).await;
                    sleep(Duration::from_millis(100)).await;
                }
                
                // Complete the jobs
                for j in 0..2 {
                    progress.update_progress(i, j + 1, 2, &format!("[test_progress_display_terminal_size_handling] [Thread {}]", i)).await;
                    sleep(Duration::from_millis(200)).await;
                }
            }

            // Allow time for final updates
            sleep(Duration::from_millis(200)).await;
            
            // Verify output is constrained by terminal height
            let outputs = progress.outputs.lock().await;
            let mut total_lines = 0;
            for (_, lines) in outputs.iter() {
                total_lines += lines.len();
            }
            assert!(total_lines <= 3, "Total lines ({}) exceeds terminal height (3)", total_lines);
            
            // Clean up
            progress.stop().await;
        }).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_full_progress_workflow() {
        timeout(TEST_TIMEOUT, async {
            clear_screen().await;
            let progress = ProgressDisplay::new().await;
            
            // Update progress displays
            for i in 0..5 {
                for j in 0..THREAD_COUNT {
                    let prefix = match j {
                        0 => "[test_full_progress_workflow] tic - toc",
                        1 => "[test_full_progress_workflow] blah - gnu",
                        _ => &format!("[test_full_progress_workflow] Thread {} Processing", j),
                    };
                    progress.update_progress(j, i + 1, 5, prefix).await;
                    sleep(Duration::from_millis(100)).await;
                }
            }

            // Allow time for final updates
            sleep(Duration::from_millis(200)).await;
            
            // Verify final state
            let outputs = progress.outputs.lock().await;
            assert_eq!(outputs.len(), THREAD_COUNT);
            for (_, lines) in outputs.iter() {
                assert!(lines.len() <= 3);
                assert!(lines.last().unwrap().contains("100%"));
            }

            progress.stop().await;
        }).await.unwrap();
    }
}
