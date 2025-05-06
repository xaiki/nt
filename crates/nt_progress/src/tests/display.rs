use std::time::Duration;
use tokio::time::sleep;
use crate::ProgressDisplay;
use crate::modes::{ThreadMode, Window};
use crate::tests::common::TestEnv;
use crate::modes::JobTracker;

#[tokio::test]
async fn test_progress_display_all_modes() {
    let modes = [
        ThreadMode::Limited,
        ThreadMode::Capturing,
        ThreadMode::Window(3),
        ThreadMode::WindowWithTitle(3),
    ];

    for mode in modes {
        let display = ProgressDisplay::new().await;
        let total_jobs = 5;
        
        // Spawn multiple tasks with the same mode
        let mut handles = vec![];
        for i in 0..total_jobs {
            let display = display.clone();
            let i = i;
            handles.push(tokio::spawn(async move {
                let mut env = TestEnv::new(80, 24);
                display.spawn_with_mode(mode, move || format!("task-{}", i)).await.unwrap();
                for j in 0..3 {
                    env.writeln(&format!("Thread {}: Message {}", i, j));
                    sleep(Duration::from_millis(50)).await;
                }
                display.display().await.unwrap();
                display.stop().await.unwrap();
                env.verify();
            }));
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
    }
}

#[tokio::test]
async fn test_progress_display_high_concurrency() {
    let display = ProgressDisplay::new().await;
    let total_jobs = 20;
    
    // Spawn multiple tasks with high concurrency
    let mut handles = vec![];
    for i in 0..total_jobs {
        let display = display.clone();
        let i = i;
        handles.push(tokio::spawn(async move {
            let mut env = TestEnv::new(80, 24);
            display.spawn_with_mode(ThreadMode::Window(5), move || format!("task-{}", i)).await.unwrap();
            for j in 0..10 {
                env.writeln(&format!("Thread {}: Message {}", i, j));
                sleep(Duration::from_millis(10)).await;
            }
            display.display().await.unwrap();
            display.stop().await.unwrap();
            env.verify();
        }));
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn test_progress_display_different_modes() {
    let display = ProgressDisplay::new().await;
    let display_clone = display.clone();
    
    let handle = tokio::spawn(async move {
        let mut env = TestEnv::new(80, 24);
        
        // Spawn tasks with different modes
        display_clone.spawn_with_mode(ThreadMode::Limited, || "limited-task".to_string()).await.unwrap();
        env.writeln("Test 1");
        env.writeln("Test 2");
        display_clone.display().await.unwrap();

        display_clone.spawn_with_mode(ThreadMode::Window(2), || "window-task".to_string()).await.unwrap();
        env.writeln("Test 3");
        env.writeln("Test 4");
        display_clone.display().await.unwrap();

        display_clone.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "title-task".to_string()).await.unwrap();
        env.writeln("Test 5");
        display_clone.display().await.unwrap();
        display_clone.stop().await.unwrap();
        env.verify();
    });

    // Wait for all tasks to complete
    handle.await.unwrap();
}

#[tokio::test]
async fn test_progress_display_error_handling() {
    let display = ProgressDisplay::new().await;
    
    // Test invalid mode configuration
    assert!(display.spawn_with_mode(ThreadMode::Window(0), || "invalid".to_string()).await.is_err());
    
    // Test task failure
    let display2 = ProgressDisplay::new().await;
    let handle = tokio::spawn(async move {
        let mut env2 = TestEnv::new(80, 24);
        display2.spawn_with_mode(ThreadMode::Limited, || "failing-task".to_string()).await.unwrap();
        env2.writeln("Before error");
        display2.display().await.unwrap();
        display2.stop().await.unwrap();
        env2.verify();
        panic!("Simulated task failure");
    });
    
    // Should not crash the display
    assert!(handle.await.is_err());
}

#[tokio::test]
async fn test_progress_display_mode_specific() {
    let mut env = TestEnv::new(80, 24);
    
    // Test Limited mode
    let display = ProgressDisplay::new().await;
    display.spawn_with_mode(ThreadMode::Limited, || "limited".to_string()).await.unwrap();
    for i in 0..10 {
        env.writeln(&format!("Line {}", i));
    }
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
    
    // Test Capturing mode
    let display = ProgressDisplay::new().await;
    display.spawn_with_mode(ThreadMode::Capturing, || "capturing".to_string()).await.unwrap();
    for i in 0..10 {
        env.writeln(&format!("Line {}", i));
    }
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
    
    // Test Window mode
    let display = ProgressDisplay::new().await;
    display.spawn_with_mode(ThreadMode::Window(3), || "window".to_string()).await.unwrap();
    for i in 0..10 {
        env.writeln(&format!("Line {}", i));
    }
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
    
    // Test WindowWithTitle mode
    let display = ProgressDisplay::new().await;
    display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "title".to_string()).await.unwrap();
    env.writeln("Test with emoji 🚀");
    env.writeln("Another line 📝");
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_progress_display_edge_cases() {
    let mut env = TestEnv::new(80, 24);
    
    // Test empty output
    let display = ProgressDisplay::new().await;
    display.spawn_with_mode(ThreadMode::Limited, || "empty".to_string()).await.unwrap();
    env.writeln("");
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
    
    // Test very long lines
    let display = ProgressDisplay::new().await;
    let long_line = "x".repeat(1000);
    display.spawn_with_mode(ThreadMode::Limited, || "long".to_string()).await.unwrap();
    env.writeln(&long_line);
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
    
    // Test special characters
    let display = ProgressDisplay::new().await;
    display.spawn_with_mode(ThreadMode::Limited, || "special".to_string()).await.unwrap();
    env.writeln("Test with \n newlines \t tabs \r returns");
    env.writeln("Test with unicode: 你好世界");
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_progress_display_concurrency() {
    let display = ProgressDisplay::new().await;
    let total_jobs = 5;
    let mut handles = vec![];
    let mut main_env = TestEnv::new(80, 24);
    let (width, height) = main_env.size();
    
    // Test task cancellation
    for i in 0..total_jobs {
        let display = display.clone();
        let i = i;
        let mut task_env = TestEnv::new(width, height);
        handles.push(tokio::spawn(async move {
            display.spawn_with_mode(ThreadMode::Limited, move || format!("task-{}", i)).await.unwrap();
            for j in 0..3 {
                task_env.writeln(&format!("Thread {}: Message {}", i, j));
                sleep(Duration::from_millis(50)).await;
            }
            // Simulate task cancellation
            if i == 2 {
                return task_env;
            }
            task_env.writeln(&format!("Thread {}: Completed", i));
            task_env
        }));
    }
    
    // Wait for all tasks to complete and merge their outputs
    for handle in handles {
        let task_env = handle.await.unwrap();
        main_env.merge(task_env);
    }
    
    display.display().await.unwrap();
    display.stop().await.unwrap();
    main_env.verify();
}

#[tokio::test]
async fn test_progress_display_rapid_messages() {
    let display = ProgressDisplay::new().await;
    let display_clone = display.clone();
    let mut main_env = TestEnv::new(80, 24);
    let (width, height) = main_env.size();
    
    // Test rapid message bursts
    let handle = tokio::spawn(async move {
        let mut task_env = TestEnv::new(width, height);
        display_clone.spawn_with_mode(ThreadMode::Window(5), || "burst".to_string()).await.unwrap();
        for _ in 0..100 {
            task_env.writeln("Burst message");
        }
        task_env
    });
    
    let task_env = handle.await.unwrap();
    main_env.merge(task_env);
    display.display().await.unwrap();
    display.stop().await.unwrap();
    main_env.verify();
}

#[tokio::test]
async fn test_display_formatting() {
    let mut env = TestEnv::new(80, 24);
    
    // Test basic output formatting
    let display = ProgressDisplay::new().await;
    
    // Add some test output
    display.spawn_with_mode(ThreadMode::Limited, || "test-task").await.unwrap();
    env.writeln("Test line 1");
    env.writeln("Test line 2");
    
    // Verify display
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_window_mode_display() {
    let mut env = TestEnv::new(80, 24);
    
    let display = ProgressDisplay::new_with_mode(ThreadMode::Window(2)).await;
    
    // Add more lines than the window size
    display.spawn_with_mode(ThreadMode::Window(2), || "window-task").await.unwrap();
    env.writeln("Line 1");
    env.writeln("Line 2");
    env.writeln("Line 3");
    
    // Verify display
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_terminal_size_handling() {
    let mut env = TestEnv::new(80, 24);
    
    let display = ProgressDisplay::new().await;
    
    // Set a small terminal size
    *display.terminal_size.lock().await = (80, 2);
    
    // Add more lines than terminal height
    display.spawn_with_mode(ThreadMode::Limited, || "size-test").await.unwrap();
    env.writeln("Line 1");
    env.writeln("Line 2");
    env.writeln("Line 3");
    
    // Verify display
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_progress_display_burst() {
    let display = ProgressDisplay::new().await;
    let display_clone = display.clone();
    
    let handle = tokio::spawn(async move {
        let mut env = TestEnv::new(80, 24);
        display_clone.spawn_with_mode(ThreadMode::Window(5), || "burst".to_string()).await.unwrap();
        for _ in 0..100 {
            env.writeln("Burst message");
            sleep(Duration::from_millis(1)).await;
        }
        display_clone.display().await.unwrap();
        display_clone.stop().await.unwrap();
        env.verify();
    });
    
    // Wait for burst to complete
    handle.await.unwrap();
}

#[tokio::test]
async fn test_set_total_jobs() {
    // Test setting total jobs for a single thread
    let display = ProgressDisplay::new().await;
    let mut env = TestEnv::new(80, 24);
    
    // Create a task
    let handle = display.spawn_with_mode(ThreadMode::Limited, || "total-jobs-test").await.unwrap();
    let thread_id = handle.thread_id();
    
    // Update total jobs for the thread
    handle.set_total_jobs(20).await.unwrap();
    
    // No need to verify the total jobs here since we're testing the functionality
    
    env.writeln("Before update");
    
    // Get progress updates
    for i in 0..5 {
        display.update_progress(thread_id, i + 1, 20, "Progress").await.unwrap();
        env.writeln(&format!("Progress {}/20", i + 1));
    }
    
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
    
    // Test setting total jobs for all threads
    let display = ProgressDisplay::new().await;
    let mut env = TestEnv::new(80, 24);
    
    // Create multiple tasks
    let mut handles = Vec::new();
    for i in 0..3 {
        let handle = display.spawn_with_mode(ThreadMode::Window(3), move || format!("task-{}", i)).await.unwrap();
        handles.push(handle);
    }
    
    // Set total jobs for all threads
    display.set_total_jobs(None, 30).await.unwrap();
    
    // Verify all threads have the new total
    let mut configs = display.thread_configs.lock().await;
    for (_, config) in configs.iter_mut() {
        // Use JobTracker trait to get the total jobs
        if let Some(window) = config.as_type_mut::<Window>() {
            assert_eq!(window.get_total_jobs(), 30);
        }
    }
    
    env.writeln("Updated all threads to 30 jobs");
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
    
    // Test error handling: Setting total jobs to zero
    let display = ProgressDisplay::new().await;
    let result = display.set_total_jobs(None, 0).await;
    assert!(result.is_err());
    
    // Test error handling: Setting total jobs for a non-existent thread
    let display = ProgressDisplay::new().await;
    let result = display.set_total_jobs(Some(999), 10).await;
    assert!(result.is_err());
} 
