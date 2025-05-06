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
use crate::modes::Config;

mod modes;
pub mod test_utils;
#[cfg(test)]
pub mod tests;

pub use modes::ThreadMode;

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

#[derive(Clone)]
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
    thread_configs: Arc<Mutex<HashMap<usize, Config>>>,
    writer: Arc<Mutex<Box<dyn Write + Send + 'static>>>,
}

impl Debug for ProgressDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProgressDisplay")
            .field("outputs", &self.outputs)
            .field("spinner_index", &self.spinner_index)
            .field("terminal_size", &self.terminal_size)
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
        let display = Self {
            outputs: Arc::new(Mutex::new(HashMap::new())),
            spinner_index: Arc::new(AtomicUsize::new(0)),
            terminal_size: Arc::new(Mutex::new((80, 24))),
            processing_task: Arc::new(Mutex::new(None)),
            thread_handles: Arc::new(Mutex::new(HashMap::new())),
            next_thread_id: Arc::new(AtomicUsize::new(0)),
            running: Arc::new(AtomicBool::new(true)),
            message_tx,
            message_rx: Arc::new(Mutex::new(message_rx)),
            thread_configs: Arc::new(Mutex::new(HashMap::new())),
            writer: Arc::new(Mutex::new(Box::new(writer))),
        };

        // Start the display thread
        let processing_task = tokio::spawn(Self::start_display_thread(
            Arc::clone(&display.outputs),
            Arc::clone(&display.terminal_size),
            Arc::clone(&display.spinner_index),
            Arc::clone(&display.message_rx),
            Arc::clone(&display.running),
            mode,
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
        let config = Config::new(mode, 1).map_err(|e| anyhow!(e))?;
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
            &self.terminal_size,
            &self.spinner_index,
            &self.writer,
        ).await
    }

    pub async fn stop(&self) -> Result<()> {
        self.running.store(false, Ordering::SeqCst);
        self.join_all().await?;
        
        if let Some(handle) = self.processing_task.lock().await.take() {
            handle.await.map_err(|e| anyhow!("Failed to join processing task: {}", e))?;
        }
        Ok(())
    }

    pub async fn set_title(&self, _title: String) -> Result<()> {
        unimplemented!("Title support not yet implemented");
    }

    pub async fn add_emoji(&self, _thread_id: usize, _emoji: &str) -> Result<()> {
        unimplemented!("Emoji support not yet implemented");
    }

    pub async fn set_total_jobs(&self, _total: usize) -> Result<()> {
        unimplemented!("Total jobs support not yet implemented");
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
        terminal_size: Arc<Mutex<(u16, u16)>>,
        spinner_index: Arc<AtomicUsize>,
        message_rx: Arc<Mutex<mpsc::Receiver<ThreadMessage>>>,
        running: Arc<AtomicBool>,
        mode: ThreadMode,
        writer: Arc<Mutex<Box<dyn Write + Send + 'static>>>,
    ) {
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
                &terminal_size,
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
        terminal_size: &Arc<Mutex<(u16, u16)>>,
        spinner_index: &Arc<AtomicUsize>,
        writer: &Arc<Mutex<Box<dyn Write + Send + 'static>>>,
    ) -> std::io::Result<()> {
        // Gather all data under their respective locks first
        let (thread_ids, lines_by_thread, line_count, terminal_height) = {
            let outputs_guard = outputs.lock().await;
            if outputs_guard.is_empty() {
                return Ok(());
            }
            
            let (_width, height) = *terminal_size.lock().await;
            
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
        
        if line_count == 0 {
            return Ok(());
        }

        // Adjust line count if needed based on terminal height
        let mut adjusted_lines_by_thread = lines_by_thread.clone();
        if line_count > terminal_height as usize && !adjusted_lines_by_thread.is_empty() {
            let available_lines = terminal_height as usize;
            let lines_per_thread = available_lines / adjusted_lines_by_thread.len();
            
            // Adjust each thread's window size
            for (_, lines) in adjusted_lines_by_thread.iter_mut() {
                if lines.len() > lines_per_thread {
                    lines.drain(0..lines.len() - lines_per_thread);
                }
            }
        }
        
        // Get the spinner index
        let spinner_chars = ["▏▎▍", "▎▍▌", "▍▌▋", "▌▋▊", "▋▊▉", "▊▉█", "▉█▉", "█▉▊", "▉▊▋", "▊▋▌", "▋▌▍", "▌▍▎", "▍▎▏"];
        let spinner_index_value = {
            let idx = spinner_index.fetch_add(1, Ordering::SeqCst);
            (idx + 1) % spinner_chars.len()
        };
        
        // Prepare all lines first
        let mut output = String::new();
        output.push_str(&format!("\x1B[{}A", line_count));
        
        // Collect all formatted lines first
        let mut all_lines = Vec::new();
        for thread_id in thread_ids {
            if let Some(lines) = adjusted_lines_by_thread.get(&thread_id) {
                for line in lines {
                    all_lines.push(format!("{} {}", spinner_chars[spinner_index_value], line));
                }
            }
        }
        
        // Join all lines with newlines and write in one operation
        let complete_output = all_lines.join("\n");
        if !complete_output.is_empty() {
            output.push_str(&complete_output);
            output.push('\n');
        }
        
        // Now do all the writing at once without any await points in between
        let mut writer = writer.lock().await;
        write!(writer, "{}", output)?;
        writer.flush()?;
        Ok(())
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
}
