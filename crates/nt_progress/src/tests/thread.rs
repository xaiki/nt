use crate::thread::{ThreadManager, ThreadState, TaskHandle};
use crate::modes::Config;
use crate::modes::ThreadMode;
use crate::io::OutputBuffer;
use tokio::time::sleep;
use std::time::Duration;
use anyhow::Result;
use tokio::sync::mpsc;
use crate::ThreadMessage;

#[tokio::test]
async fn test_thread_pool_basic() -> Result<()> {
    let manager = ThreadManager::with_thread_limit(3);
    
    // Create a message channel for task handles
    let (message_tx, _message_rx) = mpsc::channel::<ThreadMessage>(100);
    
    // Create 5 threads (should be limited to 3 at a time)
    let mut handles = Vec::new();
    for i in 0..5 {
        let thread_id = manager.next_thread_id();
        let config = Config::new(ThreadMode::Limited, 1)?;
        let task_handle = crate::thread::TaskHandle::new(thread_id, config, message_tx.clone());
        let join_handle = tokio::spawn(async move {
            sleep(Duration::from_millis(100)).await;
            Ok(())
        });
        
        manager.register_thread(thread_id, task_handle, join_handle).await;
        handles.push(thread_id);
    }
    
    // Verify we never exceed the thread limit
    assert!(manager.thread_count().await <= 3);
    
    // Wait for all threads to complete
    manager.join_all().await?;
    
    // Verify all threads are cleaned up
    manager.cleanup_completed().await?;
    assert_eq!(manager.thread_count().await, 0);
    
    Ok(())
}

#[tokio::test]
async fn test_thread_state_tracking() -> Result<()> {
    let manager = ThreadManager::new();
    
    // Create a message channel for task handles
    let (message_tx, _message_rx) = mpsc::channel::<ThreadMessage>(100);
    
    // Create threads in different states
    let running_id = manager.next_thread_id();
    let config = Config::new(ThreadMode::Limited, 1)?;
    let task_handle = crate::thread::TaskHandle::new(running_id, config, message_tx.clone());
    let join_handle = tokio::spawn(async move {
        sleep(Duration::from_millis(100)).await;
        Ok(())
    });
    manager.register_thread(running_id, task_handle, join_handle).await;
    
    let paused_id = manager.next_thread_id();
    let config = Config::new(ThreadMode::Limited, 1)?;
    let task_handle = crate::thread::TaskHandle::new(paused_id, config, message_tx.clone());
    let join_handle = tokio::spawn(async move { Ok(()) });
    manager.register_thread(paused_id, task_handle, join_handle).await;
    manager.update_thread_state(paused_id, ThreadState::Paused).await?;
    
    let failed_id = manager.next_thread_id();
    let config = Config::new(ThreadMode::Limited, 1)?;
    let task_handle = crate::thread::TaskHandle::new(failed_id, config, message_tx.clone());
    let join_handle = tokio::spawn(async move { Ok(()) });
    manager.register_thread(failed_id, task_handle, join_handle).await;
    manager.update_thread_state(failed_id, ThreadState::Failed("Test failure".to_string())).await?;
    
    // Verify state counts
    assert_eq!(manager.count_threads_by_state(ThreadState::Running).await, 1);
    assert_eq!(manager.count_threads_by_state(ThreadState::Paused).await, 1);
    assert_eq!(manager.count_threads_by_state(ThreadState::Failed("Test failure".to_string())).await, 1);
    
    // Verify thread lists by state
    let running = manager.get_threads_by_state(ThreadState::Running).await;
    assert_eq!(running.len(), 1);
    assert_eq!(running[0], running_id);
    
    let paused = manager.get_threads_by_state(ThreadState::Paused).await;
    assert_eq!(paused.len(), 1);
    assert_eq!(paused[0], paused_id);
    
    // Clean up
    manager.cancel_all().await?;
    Ok(())
}

#[tokio::test]
async fn test_thread_cleanup() -> Result<()> {
    let manager = ThreadManager::new();
    
    // Create a message channel for task handles
    let (message_tx, _message_rx) = mpsc::channel::<ThreadMessage>(100);
    
    // Create some threads that will complete quickly
    for i in 0..3 {
        let thread_id = manager.next_thread_id();
        let config = Config::new(ThreadMode::Limited, 1)?;
        let task_handle = crate::thread::TaskHandle::new(thread_id, config, message_tx.clone());
        let join_handle = tokio::spawn(async move {
            sleep(Duration::from_millis(50 * (i + 1) as u64)).await;
            Ok(())
        });
        manager.register_thread(thread_id, task_handle, join_handle).await;
    }
    
    // Wait for threads to complete
    sleep(Duration::from_millis(200)).await;
    
    // Clean up completed threads
    manager.cleanup_completed().await?;
    
    // Verify all threads are cleaned up
    assert_eq!(manager.thread_count().await, 0);
    
    Ok(())
}

#[tokio::test]
async fn test_thread_limit_adjustment() -> Result<()> {
    let manager = ThreadManager::with_thread_limit(2);
    
    // Create a message channel for task handles
    let (message_tx, _message_rx) = mpsc::channel::<ThreadMessage>(100);
    
    // Create 4 threads (should be limited to 2 at a time)
    let mut handles = Vec::new();
    for i in 0..4 {
        let thread_id = manager.next_thread_id();
        let config = Config::new(ThreadMode::Limited, 1)?;
        let task_handle = crate::thread::TaskHandle::new(thread_id, config, message_tx.clone());
        let join_handle = tokio::spawn(async move {
            sleep(Duration::from_millis(100)).await;
            Ok(())
        });
        
        manager.register_thread(thread_id, task_handle, join_handle).await;
        handles.push(thread_id);
    }
    
    // Verify we never exceed the initial thread limit
    assert!(manager.thread_count().await <= 2);
    
    // Increase the thread limit
    manager.set_thread_limit(4);
    assert_eq!(manager.get_thread_limit(), 4);
    
    // Create more threads
    for i in 0..2 {
        let thread_id = manager.next_thread_id();
        let config = Config::new(ThreadMode::Limited, 1)?;
        let task_handle = crate::thread::TaskHandle::new(thread_id, config, message_tx.clone());
        let join_handle = tokio::spawn(async move {
            sleep(Duration::from_millis(100)).await;
            Ok(())
        });
        
        manager.register_thread(thread_id, task_handle, join_handle).await;
        handles.push(thread_id);
    }
    
    // Verify we can now have up to 4 threads
    assert!(manager.thread_count().await <= 4);
    
    // Clean up
    manager.join_all().await?;
    Ok(())
}

#[tokio::test]
async fn test_task_handle_io() -> Result<()> {
    let manager = ThreadManager::new();
    
    // Create a message channel for task handles
    let (message_tx, _message_rx) = mpsc::channel::<ThreadMessage>(100);
    
    let thread_id = manager.next_thread_id();
    let config = Config::new(ThreadMode::Limited, 1)?;
    let mut task_handle = TaskHandle::new(thread_id, config, message_tx.clone());
    
    // Test writing lines
    task_handle.write_line("test line 1").await?;
    task_handle.write_line("test line 2").await?;
    
    // Test writing raw bytes
    task_handle.write(b"raw bytes").await?;
    
    // Test stdout capture
    task_handle.capture_stdout("stdout line".to_string()).await?;
    
    // Test stderr capture
    task_handle.capture_stderr("stderr line".to_string()).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_task_handle_output_buffer() -> Result<()> {
    let manager = ThreadManager::new();
    
    // Create a message channel for task handles
    let (message_tx, _message_rx) = mpsc::channel::<ThreadMessage>(100);
    
    let thread_id = manager.next_thread_id();
    let config = Config::new(ThreadMode::Limited, 1)?;
    let mut task_handle = TaskHandle::new(thread_id, config, message_tx.clone());
    
    // Write multiple lines to test buffer management
    for i in 0..150 {
        task_handle.write_line(&format!("line {}", i)).await?;
    }
    
    Ok(())
} 