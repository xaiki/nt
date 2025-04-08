use std::io::{stdout, Write};
use std::time::Duration;
use tokio::time::sleep;
use crate::ProgressDisplay;
use crate::modes::ThreadMode;

#[tokio::test]
async fn test_progress_display_all_modes() {
    let modes = [
        ThreadMode::Limited,
        ThreadMode::Capturing,
        ThreadMode::Window(3),
        ThreadMode::WindowWithTitle(3),
    ];

    for mode in modes {
        let mut display = ProgressDisplay::new().await;
        let total_jobs = 5;
        
        // Spawn multiple tasks with the same mode
        let mut handles = vec![];
        for i in 0..total_jobs {
            let mut display = display.clone();
            let i = i; // Move i into the closure
            handles.push(tokio::spawn(async move {
                display.spawn_with_mode(mode, move || format!("task-{}", i)).await.unwrap();
                for j in 0..3 {
                    writeln!(stdout(), "Thread {}: Message {}", i, j).unwrap();
                    sleep(Duration::from_millis(50)).await;
                }
            }));
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify final state by checking the display
        display.display().await.unwrap();
        display.stop().await.unwrap();
    }
}

#[tokio::test]
async fn test_progress_display_high_concurrency() {
    let mut display = ProgressDisplay::new().await;
    let total_jobs = 20;
    
    let mut handles = vec![];
    for i in 0..total_jobs {
        let mut display = display.clone();
        let i = i; // Move i into the closure
        handles.push(tokio::spawn(async move {
            display.spawn_with_mode(ThreadMode::Window(5), move || format!("task-{}", i)).await.unwrap();
            for j in 0..10 {
                writeln!(stdout(), "Thread {}: Message {}", i, j).unwrap();
                sleep(Duration::from_millis(10)).await;
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // Verify the display
    display.display().await.unwrap();
    display.stop().await.unwrap();
}

#[tokio::test]
async fn test_progress_display_different_modes() {
    let mut display = ProgressDisplay::new().await;
    
    // Spawn tasks with different modes
    display.spawn_with_mode(ThreadMode::Limited, || "limited-task".to_string()).await.unwrap();
    writeln!(stdout(), "Test 1").unwrap();
    writeln!(stdout(), "Test 2").unwrap();
    display.display().await.unwrap();

    display.spawn_with_mode(ThreadMode::Window(2), || "window-task".to_string()).await.unwrap();
    writeln!(stdout(), "Test 3").unwrap();
    writeln!(stdout(), "Test 4").unwrap();
    display.display().await.unwrap();

    display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "title-task".to_string()).await.unwrap();
    writeln!(stdout(), "Test 5").unwrap();
    display.display().await.unwrap();
    display.stop().await.unwrap();
}

#[tokio::test]
async fn test_progress_display_error_handling() {
    let mut display = ProgressDisplay::new().await;
    
    // Test invalid mode configuration
    assert!(display.spawn_with_mode(ThreadMode::Window(0), || "invalid".to_string()).await.is_err());
    
    // Test task failure
    let mut display2 = ProgressDisplay::new().await;
    let handle = tokio::spawn(async move {
        display2.spawn_with_mode(ThreadMode::Limited, || "failing-task".to_string()).await.unwrap();
        writeln!(stdout(), "Before error").unwrap();
        panic!("Simulated task failure");
    });
    
    // Should not crash the display
    assert!(handle.await.is_err());
    display.display().await.unwrap();
    display.stop().await.unwrap();
}

#[tokio::test]
async fn test_progress_display_mode_specific() {
    // Test Limited mode
    let mut display = ProgressDisplay::new().await;
    display.spawn_with_mode(ThreadMode::Limited, || "limited".to_string()).await.unwrap();
    for i in 0..10 {
        writeln!(stdout(), "Line {}", i).unwrap();
    }
    display.display().await.unwrap();
    display.stop().await.unwrap();
    
    // Test Capturing mode
    let mut display = ProgressDisplay::new().await;
    display.spawn_with_mode(ThreadMode::Capturing, || "capturing".to_string()).await.unwrap();
    for i in 0..10 {
        writeln!(stdout(), "Line {}", i).unwrap();
    }
    display.display().await.unwrap();
    display.stop().await.unwrap();
    
    // Test Window mode
    let mut display = ProgressDisplay::new().await;
    display.spawn_with_mode(ThreadMode::Window(3), || "window".to_string()).await.unwrap();
    for i in 0..10 {
        writeln!(stdout(), "Line {}", i).unwrap();
    }
    display.display().await.unwrap();
    display.stop().await.unwrap();
    
    // Test WindowWithTitle mode
    let mut display = ProgressDisplay::new().await;
    display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "title".to_string()).await.unwrap();
    writeln!(stdout(), "Test with emoji 🚀").unwrap();
    writeln!(stdout(), "Another line 📝").unwrap();
    display.display().await.unwrap();
    display.stop().await.unwrap();
}

#[tokio::test]
async fn test_progress_display_edge_cases() {
    // Test empty output
    let mut display = ProgressDisplay::new().await;
    display.spawn_with_mode(ThreadMode::Limited, || "empty".to_string()).await.unwrap();
    writeln!(stdout(), "").unwrap();
    display.display().await.unwrap();
    display.stop().await.unwrap();
    
    // Test very long lines
    let mut display = ProgressDisplay::new().await;
    let long_line = "x".repeat(1000);
    display.spawn_with_mode(ThreadMode::Limited, || "long".to_string()).await.unwrap();
    writeln!(stdout(), "{}", long_line).unwrap();
    display.display().await.unwrap();
    display.stop().await.unwrap();
    
    // Test special characters
    let mut display = ProgressDisplay::new().await;
    display.spawn_with_mode(ThreadMode::Limited, || "special".to_string()).await.unwrap();
    writeln!(stdout(), "Test with \n newlines \t tabs \r returns").unwrap();
    writeln!(stdout(), "Test with unicode: 你好世界").unwrap();
    display.display().await.unwrap();
    display.stop().await.unwrap();
}

#[tokio::test]
async fn test_progress_display_concurrency() {
    // Test task cancellation
    let mut display = ProgressDisplay::new().await;
    let total_jobs = 5;
    let mut handles = vec![];
    for i in 0..total_jobs {
        let mut display = display.clone();
        let i = i;
        handles.push(tokio::spawn(async move {
            display.spawn_with_mode(ThreadMode::Limited, move || format!("task-{}", i)).await.unwrap();
            for j in 0..3 {
                writeln!(stdout(), "Thread {}: Message {}", i, j).unwrap();
                sleep(Duration::from_millis(50)).await;
            }
            // Simulate task cancellation
            if i == 2 {
                return;
            }
            writeln!(stdout(), "Thread {}: Completed", i).unwrap();
        }));
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
    display.display().await.unwrap();
    display.stop().await.unwrap();
}

#[tokio::test]
async fn test_progress_display_rapid_messages() {
    // Test rapid message bursts
    let mut display = ProgressDisplay::new().await;
    let mut display_clone = display.clone();
    let handle = tokio::spawn(async move {
        display_clone.spawn_with_mode(ThreadMode::Window(5), || "burst".to_string()).await.unwrap();
        for _ in 0..100 {
            writeln!(stdout(), "Burst message").unwrap();
        }
    });
    
    handle.await.unwrap();
    display.display().await.unwrap();
    display.stop().await.unwrap();
}

#[tokio::test]
async fn test_display_formatting() {
    // Test basic output formatting
    let mut display = ProgressDisplay::new().await;
    
    // Add some test output
    display.spawn_with_mode(ThreadMode::Limited, || "test-task").await.unwrap();
    writeln!(stdout(), "Test line 1").unwrap();
    writeln!(stdout(), "Test line 2").unwrap();
    
    // Verify display
    display.display().await.unwrap();
    display.stop().await.unwrap();
}

#[tokio::test]
async fn test_window_mode_display() {
    let mut display = ProgressDisplay::new_with_mode(ThreadMode::Window(2)).await;
    
    // Add more lines than the window size
    display.spawn_with_mode(ThreadMode::Window(2), || "window-task").await.unwrap();
    writeln!(stdout(), "Line 1").unwrap();
    writeln!(stdout(), "Line 2").unwrap();
    writeln!(stdout(), "Line 3").unwrap();
    
    // Verify display
    display.display().await.unwrap();
    display.stop().await.unwrap();
}

#[tokio::test]
async fn test_terminal_size_handling() {
    let mut display = ProgressDisplay::new().await;
    
    // Set a small terminal size
    *display.terminal_size.lock().await = (80, 2);
    
    // Add more lines than terminal height
    display.spawn_with_mode(ThreadMode::Limited, || "size-test").await.unwrap();
    writeln!(stdout(), "Line 1").unwrap();
    writeln!(stdout(), "Line 2").unwrap();
    writeln!(stdout(), "Line 3").unwrap();
    
    // Verify display
    display.display().await.unwrap();
    display.stop().await.unwrap();
} 