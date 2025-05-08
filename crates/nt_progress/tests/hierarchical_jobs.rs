use anyhow::Result;
use nt_progress::{ProgressDisplay, ThreadMode};

/// Test hierarchical job tracking with parent and child tasks
#[tokio::test]
async fn test_hierarchical_job_tracking() -> Result<()> {
    // Create progress display
    let progress_display = ProgressDisplay::new().await?;
    
    // Create a parent task
    let mut parent_task = progress_display.create_task(ThreadMode::Window(2), 10).await?;
    parent_task.capture_stdout("Parent task".to_string()).await?;
    
    // Create two child tasks
    let child_task1 = progress_display.create_child_task_with_title(
        parent_task.thread_id(),
        ThreadMode::Window(2),
        "Child task 1".to_string(),
        Some(10)
    ).await?;
    
    let child_task2 = progress_display.create_child_task_with_title(
        parent_task.thread_id(),
        ThreadMode::Window(2),
        "Child task 2".to_string(),
        Some(10)
    ).await?;
    
    // Verify the parent-child relationships
    let child_ids = parent_task.get_child_job_ids().await?;
    assert_eq!(child_ids.len(), 2);
    assert!(child_ids.contains(&child_task1.thread_id()));
    assert!(child_ids.contains(&child_task2.thread_id()));
    
    let parent_id1 = child_task1.get_parent_job_id().await?;
    assert_eq!(parent_id1, Some(parent_task.thread_id()));
    
    let parent_id2 = child_task2.get_parent_job_id().await?;
    assert_eq!(parent_id2, Some(parent_task.thread_id()));
    
    // Test get_child_tasks
    let child_tasks = progress_display.get_child_tasks(parent_task.thread_id()).await?;
    assert_eq!(child_tasks.len(), 2);
    
    // Test progress tracking
    parent_task.set_progress(2).await?;  // 20% complete
    child_task1.set_progress(5).await?;  // 50% complete
    child_task2.set_progress(10).await?; // 100% complete
    
    // Check individual progress
    let parent_progress = parent_task.get_progress_percentage().await?;
    assert_eq!(parent_progress, 20.0);
    
    let child1_progress = child_task1.get_progress_percentage().await?;
    assert_eq!(child1_progress, 50.0);
    
    let child2_progress = child_task2.get_progress_percentage().await?;
    assert_eq!(child2_progress, 100.0);
    
    // Check cumulative progress (average of parent and children)
    let cumulative_progress = progress_display.get_cumulative_progress(parent_task.thread_id()).await?;
    // Expected: (20 + 50 + 100) / 3 = 56.67%
    assert!(cumulative_progress > 56.0 && cumulative_progress < 57.0);
    
    // Test with nested hierarchy by creating a grandchild task
    let grandchild_task = progress_display.create_child_task_with_title(
        child_task1.thread_id(),
        ThreadMode::Window(2),
        "Grandchild task".to_string(),
        Some(10)
    ).await?;
    
    // Set progress for grandchild
    grandchild_task.set_progress(7).await?; // 70% complete
    
    // Verify parent-child relationship for grandchild
    let grandchild_parent_id = grandchild_task.get_parent_job_id().await?;
    assert_eq!(grandchild_parent_id, Some(child_task1.thread_id()));
    
    let child1_child_ids = child_task1.get_child_job_ids().await?;
    assert_eq!(child1_child_ids.len(), 1);
    assert_eq!(child1_child_ids[0], grandchild_task.thread_id());
    
    // Check cumulative progress with nested hierarchy
    // Child1's progress should now be (50 + 70) / 2 = 60%
    let child1_cumulative = progress_display.get_cumulative_progress(child_task1.thread_id()).await?;
    assert!(child1_cumulative > 59.0 && child1_cumulative < 61.0);
    
    // Parent's progress should now be (20 + 60 + 100) / 3 = 60%
    let parent_cumulative = progress_display.get_cumulative_progress(parent_task.thread_id()).await?;
    assert!(parent_cumulative > 59.0 && parent_cumulative < 61.0);
    
    // Clean up
    progress_display.stop().await?;
    
    Ok(())
}

/// Test removing child tasks
#[tokio::test]
async fn test_removing_child_tasks() -> Result<()> {
    // Create progress display
    let progress_display = ProgressDisplay::new().await?;
    
    // Create a parent task
    let mut parent_task = progress_display.create_task(ThreadMode::Window(2), 10).await?;
    parent_task.capture_stdout("Parent task".to_string()).await?;
    
    // Create two child tasks
    let child_task1 = progress_display.create_child_task_with_title(
        parent_task.thread_id(),
        ThreadMode::Window(2),
        "Child task 1".to_string(),
        Some(10)
    ).await?;
    
    let child_task2 = progress_display.create_child_task_with_title(
        parent_task.thread_id(),
        ThreadMode::Window(2),
        "Child task 2".to_string(),
        Some(10)
    ).await?;
    
    // Verify we have two children
    let child_tasks = progress_display.get_child_tasks(parent_task.thread_id()).await?;
    assert_eq!(child_tasks.len(), 2);
    
    // Remove one child
    let removed = parent_task.remove_child_job(child_task1.thread_id()).await?;
    assert!(removed);
    
    // Verify we now have only one child
    let child_tasks = progress_display.get_child_tasks(parent_task.thread_id()).await?;
    assert_eq!(child_tasks.len(), 1);
    assert_eq!(child_tasks[0].thread_id(), child_task2.thread_id());
    
    // Try to remove a non-existent child
    let removed = parent_task.remove_child_job(999).await?;
    assert!(!removed);
    
    // Clean up
    progress_display.stop().await?;
    
    Ok(())
}

/// Test the spawn_child convenience method
#[tokio::test]
async fn test_spawn_child_convenience() -> Result<()> {
    // Create progress display
    let progress_display = ProgressDisplay::new().await?;
    
    // Create a parent task
    let mut parent_task = progress_display.create_task(ThreadMode::Window(2), 10).await?;
    parent_task.capture_stdout("Parent task".to_string()).await?;
    
    // Create a child task with the convenience method
    let child_task = progress_display.spawn_child(
        parent_task.thread_id(),
        ThreadMode::Window(2),
        || "Child from convenience method".to_string(),
        Some(10)
    ).await?;
    
    // Verify the parent-child relationship
    let child_ids = parent_task.get_child_job_ids().await?;
    assert_eq!(child_ids.len(), 1);
    assert_eq!(child_ids[0], child_task.thread_id());
    
    let parent_id = child_task.get_parent_job_id().await?;
    assert_eq!(parent_id, Some(parent_task.thread_id()));
    
    // Clean up
    progress_display.stop().await?;
    
    Ok(())
} 