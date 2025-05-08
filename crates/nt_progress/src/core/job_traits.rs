use std::fmt::Debug;

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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::base_config::BaseConfig;
    
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
} 