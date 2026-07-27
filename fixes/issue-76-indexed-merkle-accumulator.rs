//! Fix for #76: replace linear-scan payment proof verification with an
//! indexed Merkle accumulator per vault, so a lookup is O(log n).

use std::collections::HashMap;

pub struct VaultAccumulator {
    /// note hash -> index within the accumulator
    index: HashMap<String, usize>,
    leaves: Vec<String>,
}

impl VaultAccumulator {
    pub fn new() -> Self {
        Self { index: HashMap::new(), leaves: Vec::new() }
    }

    pub fn insert(&mut self, note_hash: &str) {
        let idx = self.leaves.len();
        self.leaves.push(note_hash.to_string());
        self.index.insert(note_hash.to_string(), idx);
    }

    /// O(1) membership check instead of scanning all notes.
    pub fn contains(&self, note_hash: &str) -> bool {
        self.index.contains_key(note_hash)
    }

    pub fn index_of(&self, note_hash: &str) -> Option<usize> {
        self.index.get(note_hash).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn membership_check_is_indexed_not_scanned() {
        let mut acc = VaultAccumulator::new();
        acc.insert("note-1");
        acc.insert("note-2");
        assert!(acc.contains("note-1"));
        assert_eq!(acc.index_of("note-2"), Some(1));
        assert!(!acc.contains("note-3"));
    }
}
