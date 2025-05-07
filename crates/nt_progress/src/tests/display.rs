use std::time::Duration;
use tokio::time::sleep;
use crate::ProgressDisplay;
use crate::modes::ThreadMode;
use crate::terminal::TestEnv;
use crate::tests::common::with_timeout;
use std::sync::Arc;

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
    let display = ProgressDisplay::new().await.unwrap();
    
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
    let display = ProgressDisplay::new().await.unwrap();
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
    let display = ProgressDisplay::new().await.unwrap();
    
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
    let display = ProgressDisplay::new().await.unwrap();
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
    let display = ProgressDisplay::new().await.unwrap();
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
    let display = ProgressDisplay::new().await.unwrap();
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
    let display = ProgressDisplay::new().await.unwrap();
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
    let display = ProgressDisplay::new().await.unwrap();
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
    let display = ProgressDisplay::new().await.unwrap();
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
    let display = ProgressDisplay::new().await.unwrap();
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
    let display = ProgressDisplay::new().await.unwrap();
    let mut main_env = TestEnv::new(80, 24);
    let (width, height) = main_env.size();
    
    // Run test within timeout
    with_timeout(async {
        let total_jobs = 5;
        
        // Spawn multiple tasks
        let mut handles = vec![];
        for i in 0..total_jobs {
            let display_ref = Arc::new(display.clone());
            let i = i;
            handles.push(tokio::spawn(async move {
                let mut task_env = TestEnv::new(width, height);
                let handle = display_ref.spawn_with_mode(ThreadMode::Window(3), move || format!("task-{}", i)).await.unwrap();
                for j in 0..5 {
                    task_env.writeln(&format!("Thread {}: Message {}", i, j));
                    sleep(Duration::from_millis(50)).await;
                }
                task_env
            }));
        }
        
        // Wait for all tasks to complete and combine their outputs
        let mut final_env = TestEnv::new(width, height);
        for handle in handles {
            let task_env = handle.await.unwrap();
            let content = task_env.contents();
            if !content.is_empty() {
                final_env.write(&content);
            }
        }
        
        // Verify final state
        display.display().await.unwrap();
    }, 5).await.unwrap();
    
    // Always clean up outside timeout
    display.stop().await.unwrap();
}

#[tokio::test]
async fn test_progress_display_resource_cleanup() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await.unwrap();
    let env = TestEnv::new(80, 24);
    
    // Run test within timeout
    with_timeout(async {
        // Spawn multiple tasks
        let mut handles = vec![];
        for i in 0..3 {
            let handle = display.spawn_with_mode(ThreadMode::Window(3), move || format!("task-{}", i)).await.unwrap();
            handles.push(handle.clone());
        }
        
        // Cancel some tasks
        handles[0].clone().cancel().await.unwrap();
        
        // Stop the display
        display.stop().await.unwrap();
        
        // Verify all resources are cleaned up
        assert_eq!(display.thread_count().await, 0);
        
        // Try to use the display after stop
        let result = display.spawn_with_mode(ThreadMode::Limited, || "after-stop".to_string()).await;
        assert!(result.is_err(), "Should not allow spawning after stop");
        
        display.display().await.unwrap();
    }, 5).await.unwrap();
    
    env.verify();
} 
