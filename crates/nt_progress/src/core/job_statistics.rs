use std::time::Duration;
use std::fmt;

use super::job_traits::{
    JobTracker, TimeTrackingJob, JobStatusTracker, FailureHandlingJob, 
    HierarchicalJobTracker, CancellableJob
};
use super::base_config::JobStatus;
use crate::config::capabilities::WithProgress;

/// A comprehensive report of job statistics.
#[derive(Debug, Clone)]
pub struct JobStatisticsReport {
    /// Total number of jobs
    pub total_jobs: usize,
    /// Number of completed jobs
    pub completed_jobs: usize,
    /// Current job status
    pub status: JobStatus,
    /// Elapsed time since job started
    pub elapsed_time: Duration,
    /// Estimated time remaining, if available
    pub estimated_time_remaining: Option<Duration>,
    /// Current progress speed in units per second, if available
    pub progress_speed: Option<f64>,
    /// Number of failures encountered
    pub failure_count: usize,
    /// Number of retries performed
    pub retry_count: usize,
    /// Maximum number of retries allowed
    pub max_retries: usize,
    /// Whether the job has been cancelled
    pub is_cancelled: bool,
    /// Parent job ID, if this is a child job
    pub parent_job_id: Option<usize>,
    /// Number of child jobs
    pub child_job_count: usize,
    /// Progress percentage (0-100)
    pub progress_percentage: f64,
}

impl fmt::Display for JobStatisticsReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Job Statistics Report:")?;
        writeln!(f, "  Status: {:?}", self.status)?;
        writeln!(f, "  Progress: {:.1}% ({}/{})", 
            self.progress_percentage * 100.0,
            self.completed_jobs,
            self.total_jobs)?;
        writeln!(f, "  Elapsed Time: {:?}", self.elapsed_time)?;
        if let Some(remaining) = self.estimated_time_remaining {
            writeln!(f, "  Estimated Time Remaining: {:?}", remaining)?;
        }
        if let Some(speed) = self.progress_speed {
            writeln!(f, "  Progress Speed: {:.2} jobs/second", speed)?;
        }
        writeln!(f, "  Failures: {} ({} retries, max: {})", 
            self.failure_count,
            self.retry_count,
            self.max_retries)?;
        writeln!(f, "  Cancelled: {}", self.is_cancelled)?;
        if let Some(parent_id) = self.parent_job_id {
            writeln!(f, "  Parent Job ID: {}", parent_id)?;
        }
        writeln!(f, "  Child Jobs: {}", self.child_job_count)?;
        Ok(())
    }
}

/// A trait for generating comprehensive job statistics.
pub trait JobStatistics: 
    JobTracker + 
    TimeTrackingJob + 
    JobStatusTracker + 
    FailureHandlingJob + 
    HierarchicalJobTracker + 
    CancellableJob +
    WithProgress
{
    /// Generate a comprehensive statistics report for the job.
    fn generate_statistics_report(&self) -> JobStatisticsReport {
        JobStatisticsReport {
            total_jobs: self.get_total_jobs(),
            completed_jobs: self.get_completed_jobs(),
            status: self.get_status(),
            elapsed_time: <Self as TimeTrackingJob>::get_elapsed_time(self),
            estimated_time_remaining: <Self as TimeTrackingJob>::get_estimated_time_remaining(self),
            progress_speed: <Self as TimeTrackingJob>::get_progress_speed(self),
            failure_count: self.get_failure_count(),
            retry_count: self.get_retry_count(),
            max_retries: self.get_max_retries(),
            is_cancelled: self.is_cancelled(),
            parent_job_id: self.get_parent_job_id(),
            child_job_count: self.get_child_job_ids().len(),
            progress_percentage: self.get_completed_jobs() as f64 / self.get_total_jobs() as f64,
        }
    }
    
    /// Get a concise summary of the job's current state.
    fn get_job_summary(&self) -> String {
        let report = self.generate_statistics_report();
        format!(
            "{} - {:.1}% complete ({} jobs) - Elapsed: {:?}, Remaining: {:?}, Speed: {:.2} jobs/s",
            report.status,
            report.progress_percentage * 100.0,
            report.completed_jobs,
            report.elapsed_time,
            report.estimated_time_remaining.unwrap_or(Duration::from_secs(0)),
            report.progress_speed.unwrap_or(0.0)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::base_config::BaseConfig;
    use super::super::job_traits::HasBaseConfig;
    
    #[derive(Debug)]
    struct TestJob {
        base: BaseConfig,
    }
    
    impl TestJob {
        fn new(total_jobs: usize) -> Self {
            Self {
                base: BaseConfig::new(total_jobs),
            }
        }
    }
    
    impl HasBaseConfig for TestJob {
        fn base_config(&self) -> &BaseConfig {
            &self.base
        }
        
        fn base_config_mut(&mut self) -> &mut BaseConfig {
            &mut self.base
        }
    }
    
    impl WithProgress for TestJob {
        fn get_completed_jobs(&self) -> usize {
            self.base.get_completed_jobs()
        }
        
        fn set_progress_format(&mut self, format: &str) {
            self.base.set_progress_format(format)
        }
        
        fn get_progress_format(&self) -> &str {
            self.base.get_progress_format()
        }
    }
    
    impl JobStatistics for TestJob {}
    
    #[test]
    fn test_job_statistics_report() {
        let mut job = TestJob::new(100);
        job.base_config_mut().mark_running();
        job.base_config_mut().increment_completed_jobs();
        job.base_config_mut().increment_completed_jobs();
        
        let report = job.generate_statistics_report();
        assert_eq!(report.total_jobs, 100);
        assert_eq!(report.completed_jobs, 2);
        assert_eq!(report.status, JobStatus::Running);
        assert_eq!(report.child_job_count, 0);
        assert_eq!(report.progress_percentage, 0.02);
        assert_eq!(report.max_retries, 3);
    }
    
    #[test]
    fn test_job_summary() {
        let mut job = TestJob::new(100);
        job.base_config_mut().mark_running();
        job.base_config_mut().increment_completed_jobs();
        job.base_config_mut().increment_completed_jobs();
        
        let summary = job.get_job_summary();
        assert!(summary.contains("Running"));
        assert!(summary.contains("2.0% complete"));
        assert!(summary.contains("2 jobs"));
    }
    
    #[test]
    fn test_job_statistics_with_failures() {
        let mut job = TestJob::new(100);
        job.base_config_mut().mark_failed("Test error");
        job.base_config_mut().retry();
        
        let report = job.generate_statistics_report();
        assert_eq!(report.failure_count, 1);
        assert_eq!(report.retry_count, 1);
    }
    
    #[test]
    fn test_job_statistics_with_cancellation() {
        let mut job = TestJob::new(100);
        job.base_config_mut().mark_failed("Test error");
        job.base_config_mut().set_cancelled(Some("Test cancellation".to_string()));
        
        let report = job.generate_statistics_report();
        assert!(report.is_cancelled);
        assert_eq!(report.status, JobStatus::Failed);
    }
} 