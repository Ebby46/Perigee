//! Fix for #81: replace sequential, enumerable client IDs with
//! unguessable UUIDs and cursor-based pagination for reconciliation.
use std::collections::HashMap;

pub struct ReconciliationBook {
    entries: HashMap<String, u64>,
    order: Vec<String>,
}
impl ReconciliationBook {
    pub fn new() -> Self {
        Self { entries: HashMap::new(), order: Vec::new() }
    }

    /// Caller supplies an unguessable UUIDv4, not a sequential integer.
    pub fn add_client(&mut self, id: &str, balance: u64) {
        self.order.push(id.to_string());
        self.entries.insert(id.to_string(), balance);
    }

    /// Cursor-based page: opaque `after` is the last-seen client id.
    pub fn page(&self, after: Option<&str>, limit: usize) -> Vec<(String, u64)> {
        let start = after
            .and_then(|id| self.order.iter().position(|c| c == id))
            .map(|i| i + 1)
            .unwrap_or(0);
        self.order[start..]
            .iter()
            .take(limit)
            .map(|id| (id.clone(), self.entries[id]))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pagination_uses_opaque_cursor_not_offset() {
        let mut book = ReconciliationBook::new();
        book.add_client("uuid-a", 10);
        book.add_client("uuid-b", 20);
        book.add_client("uuid-c", 30);
        let first_page = book.page(None, 2);
        assert_eq!(first_page.len(), 2);
        let cursor = &first_page.last().unwrap().0;
        let second_page = book.page(Some(cursor), 2);
        assert_eq!(second_page, vec![("uuid-c".to_string(), 30)]);
    }
}
