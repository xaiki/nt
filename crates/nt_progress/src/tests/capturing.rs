use crate::ProgressDisplay;
use crate::modes::ThreadMode;
use crate::test_utils::TestEnv;
use tokio::time::sleep;
use std::time::Duration;

#[tokio::test]
async fn test_capturing_mode_basic() {
    let display = ProgressDisplay::new_with_mode(ThreadMode::Capturing).await;
    let mut env = TestEnv::new(80, 24);
    
    // Test basic output
    display.spawn_with_mode(ThreadMode::Capturing, || "test-task".to_string()).await.unwrap();
    env.writeln("Test line 1");
    env.writeln("Test line 2");
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_capturing_mode_multiple_lines() {
    let display = ProgressDisplay::new_with_mode(ThreadMode::Capturing).await;
    let mut env = TestEnv::new(80, 24);
    
    // Test multiple lines
    display.spawn_with_mode(ThreadMode::Capturing, || "multi-line".to_string()).await.unwrap();
    for i in 0..10 {
        env.writeln(&format!("Line {}", i));
    }
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_capturing_mode_concurrent() {
    let display = ProgressDisplay::new_with_mode(ThreadMode::Capturing).await;
    let mut handles = vec![];
    let mut main_env = TestEnv::new(80, 24);
    let (width, height) = main_env.size();
    
    // Spawn multiple tasks in Capturing mode
    for i in 0..3 {
        let display = display.clone();
        let i = i;
        let mut task_env = TestEnv::new(width, height);
        handles.push(tokio::spawn(async move {
            display.spawn_with_mode(ThreadMode::Capturing, move || format!("task-{}", i)).await.unwrap();
            for j in 0..5 {
                task_env.writeln(&format!("Thread {}: Message {}", i, j));
                sleep(Duration::from_millis(50)).await;
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
async fn test_capturing_mode_error_handling() {
    let display = ProgressDisplay::new_with_mode(ThreadMode::Capturing).await;
    let mut env = TestEnv::new(80, 24);
    
    // Test error handling
    display.spawn_with_mode(ThreadMode::Capturing, || "error-test".to_string()).await.unwrap();
    env.writeln("Before error");
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_capturing_mode_special_chars() {
    let display = ProgressDisplay::new_with_mode(ThreadMode::Capturing).await;
    let mut env = TestEnv::new(80, 24);
    
    // Test special characters
    display.spawn_with_mode(ThreadMode::Capturing, || "special-chars".to_string()).await.unwrap();
    env.writeln("Test with emoji 🚀");
    env.writeln("Test with unicode: 你好世界");
    env.writeln("Test with control chars: \n\t\r");
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_capturing_mode_long_lines() {
    let display = ProgressDisplay::new_with_mode(ThreadMode::Capturing).await;
    let mut env = TestEnv::new(80, 24);
    
    // Test long lines
    display.spawn_with_mode(ThreadMode::Capturing, || "long-lines".to_string()).await.unwrap();
    env.writeln(&"x".repeat(1000));
    env.writeln(&"y".repeat(500));
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
} 