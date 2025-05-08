use std::fmt::Debug;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, AtomicBool, AtomicU32};
use std::sync::Mutex;

use super::job_traits::HasBaseConfig;

/// Represents the current status of a job.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    /// Job is waiting to be started
    Pending,
    /// Job is currently running
    Running,
    /// Job has completed successfully
    Completed,
    /// Job has failed
    Failed,
    /// Job is being retried after a failure
    Retry,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "Pending"),
            JobStatus::Running => write!(f, "Running"),
            JobStatus::Completed => write!(f, "Completed"),
            JobStatus::Failed => write!(f, "Failed"),
            JobStatus::Retry => write!(f, "Retry"),
        }
    }
}

/// Base configuration for progress tracking shared across different display modes.
///
/// This struct provides core functionality for tracking job progress
/// and managing job relationships, used as a base component in display modes.
#[derive(Debug, Clone)]
pub struct BaseConfig {
    /// Total number of jobs
    total_jobs: usize,
    /// Counter for completed jobs
    completed_jobs: Arc<AtomicUsize>,
    /// Format string for displaying progress
    progress_format: String,
    /// Parent job ID if this job is a child
    parent_job_id: Option<usize>,
    /// Child job IDs if this job has children
    child_job_ids: Arc<Mutex<Vec<usize>>>,
    /// Whether this job is currently paused
    paused: Arc<AtomicBool>,
    /// Priority of this job (higher values indicate higher priority)
    priority: Arc<AtomicU32>,
    /// Job IDs that this job depends on
    dependencies: Arc<Mutex<Vec<usize>>>,
    /// Number of times this job has failed
    failure_count: Arc<AtomicUsize>,
    /// Most recent error message
    error_message: Arc<Mutex<Option<String>>>,
    /// Number of retries performed
    retry_count: Arc<AtomicUsize>,
    /// Maximum number of retries allowed
    max_retries: Arc<AtomicUsize>,
    /// Current status of the job
    status: Arc<Mutex<JobStatus>>,
}

impl BaseConfig {
    /// Creates a new BaseConfig with the specified number of total jobs.
    ///
    /// # Parameters
    /// * `total_jobs` - The total number of jobs to track
    ///
    /// # Returns
    /// A new BaseConfig instance
    pub fn new(total_jobs: usize) -> Self {
        Self {
            total_jobs,
            completed_jobs: Arc::new(AtomicUsize::new(0)),
            progress_format: "{completed}/{total} ({percent}%)".to_string(),
            parent_job_id: None,
            child_job_ids: Arc::new(Mutex::new(Vec::new())),
            paused: Arc::new(AtomicBool::new(false)),
            priority: Arc::new(AtomicU32::new(0)),
            dependencies: Arc::new(Mutex::new(Vec::new())),
            failure_count: Arc::new(AtomicUsize::new(0)),
            error_message: Arc::new(Mutex::new(None)),
            retry_count: Arc::new(AtomicUsize::new(0)),
            max_retries: Arc::new(AtomicUsize::new(3)), // Default to 3 retries
            status: Arc::new(Mutex::new(JobStatus::Pending)),
        }
    }
    
    /// Get the total number of jobs.
    ///
    /// # Returns
    /// The total number of jobs
    pub fn get_total_jobs(&self) -> usize {
        self.total_jobs
    }
    
    /// Increment the completed jobs counter and return the new value.
    ///
    /// # Returns
    /// The new count of completed jobs
    pub fn increment_completed_jobs(&self) -> usize {
        self.completed_jobs.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1
    }
    
    /// Set the total number of jobs.
    ///
    /// # Parameters
    /// * `total` - The new total number of jobs
    pub fn set_total_jobs(&mut self, total: usize) {
        self.total_jobs = total;
    }
    
    /// Get the number of completed jobs.
    ///
    /// # Returns
    /// The current count of completed jobs
    pub fn get_completed_jobs(&self) -> usize {
        self.completed_jobs.load(std::sync::atomic::Ordering::SeqCst)
    }
    
    /// Set the number of completed jobs.
    ///
    /// # Parameters
    /// * `completed` - The new count of completed jobs
    ///
    /// # Returns
    /// The new count of completed jobs
    pub fn set_completed_jobs(&mut self, completed: usize) -> usize {
        self.completed_jobs.store(completed, std::sync::atomic::Ordering::SeqCst);
        completed
    }
    
    /// Get the progress format string.
    ///
    /// # Returns
    /// The current progress format string
    pub fn get_progress_format(&self) -> &str {
        &self.progress_format
    }
    
    /// Set the progress format string.
    ///
    /// # Parameters
    /// * `format` - The new progress format string
    pub fn set_progress_format(&mut self, format: &str) {
        self.progress_format = format.to_string();
    }
    
    /// Add a child job to this job.
    ///
    /// # Parameters
    /// * `child_id` - The ID of the child job
    ///
    /// # Returns
    /// `true` if the child was successfully added, `false` otherwise
    pub fn add_child_job(&mut self, child_id: usize) -> bool {
        let mut children = self.child_job_ids.lock().unwrap();
        if !children.contains(&child_id) {
            children.push(child_id);
            true
        } else {
            false
        }
    }
    
    /// Remove a child job from this job.
    ///
    /// # Parameters
    /// * `child_id` - The ID of the child job to remove
    ///
    /// # Returns
    /// `true` if the child was successfully removed, `false` otherwise
    pub fn remove_child_job(&mut self, child_id: usize) -> bool {
        let mut children = self.child_job_ids.lock().unwrap();
        if let Some(pos) = children.iter().position(|&id| id == child_id) {
            children.remove(pos);
            true
        } else {
            false
        }
    }
    
    /// Get the parent job ID if this job has a parent.
    ///
    /// # Returns
    /// The parent job ID, or `None` if this job has no parent
    pub fn get_parent_job_id(&self) -> Option<usize> {
        self.parent_job_id
    }
    
    /// Set the parent job ID for this job.
    ///
    /// # Parameters
    /// * `parent_id` - The ID of the parent job
    pub fn set_parent_job_id(&mut self, parent_id: usize) {
        self.parent_job_id = Some(parent_id);
    }
    
    /// Get the list of child job IDs associated with this job.
    ///
    /// # Returns
    /// A vector of child job IDs
    pub fn get_child_job_ids(&self) -> Vec<usize> {
        self.child_job_ids.lock().unwrap().clone()
    }
    
    /// Pause this job.
    pub fn pause(&self) {
        self.paused.store(true, std::sync::atomic::Ordering::SeqCst);
    }
    
    /// Resume this job.
    pub fn resume(&self) {
        self.paused.store(false, std::sync::atomic::Ordering::SeqCst);
    }
    
    /// Check if this job is currently paused.
    ///
    /// # Returns
    /// `true` if the job is paused, `false` otherwise
    pub fn is_paused(&self) -> bool {
        self.paused.load(std::sync::atomic::Ordering::SeqCst)
    }
    
    /// Get the priority of this job.
    ///
    /// # Returns
    /// The priority value
    pub fn get_priority(&self) -> u32 {
        self.priority.load(std::sync::atomic::Ordering::SeqCst)
    }
    
    /// Set the priority of this job.
    ///
    /// # Parameters
    /// * `priority` - The new priority value
    pub fn set_priority(&mut self, priority: u32) {
        self.priority.store(priority, std::sync::atomic::Ordering::SeqCst);
    }
    
    /// Add a dependency on another job.
    ///
    /// # Parameters
    /// * `job_id` - The ID of the job this job depends on
    ///
    /// # Returns
    /// `true` if the dependency was successfully added, `false` otherwise
    pub fn add_dependency(&mut self, job_id: usize) -> bool {
        let mut deps = self.dependencies.lock().unwrap();
        if !deps.contains(&job_id) {
            deps.push(job_id);
            true
        } else {
            false
        }
    }
    
    /// Remove a dependency.
    ///
    /// # Parameters
    /// * `job_id` - The ID of the dependency to remove
    ///
    /// # Returns
    /// `true` if the dependency was successfully removed, `false` otherwise
    pub fn remove_dependency(&mut self, job_id: usize) -> bool {
        let mut deps = self.dependencies.lock().unwrap();
        if let Some(pos) = deps.iter().position(|&id| id == job_id) {
            deps.remove(pos);
            true
        } else {
            false
        }
    }
    
    /// Get the dependencies of this job.
    ///
    /// # Returns
    /// A vector of job IDs that this job depends on
    pub fn get_dependencies(&self) -> Vec<usize> {
        self.dependencies.lock().unwrap().clone()
    }
    
    /// Check if this job has dependencies.
    ///
    /// # Returns
    /// `true` if this job has one or more dependencies, `false` otherwise
    pub fn has_dependencies(&self) -> bool {
        !self.dependencies.lock().unwrap().is_empty()
    }
    
    /// Check if all dependencies are satisfied.
    ///
    /// # Parameters
    /// * `is_completed` - A function that takes a job ID and returns true if the job is completed
    ///
    /// # Returns
    /// `true` if all dependencies are satisfied, `false` otherwise
    pub fn are_dependencies_satisfied<F>(&self, is_completed: F) -> bool
    where
        F: Fn(usize) -> bool,
    {
        let deps = self.dependencies.lock().unwrap();
        deps.iter().all(|&job_id| is_completed(job_id))
    }
    
    /// Check if a specific dependency is satisfied.
    ///
    /// # Parameters
    /// * `job_id` - The job ID to check
    /// * `is_completed` - Whether the job is completed
    ///
    /// # Returns
    /// `true` if the dependency is satisfied, `false` otherwise
    pub fn is_dependency_satisfied(&self, job_id: usize, is_completed: bool) -> bool {
        let deps = self.dependencies.lock().unwrap();
        !deps.contains(&job_id) || is_completed
    }
    
    /// Get the current status of the job.
    ///
    /// # Returns
    /// The current job status
    pub fn get_status(&self) -> JobStatus {
        *self.status.lock().unwrap()
    }
    
    /// Set the job status.
    ///
    /// # Parameters
    /// * `status` - The new job status
    pub fn set_status(&mut self, status: JobStatus) {
        *self.status.lock().unwrap() = status;
    }
    
    /// Mark the job as running.
    ///
    /// This sets the status to Running.
    pub fn mark_running(&mut self) {
        self.set_status(JobStatus::Running);
    }
    
    /// Mark the job as completed.
    ///
    /// This sets the status to Completed and calls mark_succeeded().
    pub fn mark_completed(&mut self) {
        self.set_status(JobStatus::Completed);
        self.mark_succeeded();
    }
    
    /// Mark the job as failed.
    ///
    /// This sets the status to Failed, increments the failure count, and stores the error message.
    ///
    /// # Parameters
    /// * `error` - The error message describing the failure
    ///
    /// # Returns
    /// The current number of failures for this job
    pub fn mark_failed(&mut self, error: &str) -> usize {
        self.set_status(JobStatus::Failed);
        let count = self.failure_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
        *self.error_message.lock().unwrap() = Some(error.to_string());
        count
    }
    
    /// Retry the job.
    ///
    /// This sets the status to Retry, clears the error message, and increments the retry count.
    ///
    /// # Returns
    /// The current retry count
    pub fn retry(&mut self) -> usize {
        self.set_status(JobStatus::Retry);
        
        // Clear the error message but keep failure count for history
        *self.error_message.lock().unwrap() = None;
        
        // Increment retry count
        let count = self.retry_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
        count
    }
    
    /// Check if the job is in the specified status.
    ///
    /// # Parameters
    /// * `status` - The status to check against
    ///
    /// # Returns
    /// `true` if the job is in the specified status, `false` otherwise
    pub fn is_in_status(&self, status: JobStatus) -> bool {
        self.get_status() == status
    }
    
    /// Check if the job is pending.
    ///
    /// # Returns
    /// `true` if the job is pending, `false` otherwise
    pub fn is_pending(&self) -> bool {
        self.is_in_status(JobStatus::Pending)
    }
    
    /// Check if the job is running.
    ///
    /// # Returns
    /// `true` if the job is running, `false` otherwise
    pub fn is_running(&self) -> bool {
        self.is_in_status(JobStatus::Running)
    }
    
    /// Check if the job is completed.
    ///
    /// # Returns
    /// `true` if the job is completed, `false` otherwise
    pub fn is_completed(&self) -> bool {
        self.is_in_status(JobStatus::Completed)
    }
    
    /// Check if the job is in retry state.
    ///
    /// # Returns
    /// `true` if the job is in retry state, `false` otherwise
    pub fn is_retrying(&self) -> bool {
        self.is_in_status(JobStatus::Retry)
    }
    
    /// Mark the job as succeeded.
    ///
    /// This resets the failure count, retry count, and clears any error messages.
    /// If the job is not already in Completed status, it will be set to Running.
    pub fn mark_succeeded(&mut self) {
        if !self.is_completed() {
            self.set_status(JobStatus::Running);
        }
        self.failure_count.store(0, std::sync::atomic::Ordering::SeqCst);
        self.retry_count.store(0, std::sync::atomic::Ordering::SeqCst);
        *self.error_message.lock().unwrap() = None;
    }
    
    /// Get the number of times this job has failed.
    ///
    /// # Returns
    /// The number of failures
    pub fn get_failure_count(&self) -> usize {
        self.failure_count.load(std::sync::atomic::Ordering::SeqCst)
    }
    
    /// Get the most recent error message, if any.
    ///
    /// # Returns
    /// The most recent error message, or None if the job hasn't failed
    pub fn get_error_message(&self) -> Option<String> {
        self.error_message.lock().unwrap().clone()
    }
    
    /// Check if the job has failed.
    ///
    /// # Returns
    /// `true` if the job has failed, `false` otherwise
    pub fn has_failed(&self) -> bool {
        self.get_failure_count() > 0 && self.error_message.lock().unwrap().is_some()
    }
    
    /// Get the number of times this job has been retried.
    ///
    /// # Returns
    /// The number of retries
    pub fn get_retry_count(&self) -> usize {
        self.retry_count.load(std::sync::atomic::Ordering::SeqCst)
    }
    
    /// Set the maximum number of retries allowed for this job.
    ///
    /// # Parameters
    /// * `max_retries` - The maximum number of retries allowed
    pub fn set_max_retries(&mut self, max_retries: usize) {
        self.max_retries.store(max_retries, std::sync::atomic::Ordering::SeqCst);
    }
    
    /// Get the maximum number of retries allowed for this job.
    ///
    /// # Returns
    /// The maximum number of retries allowed
    pub fn get_max_retries(&self) -> usize {
        self.max_retries.load(std::sync::atomic::Ordering::SeqCst)
    }
    
    /// Check if the job has reached its maximum retry limit.
    ///
    /// # Returns
    /// `true` if the job has reached its retry limit, `false` otherwise
    pub fn has_reached_retry_limit(&self) -> bool {
        self.get_retry_count() >= self.get_max_retries()
    }
}

impl HasBaseConfig for BaseConfig {
    fn base_config(&self) -> &BaseConfig {
        self
    }
    
    fn base_config_mut(&mut self) -> &mut BaseConfig {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_base_config_creation() {
        let base = BaseConfig::new(10);
        assert_eq!(base.get_total_jobs(), 10);
        assert_eq!(base.get_completed_jobs(), 0);
    }
    
    #[test]
    fn test_base_config_job_tracking() {
        let mut base = BaseConfig::new(10);
        assert_eq!(base.increment_completed_jobs(), 1);
        assert_eq!(base.increment_completed_jobs(), 2);
        assert_eq!(base.get_completed_jobs(), 2);
        
        base.set_total_jobs(20);
        assert_eq!(base.get_total_jobs(), 20);
        
        base.set_completed_jobs(15);
        assert_eq!(base.get_completed_jobs(), 15);
    }
    
    #[test]
    fn test_base_config_child_jobs() {
        let mut base = BaseConfig::new(10);
        assert!(base.add_child_job(1));
        assert!(base.add_child_job(2));
        assert!(!base.add_child_job(1)); // Already exists
        
        let children = base.get_child_job_ids();
        assert_eq!(children.len(), 2);
        assert!(children.contains(&1));
        assert!(children.contains(&2));
        
        assert!(base.remove_child_job(1));
        assert!(!base.remove_child_job(3)); // Doesn't exist
        
        let children = base.get_child_job_ids();
        assert_eq!(children.len(), 1);
        assert!(children.contains(&2));
    }
    
    #[test]
    fn test_base_config_pause_resume() {
        let base = BaseConfig::new(10);
        assert!(!base.is_paused());
        
        base.pause();
        assert!(base.is_paused());
        
        base.resume();
        assert!(!base.is_paused());
    }
    
    #[test]
    fn test_base_config_priority() {
        let mut base = BaseConfig::new(10);
        assert_eq!(base.get_priority(), 0);
        
        base.set_priority(5);
        assert_eq!(base.get_priority(), 5);
    }
    
    #[test]
    fn test_base_config_dependencies() {
        let mut base = BaseConfig::new(10);
        assert!(base.add_dependency(1));
        assert!(base.add_dependency(2));
        assert!(!base.add_dependency(1)); // Already exists
        
        let deps = base.get_dependencies();
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&1));
        assert!(deps.contains(&2));
        
        assert!(base.has_dependencies());
        
        assert!(base.remove_dependency(1));
        assert!(!base.remove_dependency(3)); // Doesn't exist
        
        let deps = base.get_dependencies();
        assert_eq!(deps.len(), 1);
        assert!(deps.contains(&2));
        
        let all_completed = |job_id: usize| job_id == 2;
        assert!(base.are_dependencies_satisfied(all_completed));
        
        assert!(base.is_dependency_satisfied(2, true));
        assert!(!base.is_dependency_satisfied(2, false));
        assert!(base.is_dependency_satisfied(3, false)); // Not a dependency
    }
    
    #[test]
    fn test_base_config_failure() {
        let mut base = BaseConfig::new(10);
        assert_eq!(base.get_failure_count(), 0);
        
        base.mark_failed("Test error");
        assert_eq!(base.get_failure_count(), 1);
        
        base.mark_succeeded();
        assert_eq!(base.get_failure_count(), 0);
        
        assert!(!base.has_failed());
        
        base.mark_failed("Another test error");
        assert_eq!(base.get_failure_count(), 1);
        
        assert!(base.has_failed());
    }
    
    #[test]
    fn test_base_config_retry() {
        let mut base = BaseConfig::new(10);
        assert_eq!(base.get_retry_count(), 0);
        
        base.retry();
        assert_eq!(base.get_retry_count(), 1);
        
        base.retry();
        assert_eq!(base.get_retry_count(), 2);
    }
    
    #[test]
    fn test_base_config_max_retries() {
        let mut base = BaseConfig::new(10);
        assert_eq!(base.get_max_retries(), 3);
        
        base.set_max_retries(5);
        assert_eq!(base.get_max_retries(), 5);
    }
    
    #[test]
    fn test_base_config_retry_limit() {
        let mut base = BaseConfig::new(10);
        assert!(!base.has_reached_retry_limit());
        
        base.retry();
        assert!(!base.has_reached_retry_limit());
        
        base.retry();
        assert!(!base.has_reached_retry_limit());
        
        base.retry();
        assert!(base.has_reached_retry_limit());
    }
    
    #[test]
    fn test_base_config_job_status() {
        let mut base = BaseConfig::new(10);
        
        // Test initial status
        assert_eq!(base.get_status(), JobStatus::Pending);
        
        // Test setting status
        base.set_status(JobStatus::Running);
        assert_eq!(base.get_status(), JobStatus::Running);
        
        // Test status checks
        assert!(base.is_in_status(JobStatus::Running));
        assert!(base.is_running());
        assert!(!base.is_completed());
        
        // Test mark functions
        base.mark_completed();
        assert_eq!(base.get_status(), JobStatus::Completed);
        assert!(base.is_completed());
        
        // Test failure status
        base.mark_failed("Test error");
        assert_eq!(base.get_status(), JobStatus::Failed);
        assert!(!base.is_running());
        assert!(!base.is_completed());
        
        // Test retry status
        base.retry();
        assert_eq!(base.get_status(), JobStatus::Retry);
        assert!(base.is_retrying());
        
        // Test returning to running state
        base.mark_running();
        assert_eq!(base.get_status(), JobStatus::Running);
    }
    
    #[test]
    fn test_base_config_status_to_string() {
        assert_eq!(JobStatus::Pending.to_string(), "Pending");
        assert_eq!(JobStatus::Running.to_string(), "Running");
        assert_eq!(JobStatus::Completed.to_string(), "Completed");
        assert_eq!(JobStatus::Failed.to_string(), "Failed");
        assert_eq!(JobStatus::Retry.to_string(), "Retry");
    }
} 