use std::time::Duration;
use tokio::time::sleep;
use crate::ProgressDisplay;
use crate::modes::{ThreadMode, Window};
use crate::terminal::TestEnv;
use crate::modes::JobTracker;
use crate::tests::common::with_timeout;

/**
 * IMPORTANT: Testing Pattern to Prevent Test Hangs
 * 
 * ProgressDisplay tests MUST follow this pattern to avoid hanging:
 * 
 * 1. Create ProgressDisplay OUTSIDE the timeout block
 * 2. Run test logic INSIDE the timeout block 
 * 3. Call display.stop() OUTSIDE the timeout block (to ensure cleanup even if timeout occurs)
 */

#[tokio::test]
async fn test_progress_display_high_concurrency() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await;
    
    // Run test within timeout
    with_timeout(async {
        let total_jobs = 20;
        
        // Spawn multiple tasks with high concurrency
        let mut handles = vec![];
        for i in 0..total_jobs {
            let task_display = display.clone();
            let i = i;
            handles.push(tokio::spawn(async move {
                let mut env = TestEnv::new(80, 24);
                task_display.spawn_with_mode(ThreadMode::Window(5), move || format!("task-{}", i)).await.unwrap();
                for j in 0..10 {
                    env.writeln(&format!("Thread {}: Message {}", i, j));
                    sleep(Duration::from_millis(10)).await;
                }
                task_display.display().await.unwrap();
                env.verify();
            }));
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
    }, 10).await.unwrap();
    
    // Always clean up outside timeout
    display.stop().await.unwrap();
}

#[tokio::test]
async fn test_progress_display_different_modes() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await;
    let mut env = TestEnv::new(80, 24);
    
    // Run test within timeout
    with_timeout(async {
        // Spawn tasks with different modes
        display.spawn_with_mode(ThreadMode::Limited, || "limited-task".to_string()).await.unwrap();
        env.writeln("Test 1");
        env.writeln("Test 2");
        display.display().await.unwrap();

        display.spawn_with_mode(ThreadMode::Window(2), || "window-task".to_string()).await.unwrap();
        env.writeln("Test 3");
        env.writeln("Test 4");
        display.display().await.unwrap();

        display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "title-task".to_string()).await.unwrap();
        env.writeln("Test 5");
        display.display().await.unwrap();
    }, 5).await.unwrap();
    
    // Always clean up outside timeout
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_progress_display_error_handling() {
    // Always clean up resources even if the test panics
    let display = ProgressDisplay::new().await;
    
    // Enable error propagation for this test
    crate::modes::set_error_propagation(true);
    
    // Run the test
    with_timeout(async {
        // Test invalid mode configuration
        let result = display.spawn_with_mode(ThreadMode::Window(0), || "invalid".to_string()).await;
        assert!(result.is_err());
    }, 3).await.unwrap();
    
    // Essential: always stop the display to clean up event handlers
    display.stop().await.unwrap();
    
    // Reset error propagation
    crate::modes::set_error_propagation(false);
}

#[tokio::test]
async fn test_progress_display_limited_mode() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await;
    let mut env = TestEnv::new(80, 24);
    
    // Test within timeout
    with_timeout(async {
        // Test Limited mode
        display.spawn_with_mode(ThreadMode::Limited, || "limited".to_string()).await.unwrap();
        for i in 0..10 {
            env.writeln(&format!("Line {}", i));
        }
        display.display().await.unwrap();
    }, 3).await.unwrap();
    
    // Always clean up outside timeout
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_progress_display_capturing_mode() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await;
    let mut env = TestEnv::new(80, 24);
    
    // Test within timeout
    with_timeout(async {
        // Test Capturing mode
        display.spawn_with_mode(ThreadMode::Capturing, || "capturing".to_string()).await.unwrap();
        for i in 0..10 {
            env.writeln(&format!("Line {}", i));
        }
        display.display().await.unwrap();
    }, 3).await.unwrap();
    
    // Always clean up outside timeout
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_progress_display_window_mode() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await;
    let mut env = TestEnv::new(80, 24);
    
    // Test within timeout
    with_timeout(async {
        // Test Window mode
        display.spawn_with_mode(ThreadMode::Window(3), || "window".to_string()).await.unwrap();
        for i in 0..10 {
            env.writeln(&format!("Line {}", i));
        }
        display.display().await.unwrap();
    }, 3).await.unwrap();
    
    // Always clean up outside timeout
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_progress_display_window_with_title_mode() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await;
    let mut env = TestEnv::new(80, 24);
    
    // Test within timeout
    with_timeout(async {
        // Test WindowWithTitle mode
        display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "title".to_string()).await.unwrap();
        env.writeln("Test with emoji 🚀");
        env.writeln("Another line 📝");
        display.display().await.unwrap();
    }, 3).await.unwrap();
    
    // Always clean up outside timeout
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_progress_display_empty_output() {
    // Create the display outside the timeout
    let display = ProgressDisplay::new().await;
    let mut env = TestEnv::new(80, 24);
    
    // Test within timeout
    with_timeout(async {
        // Test empty output
        display.spawn_with_mode(ThreadMode::Limited, || "empty".to_string()).await.unwrap();
        env.writeln("");
        display.display().await.unwrap();
    }, 3).await.unwrap();
    
    // Essential: always stop outside the timeout to clean up
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_progress_display_long_lines() {
    // Create the display outside the timeout
    let display = ProgressDisplay::new().await;
    let mut env = TestEnv::new(80, 24);
    
    // Test within timeout
    with_timeout(async {
        // Test very long lines
        let long_line = "x".repeat(1000);
        display.spawn_with_mode(ThreadMode::Limited, || "long".to_string()).await.unwrap();
        env.writeln(&long_line);
        display.display().await.unwrap();
    }, 3).await.unwrap();
    
    // Essential: always stop outside the timeout to clean up
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_progress_display_special_chars() {
    // Create the display outside the timeout
    let display = ProgressDisplay::new().await;
    let mut env = TestEnv::new(80, 24);
    
    // Test within timeout
    with_timeout(async {
        // Test special characters
        display.spawn_with_mode(ThreadMode::Limited, || "special".to_string()).await.unwrap();
        env.writeln("Special characters: !@#$%^&*()_+{}|:<>?~`-=[]\\;',./");
        display.display().await.unwrap();
    }, 3).await.unwrap();
    
    // Essential: always stop outside the timeout to clean up
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_progress_display_concurrency() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await;
    let mut main_env = TestEnv::new(80, 24);
    let (width, height) = main_env.size();
    
    // Run test within timeout
    with_timeout(async {
        let total_jobs = 5;
        let mut handles = vec![];
        
        // Test task cancellation
        for i in 0..total_jobs {
            let task_display = display.clone();
            let i = i;
            let mut task_env = TestEnv::new(width, height);
            handles.push(tokio::spawn(async move {
                task_display.spawn_with_mode(ThreadMode::Limited, move || format!("task-{}", i)).await.unwrap();
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
    }, 5).await.unwrap();
    
    // Always clean up outside timeout
    display.stop().await.unwrap();
    main_env.verify();
}

#[tokio::test]
async fn test_progress_display_rapid_messages() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await;
    let mut main_env = TestEnv::new(80, 24);
    let (width, height) = main_env.size();
    
    // Run test within timeout
    with_timeout(async {
        // Test rapid message bursts
        let handle = tokio::spawn({
            let task_display = display.clone();
            async move {
                let mut task_env = TestEnv::new(width, height);
                task_display.spawn_with_mode(ThreadMode::Window(5), || "burst".to_string()).await.unwrap();
                for _ in 0..100 {
                    task_env.writeln("Burst message");
                }
                task_env
            }
        });
        
        let task_env = handle.await.unwrap();
        main_env.merge(task_env);
        display.display().await.unwrap();
    }, 5).await.unwrap();
    
    // Always clean up outside timeout
    display.stop().await.unwrap();
    main_env.verify();
}

#[tokio::test]
async fn test_display_formatting() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await;
    let mut env = TestEnv::new(80, 24);
    
    // Run test within timeout
    with_timeout(async {
        // Test basic output formatting
        // Add some test output
        display.spawn_with_mode(ThreadMode::Limited, || "test-task").await.unwrap();
        env.writeln("Test line 1");
        env.writeln("Test line 2");
        
        // Verify display
        display.display().await.unwrap();
    }, 3).await.unwrap();
    
    // Always clean up outside timeout
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_window_mode_display() {
    // Create display outside timeout
    let display = ProgressDisplay::new_with_mode(ThreadMode::Window(2)).await;
    let mut env = TestEnv::new(80, 24);
    
    // Run test within timeout
    with_timeout(async {
        // Add more lines than the window size
        display.spawn_with_mode(ThreadMode::Window(2), || "window-task").await.unwrap();
        env.writeln("Line 1");
        env.writeln("Line 2");
        env.writeln("Line 3");
        
        // Verify display
        display.display().await.unwrap();
    }, 3).await.unwrap();
    
    // Always clean up outside timeout
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_terminal_size_handling() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await;
    let mut env = TestEnv::new(80, 24);
    
    // Run test within timeout
    with_timeout(async {
        // Set a small terminal size
        display.terminal.set_size(80, 2).await.expect("Failed to set terminal size");
        
        // Add more lines than terminal height
        display.spawn_with_mode(ThreadMode::Limited, || "size-test").await.unwrap();
        env.writeln("Line 1");
        env.writeln("Line 2");
        env.writeln("Line 3");
        
        // Verify display
        display.display().await.unwrap();
    }, 3).await.unwrap();
    
    // Always clean up outside timeout
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_progress_display_burst() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await;
    let mut env = TestEnv::new(80, 24);
    
    // Run test within timeout
    with_timeout(async {
        display.spawn_with_mode(ThreadMode::Window(5), || "burst".to_string()).await.unwrap();
        for _ in 0..100 {
            env.writeln("Burst message");
            sleep(Duration::from_millis(1)).await;
        }
        display.display().await.unwrap();
    }, 5).await.unwrap();
    
    // Always clean up outside timeout
    display.stop().await.unwrap();
    env.verify();
}

// Split into separate tests for safety
#[tokio::test]
async fn test_set_total_jobs_single_thread() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await;
    let mut env = TestEnv::new(80, 24);
    
    // Run test within timeout
    with_timeout(async {
        // Create a task
        let handle = display.spawn_with_mode(ThreadMode::Limited, || "total-jobs-test").await.unwrap();
        let thread_id = handle.thread_id();
        
        // Update total jobs for the thread
        handle.set_total_jobs(20).await.unwrap();
        
        env.writeln("Before update");
        
        // Get progress updates
        for i in 0..5 {
            display.update_progress(thread_id, i + 1, 20, "Progress").await.unwrap();
            env.writeln(&format!("Progress {}/20", i + 1));
        }
        
        display.display().await.unwrap();
    }, 5).await.unwrap();
    
    // Always clean up outside timeout
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_set_total_jobs_all_threads() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await;
    let mut env = TestEnv::new(80, 24);
    
    // Run test within timeout
    with_timeout(async {
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
        drop(configs); // Explicitly drop the mutex guard
        
        env.writeln("Updated all threads to 30 jobs");
        display.display().await.unwrap();
    }, 5).await.unwrap();
    
    // Always clean up outside timeout
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_set_total_jobs_error_zero() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await;
    
    // Run test within timeout
    with_timeout(async {
        // Test error handling: Setting total jobs to zero
        let result = display.set_total_jobs(None, 0).await;
        assert!(result.is_err());
    }, 3).await.unwrap();
    
    // Always clean up outside timeout
    display.stop().await.unwrap();
}

#[tokio::test]
async fn test_set_total_jobs_error_nonexistent() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await;
    
    // Run test within timeout
    with_timeout(async {
        // Test error handling: Setting total jobs for a non-existent thread
        let result = display.set_total_jobs(Some(999), 10).await;
        assert!(result.is_err());
    }, 3).await.unwrap();
    
    // Always clean up outside timeout
    display.stop().await.unwrap();
}

#[tokio::test]
async fn test_progress_display_limited_mode_concurrent() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await;
    
    // Run test within timeout
    with_timeout(async {
        let total_jobs = 5;
        
        // Spawn multiple tasks with Limited mode
        let mut handles = vec![];
        for i in 0..total_jobs {
            let task_display = display.clone();
            let i = i;
            handles.push(tokio::spawn(async move {
                task_display.spawn_with_mode(ThreadMode::Limited, move || format!("task-{}", i)).await.unwrap();
                for j in 0..3 {
                    // Just log messages, no TestEnv inside spawned tasks
                    let message = format!("Thread {}: Message {}", i, j);
                    // Brief sleep to allow interleaving
                    sleep(Duration::from_millis(10)).await;
                }
            }));
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Display after all tasks completed
        display.display().await.unwrap();
    }, 5).await.unwrap();
    
    // Always stop the display outside the timeout
    display.stop().await.unwrap();
}

#[tokio::test]
async fn test_progress_display_capturing_mode_concurrent() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await;
    
    // Run test within timeout
    with_timeout(async {
        let total_jobs = 5;
        
        // Spawn multiple tasks with Capturing mode
        let mut handles = vec![];
        for i in 0..total_jobs {
            let task_display = display.clone();
            let i = i;
            handles.push(tokio::spawn(async move {
                task_display.spawn_with_mode(ThreadMode::Capturing, move || format!("task-{}", i)).await.unwrap();
                for j in 0..3 {
                    // Just log messages, no TestEnv inside spawned tasks
                    let message = format!("Thread {}: Message {}", i, j);
                    // Brief sleep to allow interleaving
                    sleep(Duration::from_millis(10)).await;
                }
            }));
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Display after all tasks completed
        display.display().await.unwrap();
    }, 5).await.unwrap();
    
    // Always stop the display outside the timeout
    display.stop().await.unwrap();
}

#[tokio::test]
async fn test_progress_display_window_mode_concurrent() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await;
    
    // Run test within timeout
    with_timeout(async {
        let total_jobs = 5;
        
        // Spawn multiple tasks with Window mode
        let mut handles = vec![];
        for i in 0..total_jobs {
            let task_display = display.clone();
            let i = i;
            handles.push(tokio::spawn(async move {
                task_display.spawn_with_mode(ThreadMode::Window(3), move || format!("task-{}", i)).await.unwrap();
                for j in 0..3 {
                    // Just log messages, no TestEnv inside spawned tasks
                    let message = format!("Thread {}: Message {}", i, j);
                    // Brief sleep to allow interleaving
                    sleep(Duration::from_millis(10)).await;
                }
            }));
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Display after all tasks completed
        display.display().await.unwrap();
    }, 5).await.unwrap();
    
    // Always stop the display outside the timeout
    display.stop().await.unwrap();
}

#[tokio::test]
async fn test_progress_display_window_with_title_mode_concurrent() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await;
    
    // Run test within timeout
    with_timeout(async {
        let total_jobs = 5;
        
        // Spawn multiple tasks with WindowWithTitle mode
        let mut handles = vec![];
        for i in 0..total_jobs {
            let task_display = display.clone();
            let i = i;
            handles.push(tokio::spawn(async move {
                task_display.spawn_with_mode(ThreadMode::WindowWithTitle(3), move || format!("task-{}", i)).await.unwrap();
                for j in 0..3 {
                    // Just log messages, no TestEnv inside spawned tasks
                    let message = format!("Thread {}: Message {}", i, j);
                    // Brief sleep to allow interleaving
                    sleep(Duration::from_millis(10)).await;
                }
            }));
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Display after all tasks completed
        display.display().await.unwrap();
    }, 5).await.unwrap();
    
    // Always stop the display outside the timeout
    display.stop().await.unwrap();
} 
