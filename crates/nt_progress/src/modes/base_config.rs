use std::fmt::Debug;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, AtomicBool, AtomicU32};
use std::sync::Mutex;

use super::job_traits::HasBaseConfig;

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
} 