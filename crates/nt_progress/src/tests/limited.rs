use crate::ProgressDisplay;
use crate::modes::ThreadMode;
use crate::tests::common::TestEnv;
use tokio::time::sleep;
use std::time::Duration;

#[tokio::test]
async fn test_limited_basic() {
    let display = ProgressDisplay::new_with_mode(ThreadMode::Limited).await;
    let mut env = TestEnv::new(80, 24);
    
    let _handle = display.spawn_with_mode(ThreadMode::Limited, || "limited-test").await.unwrap();
    env.writeln("Test message");
    
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_limited_concurrent() {
    let display = ProgressDisplay::new().await;
    let total_jobs = 5;
    
    // Spawn multiple tasks in Limited mode
    let mut handles = vec![];
    for i in 0..total_jobs {
        let display = display.clone();
        let mut env = TestEnv::new(80, 24);
        let i = i;
        handles.push(tokio::spawn(async move {
            display.spawn_with_mode(ThreadMode::Limited, move || format!("task-{}", i)).await.unwrap();
            for j in 0..3 {
                env.writeln(&format!("Thread {}: Message {}", i, j));
                sleep(Duration::from_millis(50)).await;
            }
            env
        }));
    }
    
    // Wait for all tasks to complete and combine their outputs
    let mut final_env = TestEnv::new(80, 24);
    for handle in handles {
        let task_env = handle.await.unwrap();
        for line in task_env.expected {
            final_env.write(&line);
        }
    }
    
    // Verify final state
    display.display().await.unwrap();
    display.stop().await.unwrap();
    final_env.verify();
} 