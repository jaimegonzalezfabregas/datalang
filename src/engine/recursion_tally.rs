use std::collections::BTreeMap;

use super::RelId;
#[derive(Clone)]
pub struct RecursionTally {
    max_recursion: usize,
    tally: BTreeMap<RelId, usize>,
}

impl RecursionTally {
    pub fn new(max_recursion: usize) -> Self {
        Self {
            max_recursion,
            tally: BTreeMap::new(),
        }
    }

    pub fn go_deeper(&self, rel_id: &RelId) -> bool {
        if let Some(depth_count) = self.tally.get(rel_id) {
            depth_count.to_owned() > 0
        } else {
            self.max_recursion > 0
        }
    }

    pub fn count_up(&mut self, rel_id: &RelId) {
        if let Some(depth_count) = self.tally.get_mut(rel_id) {
            *depth_count -= 1;
        } else {
            self.tally.insert(rel_id.to_owned(), self.max_recursion);
        }
    }
}
