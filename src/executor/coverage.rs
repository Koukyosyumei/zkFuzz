use std::hash::{Hash, Hasher};

use rustc_hash::{FxHashMap, FxHashSet, FxHasher};

/// A utility struct for tracking code coverage during program execution, focusing on paths and branches.
///
/// The `CoverageTracker` collects information about the execution paths taken by a program,
/// counts visits to specific branches, and tracks unique paths using a hash-based approach.
///
/// # Fields
/// - `paths`: A set of unique hashed paths (`FxHashSet<u64>`) representing distinct execution flows.
/// - `visit_counter`: A hash map (`FxHashMap<usize, usize>`) tracking the number of visits to each branch,
///   where the key is a branch identifier, and the value is the visit count.
/// - `current_path`: A vector (`Vec<(usize, usize, bool)>`) storing the sequence of branches taken in the current execution path.
///   Each entry is a tuple of the branch ID, visit count, and the branch condition.
///
/// # Methods
/// ## `new`
/// Creates a new, empty `CoverageTracker`.
///
/// ### Returns
/// A `CoverageTracker` instance with all fields initialized to their default values.
///
/// ## `record_branch`
/// Records the occurrence of a branch in the current path.
///
/// ### Parameters
/// - `meta_elem_id`: The identifier of the branch being recorded.
/// - `branch_cond`: A boolean representing the outcome of the branch condition.
///
/// ### Behavior
/// - Increments the visit count for the branch ID in `visit_counter`.
/// - Appends a tuple containing the branch ID, its visit count, and the branch condition to `current_path`.
///
/// ## `record_path`
/// Finalizes and records the current execution path by hashing it and adding the result to `paths`.
///
/// ## `hash_current_path`
/// Computes a hash for the current execution path stored in `current_path`.
///
/// ### Returns
/// A `u64` hash value representing the current path.
///
/// ## `clear`
/// Resets all tracking data, clearing both the recorded paths and the current execution path.
///
/// ## `clear_current_path`
/// Clears only the current execution path and its associated visit counters.
///
/// ## `coverage_count`
/// Returns the total number of unique paths recorded.
///
/// ### Returns
/// The size of the `paths` set, representing the count of unique execution paths.
///
/// # Example
/// ```rust
/// use zkfuzz::executor::coverage::CoverageTracker;
///
/// let mut tracker = CoverageTracker::new();
///
/// tracker.record_branch(1, true);
/// tracker.record_branch(2, false);
/// tracker.record_path();
///
/// assert_eq!(tracker.coverage_count(), 1);
///
/// tracker.clear_current_path();
/// tracker.record_branch(3, true);
/// tracker.record_path();
///
/// assert_eq!(tracker.coverage_count(), 2);
/// tracker.clear();
/// assert_eq!(tracker.coverage_count(), 0);
/// ```
///
/// # Notes
/// - Designed for use in symbolic execution or fuzzing contexts to analyze code coverage.
/// - The `FxHasher` is used for efficient hashing of paths.
#[derive(Clone)]
pub struct CoverageTracker {
    paths: FxHashSet<u64>,
    visit_counter: FxHashMap<usize, usize>,
    current_path: Vec<(usize, usize, bool)>,
}

impl CoverageTracker {
    pub fn new() -> Self {
        CoverageTracker {
            paths: FxHashSet::default(),
            visit_counter: FxHashMap::default(),
            current_path: Vec::new(),
        }
    }

    pub fn record_branch(&mut self, meta_elem_id: usize, branch_cond: bool) {
        *self.visit_counter.entry(meta_elem_id).or_insert(0) += 1;
        self.current_path
            .push((meta_elem_id, self.visit_counter[&meta_elem_id], branch_cond));
    }

    pub fn record_path(&mut self) {
        let path_hash = self.hash_current_path();
        self.paths.insert(path_hash);
    }

    fn hash_current_path(&self) -> u64 {
        let mut hasher = FxHasher::default();
        self.current_path.hash(&mut hasher);
        hasher.finish()
    }

    pub fn clear(&mut self) {
        self.clear_current_path();
        self.paths.clear();
    }

    pub fn clear_current_path(&mut self) {
        self.visit_counter.clear();
        self.current_path.clear();
    }

    pub fn coverage_count(&self) -> usize {
        self.paths.len()
    }
}
