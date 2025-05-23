use std::fmt::Debug;

use super::base_config::JobStatus;

/// Trait for types that contain a BaseConfig, either directly or through composition.
///
/// This trait is used to provide a uniform way to access the BaseConfig
/// of different types, which enables generic implementations for traits
/// like JobTracker.
pub trait HasBaseConfig {
    /// Get a reference to the BaseConfig.
    ///
    /// # Returns
    /// A reference to the BaseConfig
    fn base_config(&self) -> &super::base_config::BaseConfig;
    
    /// Get a mutable reference to the BaseConfig.
    ///
    /// # Returns
    /// A mutable reference to the BaseConfig
    fn base_config_mut(&mut self) -> &mut super::base_config::BaseConfig;
}

/// Trait for tracking job progress across different display modes.
///
/// This trait is implemented by display modes to track the progress
/// of jobs being processed. It provides methods for getting the total
/// number of jobs and incrementing the completed jobs counter.
pub trait JobTracker: Send + Sync + Debug {
    /// Get the total number of jobs assigned to this tracker.
    ///
    /// # Returns
    /// The total number of jobs
    fn get_total_jobs(&self) -> usize;
    
    /// Increment the completed jobs counter and return the new value.
    ///
    /// This method is used to mark a job as completed and get the
    /// new count of completed jobs.
    ///
    /// # Returns
    /// The new count of completed jobs
    fn increment_completed_jobs(&self) -> usize;
    
    /// Set the total number of jobs for this tracker.
    ///
    /// This method is used to update the total number of jobs
    /// when it was not known at creation time or has changed.
    ///
    /// # Parameters
    /// * `total` - The new total number of jobs
    fn set_total_jobs(&mut self, total: usize);
}

/// Trait for jobs that can be cancelled.
///
/// This trait extends JobTracker to support cancellation operations,
/// allowing jobs to be cancelled and their cancellation state to be tracked.
pub trait CancellableJob: JobTracker {
    /// Check if this job has been cancelled.
    ///
    /// # Returns
    /// `true` if the job has been cancelled, `false` otherwise
    fn is_cancelled(&self) -> bool;
    
    /// Cancel this job with an optional reason.
    ///
    /// # Parameters
    /// * `reason` - An optional reason for the cancellation
    fn set_cancelled(&mut self, reason: Option<String>);
    
    /// Get the reason this job was cancelled, if any.
    ///
    /// # Returns
    /// The cancellation reason, or None if the job wasn't cancelled or no reason was provided
    fn get_cancellation_reason(&self) -> Option<String>;
}

/// Trait for jobs that can be paused and resumed.
///
/// This trait extends JobTracker to support pausing and resuming jobs,
/// allowing for temporary suspension of progress tracking.
pub trait PausableJob: JobTracker {
    /// Pause this job.
    ///
    /// When a job is paused, it will stop incrementing its progress counter
    /// until it is resumed.
    fn pause(&mut self);
    
    /// Resume this job.
    ///
    /// When a job is resumed, it will start incrementing its progress counter again.
    fn resume(&mut self);
    
    /// Check if this job is currently paused.
    ///
    /// # Returns
    /// `true` if the job is paused, `false` otherwise
    fn is_paused(&self) -> bool;
}

/// Trait for tracking hierarchical job progress.
///
/// This trait extends JobTracker to support parent-child relationships
/// between jobs, enabling hierarchical job tracking.
pub trait HierarchicalJobTracker: JobTracker {
    /// Add a child job to this tracker.
    ///
    /// # Parameters
    /// * `child_id` - The ID of the child job
    ///
    /// # Returns
    /// `true` if the child was successfully added, `false` otherwise
    fn add_child_job(&mut self, child_id: usize) -> bool;
    
    /// Remove a child job from this tracker.
    ///
    /// # Parameters
    /// * `child_id` - The ID of the child job to remove
    ///
    /// # Returns
    /// `true` if the child was successfully removed, `false` otherwise
    fn remove_child_job(&mut self, child_id: usize) -> bool;
    
    /// Get the parent job ID if this job has a parent.
    ///
    /// # Returns
    /// The parent job ID, or `None` if this job has no parent
    fn get_parent_job_id(&self) -> Option<usize>;
    
    /// Set the parent job ID for this job.
    ///
    /// # Parameters
    /// * `parent_id` - The ID of the parent job
    fn set_parent_job_id(&mut self, parent_id: usize);
    
    /// Get the list of child job IDs associated with this job.
    ///
    /// # Returns
    /// A vector of child job IDs
    fn get_child_job_ids(&self) -> Vec<usize>;
    
    /// Calculate the cumulative progress including child jobs.
    ///
    /// # Returns
    /// A float between 0.0 and 1.0 representing the progress across all child jobs
    fn get_cumulative_progress(&self) -> f64;
}

/// Trait for jobs that can be prioritized.
///
/// This trait extends JobTracker to support job prioritization,
/// allowing for jobs to be prioritized based on importance.
pub trait PrioritizedJob: JobTracker {
    /// Get the priority of this job.
    ///
    /// Higher values indicate higher priority.
    ///
    /// # Returns
    /// The priority value of the job
    fn get_priority(&self) -> u32;
    
    /// Set the priority of this job.
    ///
    /// Higher values indicate higher priority.
    ///
    /// # Parameters
    /// * `priority` - The new priority value
    fn set_priority(&mut self, priority: u32);
}

/// Trait for jobs that depend on other jobs.
///
/// This trait extends JobTracker to support job dependencies,
/// allowing for jobs to depend on the completion of other jobs.
pub trait DependentJob: JobTracker {
    /// Add a dependency on another job.
    ///
    /// # Parameters
    /// * `job_id` - The ID of the job this job depends on
    ///
    /// # Returns
    /// `true` if the dependency was successfully added, `false` otherwise
    fn add_dependency(&mut self, job_id: usize) -> bool;
    
    /// Remove a dependency.
    ///
    /// # Parameters
    /// * `job_id` - The ID of the dependency to remove
    ///
    /// # Returns
    /// `true` if the dependency was successfully removed, `false` otherwise
    fn remove_dependency(&mut self, job_id: usize) -> bool;
    
    /// Get the dependencies of this job.
    ///
    /// # Returns
    /// A vector of job IDs that this job depends on
    fn get_dependencies(&self) -> Vec<usize>;
    
    /// Check if this job has dependencies.
    ///
    /// # Returns
    /// `true` if this job has one or more dependencies, `false` otherwise
    fn has_dependencies(&self) -> bool;
    
    /// Check if a specific dependency is satisfied.
    ///
    /// # Parameters
    /// * `job_id` - The job ID to check
    /// * `is_completed` - Whether the job is completed
    ///
    /// # Returns
    /// `true` if the dependency is satisfied, `false` otherwise
    fn is_dependency_satisfied(&self, job_id: usize, is_completed: bool) -> bool;
}

/// Trait for jobs that can handle failures and retries.
///
/// This trait extends JobTracker to support failure handling and retry logic,
/// allowing for jobs to be retried when they fail.
pub trait FailureHandlingJob: JobTracker {
    /// Mark the job as failed.
    ///
    /// # Parameters
    /// * `error` - The error message describing the failure
    ///
    /// # Returns
    /// The current number of failures for this job
    fn mark_failed(&mut self, error: &str) -> usize;
    
    /// Mark the job as succeeded.
    ///
    /// This resets the failure count and clears any error messages.
    fn mark_succeeded(&mut self);
    
    /// Get the number of times this job has failed.
    ///
    /// # Returns
    /// The number of failures
    fn get_failure_count(&self) -> usize;
    
    /// Get the most recent error message, if any.
    ///
    /// # Returns
    /// The most recent error message, or None if the job hasn't failed
    fn get_error_message(&self) -> Option<String>;
    
    /// Check if the job has failed.
    ///
    /// # Returns
    /// `true` if the job has failed, `false` otherwise
    fn has_failed(&self) -> bool;
    
    /// Retry the job.
    ///
    /// This resets the error state but keeps track of the number of retries.
    ///
    /// # Returns
    /// The current retry count
    fn retry(&mut self) -> usize;
    
    /// Get the number of times this job has been retried.
    ///
    /// # Returns
    /// The number of retries
    fn get_retry_count(&self) -> usize;
    
    /// Set the maximum number of retries allowed for this job.
    ///
    /// # Parameters
    /// * `max_retries` - The maximum number of retries allowed
    fn set_max_retries(&mut self, max_retries: usize);
    
    /// Get the maximum number of retries allowed for this job.
    ///
    /// # Returns
    /// The maximum number of retries allowed
    fn get_max_retries(&self) -> usize;
    
    /// Check if the job has reached its maximum retry limit.
    ///
    /// # Returns
    /// `true` if the job has reached its retry limit, `false` otherwise
    fn has_reached_retry_limit(&self) -> bool;
}

/// Trait for tracking and reporting job status.
///
/// This trait extends JobTracker to support status tracking and reporting,
/// allowing for jobs to have distinct statuses like Pending, Running, Completed, etc.
pub trait JobStatusTracker: JobTracker {
    /// Get the current status of the job.
    ///
    /// # Returns
    /// The current job status
    fn get_status(&self) -> JobStatus;
    
    /// Set the job status.
    ///
    /// # Parameters
    /// * `status` - The new job status
    fn set_status(&mut self, status: JobStatus);
    
    /// Mark the job as running.
    ///
    /// This sets the status to Running.
    fn mark_running(&mut self);
    
    /// Mark the job as completed.
    ///
    /// This sets the status to Completed.
    fn mark_completed(&mut self);
    
    /// Check if the job is in the specified status.
    ///
    /// # Parameters
    /// * `status` - The status to check against
    ///
    /// # Returns
    /// `true` if the job is in the specified status, `false` otherwise
    fn is_in_status(&self, status: JobStatus) -> bool;
    
    /// Check if the job is pending.
    ///
    /// # Returns
    /// `true` if the job is pending, `false` otherwise
    fn is_pending(&self) -> bool;
    
    /// Check if the job is running.
    ///
    /// # Returns
    /// `true` if the job is running, `false` otherwise
    fn is_running(&self) -> bool;
    
    /// Check if the job is completed.
    ///
    /// # Returns
    /// `true` if the job is completed, `false` otherwise
    fn is_completed(&self) -> bool;
    
    /// Check if the job is in retry state.
    ///
    /// # Returns
    /// `true` if the job is in retry state, `false` otherwise
    fn is_retrying(&self) -> bool;
}

/// Trait for jobs that report time-related statistics.
///
/// This trait extends JobTracker to support time tracking and estimation,
/// providing methods to retrieve elapsed time, time remaining, and progress speed.
pub trait TimeTrackingJob: JobTracker {
    /// Get the elapsed time since the job started.
    ///
    /// # Returns
    /// The duration since the job started
    fn get_elapsed_time(&self) -> std::time::Duration;
    
    /// Get the estimated time remaining until the job completes.
    ///
    /// # Returns
    /// Some(Duration) with the estimated time remaining, or None if an estimate cannot be made
    fn get_estimated_time_remaining(&self) -> Option<std::time::Duration>;
    
    /// Get the current progress speed in units per second.
    ///
    /// # Returns
    /// Some(f64) with the speed in units per second, or None if the speed cannot be calculated
    fn get_progress_speed(&self) -> Option<f64>;
    
    /// Update the time estimates based on current progress.
    ///
    /// # Returns
    /// The updated progress percentage
    fn update_time_estimates(&self) -> f64;
}

/// Trait for jobs that can be persisted to storage for long-running operations.
///
/// This trait extends JobTracker to support persistence operations,
/// allowing job state to be saved to and loaded from storage for use
/// across multiple program executions or restarts.
pub trait PersistentJob: 
    JobTracker + 
    JobStatusTracker + 
    TimeTrackingJob + 
    FailureHandlingJob +
    HierarchicalJobTracker
{
    /// Save the current job state to the specified path.
    ///
    /// # Parameters
    /// * `path` - The path to save the job state to
    ///
    /// # Returns
    /// `Ok(())` if the job state was successfully saved, or an error otherwise
    fn save_state(&self, path: &str) -> std::io::Result<()>;
    
    /// Load job state from the specified path.
    ///
    /// # Parameters
    /// * `path` - The path to load the job state from
    ///
    /// # Returns
    /// `Ok(())` if the job state was successfully loaded, or an error otherwise
    fn load_state(&mut self, path: &str) -> std::io::Result<()>;
    
    /// Check if a job state exists at the specified path.
    ///
    /// # Parameters
    /// * `path` - The path to check for existing job state
    ///
    /// # Returns
    /// `true` if job state exists at the path, `false` otherwise
    fn state_exists(path: &str) -> bool;
    
    /// Generate a unique identifier for the job for persistence purposes.
    ///
    /// # Returns
    /// A string identifier for the job
    fn get_persistence_id(&self) -> String;
    
    /// Set the persistence identifier for the job.
    ///
    /// # Parameters
    /// * `id` - The new identifier for the job
    fn set_persistence_id(&mut self, id: String);
    
    /// Check if the job has a persistence identifier.
    ///
    /// # Returns
    /// `true` if the job has a persistence ID, `false` otherwise
    fn has_persistence_id(&self) -> bool;
}

// Generic implementations for base traits

impl<T: HasBaseConfig + Send + Sync + Debug> JobTracker for T {
    fn get_total_jobs(&self) -> usize {
        self.base_config().get_total_jobs()
    }
    
    fn increment_completed_jobs(&self) -> usize {
        self.base_config().increment_completed_jobs()
    }
    
    fn set_total_jobs(&mut self, total: usize) {
        self.base_config_mut().set_total_jobs(total);
    }
}

impl<T: HasBaseConfig + Send + Sync + Debug> CancellableJob for T {
    fn is_cancelled(&self) -> bool {
        self.base_config().is_cancelled()
    }
    
    fn set_cancelled(&mut self, reason: Option<String>) {
        self.base_config_mut().set_cancelled(reason);
    }
    
    fn get_cancellation_reason(&self) -> Option<String> {
        self.base_config().get_cancellation_reason()
    }
}

impl<T: HasBaseConfig + Send + Sync + Debug> PausableJob for T {
    fn pause(&mut self) {
        self.base_config().pause();
    }
    
    fn resume(&mut self) {
        self.base_config().resume();
    }
    
    fn is_paused(&self) -> bool {
        self.base_config().is_paused()
    }
}

impl<T: HasBaseConfig + Send + Sync + Debug> HierarchicalJobTracker for T {
    fn add_child_job(&mut self, child_id: usize) -> bool {
        self.base_config_mut().add_child_job(child_id)
    }
    
    fn remove_child_job(&mut self, child_id: usize) -> bool {
        self.base_config_mut().remove_child_job(child_id)
    }
    
    fn get_parent_job_id(&self) -> Option<usize> {
        self.base_config().get_parent_job_id()
    }
    
    fn set_parent_job_id(&mut self, parent_id: usize) {
        self.base_config_mut().set_parent_job_id(parent_id);
    }
    
    fn get_child_job_ids(&self) -> Vec<usize> {
        self.base_config().get_child_job_ids()
    }
    
    fn get_cumulative_progress(&self) -> f64 {
        // Basic implementation - can be overridden by specific modes
        let total = self.get_total_jobs();
        if total == 0 {
            return 0.0;
        }
        
        let completed = self.base_config().get_completed_jobs();
        (completed as f64) / (total as f64)
    }
}

impl<T: HasBaseConfig + Send + Sync + Debug> PrioritizedJob for T {
    fn get_priority(&self) -> u32 {
        self.base_config().get_priority()
    }
    
    fn set_priority(&mut self, priority: u32) {
        self.base_config_mut().set_priority(priority);
    }
}

impl<T: HasBaseConfig + Send + Sync + Debug> DependentJob for T {
    fn add_dependency(&mut self, job_id: usize) -> bool {
        self.base_config_mut().add_dependency(job_id)
    }
    
    fn remove_dependency(&mut self, job_id: usize) -> bool {
        self.base_config_mut().remove_dependency(job_id)
    }
    
    fn get_dependencies(&self) -> Vec<usize> {
        self.base_config().get_dependencies()
    }
    
    fn has_dependencies(&self) -> bool {
        self.base_config().has_dependencies()
    }
    
    fn is_dependency_satisfied(&self, job_id: usize, is_completed: bool) -> bool {
        self.base_config().is_dependency_satisfied(job_id, is_completed)
    }
}

impl<T: HasBaseConfig + Send + Sync + Debug> FailureHandlingJob for T {
    fn mark_failed(&mut self, error: &str) -> usize {
        self.base_config_mut().mark_failed(error)
    }
    
    fn mark_succeeded(&mut self) {
        self.base_config_mut().mark_succeeded();
    }
    
    fn get_failure_count(&self) -> usize {
        self.base_config().get_failure_count()
    }
    
    fn get_error_message(&self) -> Option<String> {
        self.base_config().get_error_message()
    }
    
    fn has_failed(&self) -> bool {
        self.base_config().has_failed()
    }
    
    fn retry(&mut self) -> usize {
        self.base_config_mut().retry()
    }
    
    fn get_retry_count(&self) -> usize {
        self.base_config().get_retry_count()
    }
    
    fn set_max_retries(&mut self, max_retries: usize) {
        self.base_config_mut().set_max_retries(max_retries);
    }
    
    fn get_max_retries(&self) -> usize {
        self.base_config().get_max_retries()
    }
    
    fn has_reached_retry_limit(&self) -> bool {
        self.base_config().has_reached_retry_limit()
    }
}

impl<T: HasBaseConfig + Send + Sync + Debug> JobStatusTracker for T {
    fn get_status(&self) -> JobStatus {
        self.base_config().get_status()
    }
    
    fn set_status(&mut self, status: JobStatus) {
        self.base_config_mut().set_status(status);
    }
    
    fn mark_running(&mut self) {
        self.base_config_mut().mark_running();
    }
    
    fn mark_completed(&mut self) {
        self.base_config_mut().mark_completed();
    }
    
    fn is_in_status(&self, status: JobStatus) -> bool {
        self.base_config().is_in_status(status)
    }
    
    fn is_pending(&self) -> bool {
        self.base_config().is_pending()
    }
    
    fn is_running(&self) -> bool {
        self.base_config().is_running()
    }
    
    fn is_completed(&self) -> bool {
        self.base_config().is_completed()
    }
    
    fn is_retrying(&self) -> bool {
        self.base_config().is_retrying()
    }
}

impl<T: HasBaseConfig + Send + Sync + Debug> TimeTrackingJob for T {
    fn get_elapsed_time(&self) -> std::time::Duration {
        self.base_config().get_elapsed_time()
    }
    
    fn get_estimated_time_remaining(&self) -> Option<std::time::Duration> {
        self.base_config().get_estimated_time_remaining()
    }
    
    fn get_progress_speed(&self) -> Option<f64> {
        self.base_config().get_progress_speed()
    }
    
    fn update_time_estimates(&self) -> f64 {
        self.base_config().update_time_estimates()
    }
}

impl<T: HasBaseConfig + Send + Sync + Debug> PersistentJob for T {
    fn save_state(&self, path: &str) -> std::io::Result<()> {
        self.base_config().save_state(path)
    }
    
    fn load_state(&mut self, path: &str) -> std::io::Result<()> {
        self.base_config_mut().load_state(path)
    }
    
    fn state_exists(path: &str) -> bool {
        super::base_config::BaseConfig::state_exists(path)
    }
    
    fn get_persistence_id(&self) -> String {
        self.base_config().get_persistence_id()
            .unwrap_or_default()
    }
    
    fn set_persistence_id(&mut self, id: String) {
        self.base_config_mut().set_persistence_id(id);
    }
    
    fn has_persistence_id(&self) -> bool {
        self.base_config().has_persistence_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::base_config::{BaseConfig, JobStatus};
    use std::time::Duration;
    use std::fs;
    use std::path::Path;
    
    // Simple test struct that implements HasBaseConfig
    #[derive(Debug)]
    struct TestTracker {
        base: BaseConfig,
    }
    
    impl TestTracker {
        fn new(total_jobs: usize) -> Self {
            Self {
                base: BaseConfig::new(total_jobs),
            }
        }
    }
    
    impl HasBaseConfig for TestTracker {
        fn base_config(&self) -> &BaseConfig {
            &self.base
        }
        
        fn base_config_mut(&mut self) -> &mut BaseConfig {
            &mut self.base
        }
    }
    
    #[test]
    fn test_job_tracker_impl() {
        let mut tracker = TestTracker::new(10);
        assert_eq!(tracker.get_total_jobs(), 10);
        assert_eq!(tracker.increment_completed_jobs(), 1);
        
        tracker.set_total_jobs(20);
        assert_eq!(tracker.get_total_jobs(), 20);
    }
    
    #[test]
    fn test_pausable_job_impl() {
        let mut tracker = TestTracker::new(10);
        assert!(!tracker.is_paused());
        
        tracker.pause();
        assert!(tracker.is_paused());
        
        tracker.resume();
        assert!(!tracker.is_paused());
    }
    
    #[test]
    fn test_hierarchical_job_tracker_impl() {
        let mut tracker = TestTracker::new(10);
        
        // Test child jobs
        assert!(tracker.add_child_job(1));
        assert!(tracker.add_child_job(2));
        
        let children = tracker.get_child_job_ids();
        assert_eq!(children.len(), 2);
        assert!(children.contains(&1));
        assert!(children.contains(&2));
        
        // Test parent job
        assert_eq!(tracker.get_parent_job_id(), None);
        tracker.set_parent_job_id(5);
        assert_eq!(tracker.get_parent_job_id(), Some(5));
        
        // Test cumulative progress
        tracker.base.set_completed_jobs(5);
        assert_eq!(tracker.get_cumulative_progress(), 0.5);
    }
    
    #[test]
    fn test_prioritized_job_impl() {
        let mut tracker = TestTracker::new(10);
        assert_eq!(tracker.get_priority(), 0);
        
        tracker.set_priority(5);
        assert_eq!(tracker.get_priority(), 5);
    }
    
    #[test]
    fn test_dependent_job_impl() {
        let mut tracker = TestTracker::new(10);
        
        // Test dependencies
        assert!(tracker.add_dependency(1));
        assert!(tracker.add_dependency(2));
        
        let deps = tracker.get_dependencies();
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&1));
        assert!(deps.contains(&2));
        
        assert!(tracker.has_dependencies());
        
        assert!(tracker.is_dependency_satisfied(3, false)); // Not a dependency
        assert!(!tracker.is_dependency_satisfied(1, false)); // Dependency not completed
        assert!(tracker.is_dependency_satisfied(1, true)); // Dependency completed
        
        assert!(tracker.remove_dependency(1));
        assert_eq!(tracker.get_dependencies().len(), 1);
    }
    
    #[test]
    fn test_failure_handling_job_impl() {
        let mut tracker = TestTracker::new(10);
        
        // Test failure handling
        assert_eq!(tracker.get_failure_count(), 0);
        assert_eq!(tracker.mark_failed("Test error"), 1);
        assert_eq!(tracker.get_failure_count(), 1);
        assert_eq!(tracker.get_error_message(), Some("Test error".to_string()));
        assert!(tracker.has_failed());
        
        // Test retry
        assert_eq!(tracker.retry(), 1);
        assert_eq!(tracker.get_retry_count(), 1);
        assert!(!tracker.has_failed()); // Retry clears the error state
        
        // Test max retries
        assert_eq!(tracker.get_max_retries(), 3); // Default
        tracker.set_max_retries(2);
        assert_eq!(tracker.get_max_retries(), 2);
        
        // Test retry limit
        assert!(!tracker.has_reached_retry_limit());
        tracker.retry(); // Now at 2 retries with max of 2
        assert!(tracker.has_reached_retry_limit());
        
        // Test mark succeeded
        tracker.mark_succeeded();
        assert_eq!(tracker.get_failure_count(), 0);
        assert_eq!(tracker.get_retry_count(), 0);
        assert!(!tracker.has_failed());
    }
    
    #[test]
    fn test_job_status_tracker_impl() {
        let mut tracker = TestTracker::new(10);
        
        // Test initial status
        assert_eq!(tracker.get_status(), JobStatus::Pending);
        
        // Test setting status
        tracker.mark_running();
        assert_eq!(tracker.get_status(), JobStatus::Running);
        assert!(tracker.is_running());
        
        // Test completing
        tracker.mark_completed();
        assert_eq!(tracker.get_status(), JobStatus::Completed);
        assert!(tracker.is_completed());
        
        // Test status checks
        assert!(tracker.is_in_status(JobStatus::Completed));
        assert!(!tracker.is_in_status(JobStatus::Running));
        
        // Test failure and retry
        tracker.mark_failed("Test error");
        assert_eq!(tracker.get_status(), JobStatus::Failed);
        
        tracker.retry();
        assert_eq!(tracker.get_status(), JobStatus::Retry);
        assert!(tracker.is_retrying());
    }
    
    #[test]
    fn test_time_tracking_job_impl() {
        let mut tracker = TestTracker::new(10);
        
        // Test elapsed time
        let elapsed = tracker.get_elapsed_time();
        assert!(elapsed.as_secs() < 1, "Elapsed time should be < 1 second at the start");
        
        // Initially, no estimates are available
        assert!(tracker.get_estimated_time_remaining().is_none(), "Initial ETA should be None");
        assert!(tracker.get_progress_speed().is_none(), "Initial speed should be None");
        
        // Simulate progress over time
        for i in 1..=5 {
            // Sleep to simulate work
            std::thread::sleep(Duration::from_millis(50));
            
            // Update progress
            tracker.base_config_mut().set_completed_jobs(i);
            let progress = tracker.update_time_estimates();
            
            // Verify progress percentage
            let expected_progress = (i as f64) / 10.0 * 100.0;
            assert!((progress - expected_progress).abs() < 0.01, 
                    "Expected progress: {}, got: {}", expected_progress, progress);
            
            // After several updates, we should have estimates
            if i >= 3 {
                // We should now have speed and ETA
                let speed = tracker.get_progress_speed();
                let eta = tracker.get_estimated_time_remaining();
                
                println!("Progress: {}/10, ETA: {:?}, Speed: {:?}", i, eta, speed);
                
                // After a few updates, we should get some estimates
                if let Some(speed_value) = speed {
                    assert!(speed_value > 0.0, "Speed should be positive: {}", speed_value);
                }
                
                if let Some(eta_duration) = eta {
                    // Ensure ETA is reasonable for this small test
                    assert!(eta_duration.as_secs() < 10, 
                           "ETA should be reasonable: {:?}", eta_duration);
                }
            }
        }
        
        // Complete the job
        tracker.base_config_mut().set_completed_jobs(10);
        tracker.update_time_estimates();
        
        // After completion, elapsed time should be larger
        let final_elapsed = tracker.get_elapsed_time();
        assert!(final_elapsed > elapsed, "Elapsed time should increase");
        
        // ETA should be None or very small after completion
        if let Some(time) = tracker.get_estimated_time_remaining() {
            assert!(time.as_secs() < 1, "ETA should be very small after completion");
        }
    }
    
    #[test]
    fn test_persistent_job_impl() {
        // Create a temporary file path for testing
        let test_dir = std::env::temp_dir().join("nt_progress_test");
        fs::create_dir_all(&test_dir).unwrap();
        let test_file = test_dir.join("test_job_state.json").to_str().unwrap().to_string();
        
        // Clean up from any previous test runs
        if Path::new(&test_file).exists() {
            fs::remove_file(&test_file).unwrap();
        }
        
        // Create a test job with BaseConfig
        let mut job = TestTracker::new(100);
        
        // Set a persistence ID
        job.set_persistence_id("test-job-123".to_string());
        assert!(job.has_persistence_id());
        assert_eq!(job.get_persistence_id(), "test-job-123");
        
        // Modify job state
        job.set_total_jobs(150);
        job.increment_completed_jobs();
        job.increment_completed_jobs();
        job.mark_running();
        job.add_child_job(1);
        job.add_child_job(2);
        job.set_priority(5);
        job.add_dependency(10);
        job.set_max_retries(5);
        
        // Save state
        job.save_state(&test_file).unwrap();
        
        // Verify file exists
        assert!(Path::new(&test_file).exists());
        assert!(BaseConfig::state_exists(&test_file));
        
        // Create a new job and load state
        let mut new_job = TestTracker::new(0);
        
        // Load state into the new job
        new_job.load_state(&test_file).unwrap();
        
        // Verify state was loaded correctly
        assert_eq!(new_job.get_persistence_id(), "test-job-123");
        assert_eq!(new_job.get_total_jobs(), 150);
        assert_eq!(new_job.base_config().get_completed_jobs(), 2);
        assert_eq!(new_job.get_status(), JobStatus::Running);
        assert_eq!(new_job.get_child_job_ids().len(), 2);
        assert!(new_job.get_child_job_ids().contains(&1));
        assert!(new_job.get_child_job_ids().contains(&2));
        assert_eq!(new_job.get_priority(), 5);
        assert_eq!(new_job.get_dependencies().len(), 1);
        assert!(new_job.get_dependencies().contains(&10));
        assert_eq!(new_job.get_max_retries(), 5);
        
        // Clean up
        fs::remove_file(&test_file).unwrap();
        if Path::new(&test_dir).exists() {
            fs::remove_dir_all(&test_dir).unwrap();
        }
    }
} 