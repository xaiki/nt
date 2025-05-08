use crate::thread::{ThreadManager, ThreadState, TaskHandle};
use crate::Config;
use crate::ThreadMode;
use crate::io::OutputBuffer;
use tokio::time::sleep;
use std::time::Duration;
use anyhow::Result;
use tokio::sync::mpsc;
use crate::ThreadMessage;
use std::sync::Arc;

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

#[tokio::test]
async fn test_task_handle_pause_resume() -> Result<()> {
    // Create a message channel
    let (message_tx, _message_rx) = mpsc::channel::<ThreadMessage>(100);
    
    // Create a task handle with a mode that supports pause/resume
    let config = Config::new(ThreadMode::Window(3), 10)?;
    let task_handle = TaskHandle::new(1, config, message_tx.clone());
    
    // Check initial pause state
    assert_eq!(task_handle.is_paused().await?, false);
    
    // Test pausing
    task_handle.pause().await?;
    assert_eq!(task_handle.is_paused().await?, true);
    
    // Complete some jobs - should not increment while paused
    task_handle.set_progress(5).await?;
    assert_eq!(task_handle.get_completed_jobs().await?, 5);
    assert_eq!(task_handle.update_progress().await?, 50.0); // Should stay at 50% (5/10)
    assert_eq!(task_handle.get_completed_jobs().await?, 5); // Still 5 due to being paused
    
    // Test resuming
    task_handle.resume().await?;
    assert_eq!(task_handle.is_paused().await?, false);
    
    // Now progress updates should work
    assert_eq!(task_handle.update_progress().await?, 60.0); // 6/10 = 60%
    assert_eq!(task_handle.get_completed_jobs().await?, 6);
    
    Ok(())
}

#[tokio::test]
async fn test_progress_manager_pause_resume() -> Result<()> {
    // Create a message channel
    let (message_tx, _message_rx) = mpsc::channel::<ThreadMessage>(100);
    
    // Create a mode factory
    let factory = crate::modes::factory::ModeFactory::new();
    
    // Create a progress manager
    let manager = crate::progress_manager::ProgressManager::new(
        Arc::new(factory), 
        message_tx.clone()
    );
    
    // Create some tasks
    let task1 = manager.create_task(ThreadMode::Window(3), 10).await?;
    let task2 = manager.create_task(ThreadMode::Limited, 5).await?;
    let task3 = manager.create_task(ThreadMode::WindowWithTitle(2), 8).await?;
    
    // Verify initial states
    assert_eq!(manager.is_thread_paused(task1.thread_id()).await?, false);
    assert_eq!(manager.is_thread_paused(task2.thread_id()).await?, false);
    assert_eq!(manager.is_thread_paused(task3.thread_id()).await?, false);
    
    // Pause a single thread
    manager.pause_thread(task1.thread_id()).await?;
    assert_eq!(manager.is_thread_paused(task1.thread_id()).await?, true);
    assert_eq!(manager.is_thread_paused(task2.thread_id()).await?, false);
    assert_eq!(manager.is_thread_paused(task3.thread_id()).await?, false);
    
    // Resume the paused thread
    manager.resume_thread(task1.thread_id()).await?;
    assert_eq!(manager.is_thread_paused(task1.thread_id()).await?, false);
    
    // Pause all threads
    manager.pause_all().await?;
    assert_eq!(manager.is_thread_paused(task1.thread_id()).await?, true);
    assert_eq!(manager.is_thread_paused(task2.thread_id()).await?, true);
    assert_eq!(manager.is_thread_paused(task3.thread_id()).await?, true);
    
    // Test thread state - should all be in Paused state
    let thread_states = manager.thread_manager().get_threads_by_state(ThreadState::Paused).await;
    assert_eq!(thread_states.len(), 3);
    
    // Resume all threads
    manager.resume_all().await?;
    assert_eq!(manager.is_thread_paused(task1.thread_id()).await?, false);
    assert_eq!(manager.is_thread_paused(task2.thread_id()).await?, false);
    assert_eq!(manager.is_thread_paused(task3.thread_id()).await?, false);
    
    // Test thread state - should all be in Running state
    let thread_states = manager.thread_manager().get_threads_by_state(ThreadState::Running).await;
    assert_eq!(thread_states.len(), 3);
    
    Ok(())
} 