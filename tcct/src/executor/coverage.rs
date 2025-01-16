use std::hash::{Hash, Hasher};

use rustc_hash::{FxHashMap, FxHashSet, FxHasher};

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
