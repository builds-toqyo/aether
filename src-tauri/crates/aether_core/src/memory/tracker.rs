//! Memory Allocation Tracker
//! 
//! Provides utilities for tracking memory allocations and detecting leaks.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, atomic::{AtomicU64, AtomicUsize, Ordering}};
use std::time::Instant;

/// Memory allocation record
#[derive(Debug, Clone)]
pub struct AllocationRecord {
    pub id: u64,
    pub size: usize,
    pub category: String,
    pub location: Option<String>,
    pub timestamp: Instant,
}

/// Memory tracker for monitoring allocations
pub struct MemoryTracker {
    allocations: Mutex<HashMap<u64, AllocationRecord>>,
    next_id: AtomicU64,
    total_allocated: AtomicUsize,
    total_freed: AtomicUsize,
    peak_usage: AtomicUsize,
    category_usage: Mutex<HashMap<String, usize>>,
    enabled: bool,
}

impl MemoryTracker {
    pub fn new() -> Self {
        Self {
            allocations: Mutex::new(HashMap::new()),
            next_id: AtomicU64::new(1),
            total_allocated: AtomicUsize::new(0),
            total_freed: AtomicUsize::new(0),
            peak_usage: AtomicUsize::new(0),
            category_usage: Mutex::new(HashMap::new()),
            enabled: true,
        }
    }

    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Self::new()
        }
    }

    /// Record an allocation
    pub fn allocate(&self, size: usize, category: &str) -> u64 {
        self.allocate_with_location(size, category, None)
    }

    /// Record an allocation with source location
    pub fn allocate_with_location(&self, size: usize, category: &str, location: Option<&str>) -> u64 {
        if !self.enabled {
            return 0;
        }

        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        
        let record = AllocationRecord {
            id,
            size,
            category: category.to_string(),
            location: location.map(String::from),
            timestamp: Instant::now(),
        };

        {
            let mut allocations = self.allocations.lock().unwrap();
            allocations.insert(id, record);
        }

        {
            let mut categories = self.category_usage.lock().unwrap();
            *categories.entry(category.to_string()).or_insert(0) += size;
        }

        let new_total = self.total_allocated.fetch_add(size, Ordering::Relaxed) + size;
        let freed = self.total_freed.load(Ordering::Relaxed);
        let current = new_total - freed;

        // Update peak
        let mut peak = self.peak_usage.load(Ordering::Relaxed);
        while current > peak {
            match self.peak_usage.compare_exchange_weak(
                peak, current, Ordering::Relaxed, Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(p) => peak = p,
            }
        }

        id
    }

    /// Record a deallocation
    pub fn free(&self, id: u64) -> Option<usize> {
        if !self.enabled || id == 0 {
            return None;
        }

        let record = {
            let mut allocations = self.allocations.lock().unwrap();
            allocations.remove(&id)
        };

        if let Some(record) = record {
            self.total_freed.fetch_add(record.size, Ordering::Relaxed);

            {
                let mut categories = self.category_usage.lock().unwrap();
                if let Some(usage) = categories.get_mut(&record.category) {
                    *usage = usage.saturating_sub(record.size);
                }
            }

            Some(record.size)
        } else {
            None
        }
    }

    /// Get current memory usage
    pub fn current_usage(&self) -> usize {
        let allocated = self.total_allocated.load(Ordering::Relaxed);
        let freed = self.total_freed.load(Ordering::Relaxed);
        allocated.saturating_sub(freed)
    }

    /// Get peak memory usage
    pub fn peak_usage(&self) -> usize {
        self.peak_usage.load(Ordering::Relaxed)
    }

    /// Get total bytes allocated
    pub fn total_allocated(&self) -> usize {
        self.total_allocated.load(Ordering::Relaxed)
    }

    /// Get total bytes freed
    pub fn total_freed(&self) -> usize {
        self.total_freed.load(Ordering::Relaxed)
    }

    /// Get number of active allocations
    pub fn active_allocations(&self) -> usize {
        self.allocations.lock().unwrap().len()
    }

    /// Get usage by category
    pub fn category_usage(&self) -> HashMap<String, usize> {
        self.category_usage.lock().unwrap().clone()
    }

    /// Get all active allocation records
    pub fn get_allocations(&self) -> Vec<AllocationRecord> {
        self.allocations.lock().unwrap().values().cloned().collect()
    }

    /// Find potential leaks (allocations older than threshold)
    pub fn find_potential_leaks(&self, age_threshold: std::time::Duration) -> Vec<AllocationRecord> {
        let allocations = self.allocations.lock().unwrap();
        allocations.values()
            .filter(|r| r.timestamp.elapsed() > age_threshold)
            .cloned()
            .collect()
    }

    /// Generate a memory report
    pub fn report(&self) -> MemoryReport {
        let allocations = self.allocations.lock().unwrap();
        let categories = self.category_usage.lock().unwrap();

        let mut category_stats: Vec<_> = categories.iter()
            .map(|(name, &size)| CategoryStats {
                name: name.clone(),
                size,
                count: allocations.values().filter(|a| a.category == *name).count(),
            })
            .collect();
        
        category_stats.sort_by(|a, b| b.size.cmp(&a.size));

        MemoryReport {
            current_usage: self.current_usage(),
            peak_usage: self.peak_usage(),
            total_allocated: self.total_allocated(),
            total_freed: self.total_freed(),
            active_allocations: allocations.len(),
            category_stats,
        }
    }

    /// Reset all tracking data
    pub fn reset(&self) {
        self.allocations.lock().unwrap().clear();
        self.total_allocated.store(0, Ordering::Relaxed);
        self.total_freed.store(0, Ordering::Relaxed);
        self.peak_usage.store(0, Ordering::Relaxed);
        self.category_usage.lock().unwrap().clear();
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub name: String,
    pub size: usize,
    pub count: usize,
}

#[derive(Debug)]
pub struct MemoryReport {
    pub current_usage: usize,
    pub peak_usage: usize,
    pub total_allocated: usize,
    pub total_freed: usize,
    pub active_allocations: usize,
    pub category_stats: Vec<CategoryStats>,
}

impl MemoryReport {
    pub fn print(&self) {
        println!("=== Memory Report ===");
        println!("Current usage: {} bytes ({:.2} MB)", 
            self.current_usage, self.current_usage as f64 / 1_048_576.0);
        println!("Peak usage: {} bytes ({:.2} MB)", 
            self.peak_usage, self.peak_usage as f64 / 1_048_576.0);
        println!("Total allocated: {} bytes", self.total_allocated);
        println!("Total freed: {} bytes", self.total_freed);
        println!("Active allocations: {}", self.active_allocations);
        println!();
        println!("By Category:");
        for cat in &self.category_stats {
            println!("  {}: {} bytes ({} allocations)", cat.name, cat.size, cat.count);
        }
    }
}

/// RAII guard for tracked allocations
pub struct TrackedAllocation<'a> {
    id: u64,
    tracker: &'a MemoryTracker,
}

impl<'a> TrackedAllocation<'a> {
    pub fn new(tracker: &'a MemoryTracker, size: usize, category: &str) -> Self {
        let id = tracker.allocate(size, category);
        Self { id, tracker }
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}

impl<'a> Drop for TrackedAllocation<'a> {
    fn drop(&mut self) {
        self.tracker.free(self.id);
    }
}

/// Memory budget manager
pub struct MemoryBudget {
    tracker: Arc<MemoryTracker>,
    budgets: Mutex<HashMap<String, usize>>,
}

impl MemoryBudget {
    pub fn new(tracker: Arc<MemoryTracker>) -> Self {
        Self {
            tracker,
            budgets: Mutex::new(HashMap::new()),
        }
    }

    /// Set a budget for a category
    pub fn set_budget(&self, category: &str, max_bytes: usize) {
        let mut budgets = self.budgets.lock().unwrap();
        budgets.insert(category.to_string(), max_bytes);
    }

    /// Check if allocation would exceed budget
    pub fn can_allocate(&self, category: &str, size: usize) -> bool {
        let budgets = self.budgets.lock().unwrap();
        let categories = self.tracker.category_usage();

        if let Some(&budget) = budgets.get(category) {
            let current = categories.get(category).copied().unwrap_or(0);
            current + size <= budget
        } else {
            true // No budget set
        }
    }

    /// Get remaining budget for a category
    pub fn remaining(&self, category: &str) -> Option<usize> {
        let budgets = self.budgets.lock().unwrap();
        let categories = self.tracker.category_usage();

        budgets.get(category).map(|&budget| {
            let current = categories.get(category).copied().unwrap_or(0);
            budget.saturating_sub(current)
        })
    }

    /// Get usage percentage for a category
    pub fn usage_percent(&self, category: &str) -> Option<f64> {
        let budgets = self.budgets.lock().unwrap();
        let categories = self.tracker.category_usage();

        budgets.get(category).map(|&budget| {
            if budget == 0 {
                return 0.0;
            }
            let current = categories.get(category).copied().unwrap_or(0);
            current as f64 / budget as f64 * 100.0
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_tracker_basic() {
        let tracker = MemoryTracker::new();

        let id1 = tracker.allocate(1024, "test");
        let id2 = tracker.allocate(2048, "test");

        assert_eq!(tracker.current_usage(), 3072);
        assert_eq!(tracker.active_allocations(), 2);

        tracker.free(id1);
        assert_eq!(tracker.current_usage(), 2048);
        assert_eq!(tracker.active_allocations(), 1);

        tracker.free(id2);
        assert_eq!(tracker.current_usage(), 0);
    }

    #[test]
    fn test_peak_usage() {
        let tracker = MemoryTracker::new();

        let id1 = tracker.allocate(1024, "test");
        let id2 = tracker.allocate(2048, "test");
        
        assert_eq!(tracker.peak_usage(), 3072);

        tracker.free(id1);
        tracker.free(id2);

        // Peak should remain
        assert_eq!(tracker.peak_usage(), 3072);
        assert_eq!(tracker.current_usage(), 0);
    }

    #[test]
    fn test_category_tracking() {
        let tracker = MemoryTracker::new();

        tracker.allocate(1024, "video");
        tracker.allocate(2048, "audio");
        tracker.allocate(512, "video");

        let categories = tracker.category_usage();
        assert_eq!(categories.get("video"), Some(&1536));
        assert_eq!(categories.get("audio"), Some(&2048));
    }

    #[test]
    fn test_tracked_allocation_raii() {
        let tracker = MemoryTracker::new();

        {
            let _alloc = TrackedAllocation::new(&tracker, 1024, "test");
            assert_eq!(tracker.current_usage(), 1024);
        }

        assert_eq!(tracker.current_usage(), 0);
    }

    #[test]
    fn test_memory_budget() {
        let tracker = Arc::new(MemoryTracker::new());
        let budget = MemoryBudget::new(Arc::clone(&tracker));

        budget.set_budget("video", 1024);

        assert!(budget.can_allocate("video", 512));
        
        tracker.allocate(512, "video");
        
        assert!(budget.can_allocate("video", 512));
        assert!(!budget.can_allocate("video", 1024));

        assert_eq!(budget.remaining("video"), Some(512));
        assert!((budget.usage_percent("video").unwrap() - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_memory_report() {
        let tracker = MemoryTracker::new();

        tracker.allocate(1024, "video");
        tracker.allocate(2048, "audio");

        let report = tracker.report();
        assert_eq!(report.current_usage, 3072);
        assert_eq!(report.active_allocations, 2);
        assert_eq!(report.category_stats.len(), 2);
    }
}
