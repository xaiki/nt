use std::collections::HashMap;
use super::bar::ProgressBar;

/// Manages multiple progress bars simultaneously
#[derive(Debug)]
pub struct MultiProgressBar {
    /// Map of ID to progress bars
    bars: HashMap<String, ProgressBar>,
    /// The order in which the bars should be displayed
    order: Vec<String>,
}

impl MultiProgressBar {
    /// Creates a new empty multi-progress bar
    pub fn new() -> Self {
        Self {
            bars: HashMap::new(),
            order: Vec::new(),
        }
    }

    /// Adds a progress bar with the given ID
    ///
    /// # Parameters
    /// * `id` - Unique identifier for the progress bar
    /// * `bar` - The progress bar to add
    ///
    /// # Returns
    /// A reference to self for method chaining
    pub fn add(&mut self, id: impl Into<String>, bar: ProgressBar) -> &mut Self {
        let id = id.into();
        if !self.bars.contains_key(&id) {
            self.order.push(id.clone());
        }
        self.bars.insert(id, bar);
        self
    }

    /// Removes a progress bar with the given ID
    ///
    /// # Parameters
    /// * `id` - The ID of the progress bar to remove
    ///
    /// # Returns
    /// The removed progress bar, if it existed
    pub fn remove(&mut self, id: &str) -> Option<ProgressBar> {
        self.order.retain(|i| i != id);
        self.bars.remove(id)
    }

    /// Gets a reference to a progress bar by ID
    ///
    /// # Parameters
    /// * `id` - The ID of the progress bar to get
    ///
    /// # Returns
    /// An optional reference to the progress bar
    pub fn get(&self, id: &str) -> Option<&ProgressBar> {
        self.bars.get(id)
    }

    /// Gets a mutable reference to a progress bar by ID
    ///
    /// # Parameters
    /// * `id` - The ID of the progress bar to get
    ///
    /// # Returns
    /// An optional mutable reference to the progress bar
    pub fn get_mut(&mut self, id: &str) -> Option<&mut ProgressBar> {
        self.bars.get_mut(id)
    }

    /// Updates the progress of a bar with the given ID
    ///
    /// # Parameters
    /// * `id` - The ID of the progress bar to update
    /// * `progress` - The new progress value (0.0 to 1.0)
    ///
    /// # Returns
    /// A reference to self for method chaining
    pub fn update(&mut self, id: &str, progress: f64) -> &mut Self {
        if let Some(bar) = self.bars.get_mut(id) {
            bar.update(progress);
        }
        self
    }

    /// Updates the progress of a bar with the given ID using current and total values
    ///
    /// # Parameters
    /// * `id` - The ID of the progress bar to update
    /// * `current` - The current value
    /// * `total` - The total value
    ///
    /// # Returns
    /// A reference to self for method chaining
    pub fn update_with_values(&mut self, id: &str, current: usize, total: usize) -> &mut Self {
        if let Some(bar) = self.bars.get_mut(id) {
            bar.update_with_values(current, total);
        }
        self
    }

    /// Gets the number of progress bars
    ///
    /// # Returns
    /// The number of progress bars
    pub fn len(&self) -> usize {
        self.bars.len()
    }

    /// Checks if there are no progress bars
    ///
    /// # Returns
    /// True if there are no progress bars, false otherwise
    pub fn is_empty(&self) -> bool {
        self.bars.is_empty()
    }

    /// Gets all progress bars in the order they were added
    ///
    /// # Returns
    /// A vector of references to all progress bars in order
    pub fn get_all(&self) -> Vec<&ProgressBar> {
        self.order.iter()
            .filter_map(|id| self.bars.get(id))
            .collect()
    }

    /// Renders all progress bars to a string
    ///
    /// # Returns
    /// A string containing all progress bars rendered in order
    pub fn render(&self) -> String {
        let mut output = String::new();
        for id in &self.order {
            if let Some(bar) = self.bars.get(id) {
                output.push_str(&format!("{}\n", bar.template()));
            }
        }
        output
    }
}

impl Default for MultiProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::progress_bar::{ProgressBarConfig, ProgressBarStyle};

    #[test]
    fn test_multi_progress_bar_basic() {
        let mut multi = MultiProgressBar::new();
        
        let bar1 = ProgressBar::with_defaults();
        let bar2 = ProgressBar::with_defaults();
        
        multi.add("task1", bar1)
             .add("task2", bar2);
        
        assert_eq!(multi.len(), 2);
        assert!(!multi.is_empty());
        
        multi.update("task1", 0.3)
             .update("task2", 0.7);
        
        assert_eq!(multi.get("task1").unwrap().progress(), 0.3);
        assert_eq!(multi.get("task2").unwrap().progress(), 0.7);
        
        let removed = multi.remove("task1");
        assert!(removed.is_some());
        assert_eq!(multi.len(), 1);
    }

    #[test]
    fn test_multi_progress_bar_rendering() {
        let mut multi = MultiProgressBar::new();
        
        let bar1 = ProgressBar::new(
            ProgressBarConfig::new()
                .prefix("Task 1")
                .width(10)
                .style(ProgressBarStyle::Standard)
        );
        
        let bar2 = ProgressBar::new(
            ProgressBarConfig::new()
                .prefix("Task 2")
                .width(10)
                .style(ProgressBarStyle::Block)
        );
        
        multi.add("task1", bar1)
             .add("task2", bar2);
        
        multi.update("task1", 0.5)
             .update("task2", 0.8);
        
        let rendered = multi.render();
        assert!(rendered.contains("Task 1"));
        assert!(rendered.contains("Task 2"));
    }

    #[test]
    fn test_multi_progress_bar_order() {
        let mut multi = MultiProgressBar::new();
        
        let bar1 = ProgressBar::with_defaults();
        let bar2 = ProgressBar::with_defaults();
        let bar3 = ProgressBar::with_defaults();
        
        // Add in a specific order
        multi.add("task2", bar2)
             .add("task1", bar1)
             .add("task3", bar3);
        
        // Order should be preserved
        let all_bars = multi.get_all();
        assert_eq!(all_bars.len(), 3);
        
        // Remove and add again
        multi.remove("task2");
        multi.add("task2", ProgressBar::with_defaults());
        
        // Now task2 should be at the end
        let order_ids: Vec<&String> = multi.order.iter().collect();
        assert_eq!(order_ids, vec!["task1", "task3", "task2"]);
    }
} 