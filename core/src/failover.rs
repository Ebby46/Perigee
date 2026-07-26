pub struct FailoverManager {
    pending_reauth: Vec<String>,
}

impl FailoverManager {
    pub fn new() -> Self {
        Self {
            pending_reauth: Vec::new(),
        }
    }

    pub fn initiate_failover(&mut self, vault_id: &str, new_agent_id: &str) {
        let key = format!("{}:{}", vault_id, new_agent_id);
        if !self.pending_reauth.contains(&key) {
            self.pending_reauth.push(key);
        }
    }

    pub fn confirm_reauth(&mut self, vault_id: &str) -> bool {
        let prefix = format!("{}:", vault_id);
        let initial_len = self.pending_reauth.len();
        self.pending_reauth.retain(|entry| !entry.starts_with(&prefix));
        self.pending_reauth.len() < initial_len
    }

    pub fn can_execute(&self, vault_id: &str) -> bool {
        let prefix = format!("{}:", vault_id);
        !self.pending_reauth.iter().any(|entry| entry.starts_with(&prefix))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_failover_lifecycle() {
        let mut mgr = FailoverManager::new();
        assert!(mgr.can_execute("v1"));
        mgr.initiate_failover("v1", "a1");
        assert!(!mgr.can_execute("v1"));
        assert!(mgr.confirm_reauth("v1"));
        assert!(mgr.can_execute("v1"));
    }

    #[test]
    fn test_confirm_reauth_returns_false_when_no_pending() {
        let mut mgr = FailoverManager::new();
        assert!(!mgr.confirm_reauth("v1"));
    }

    #[test]
    fn test_no_duplicate_pending() {
        let mut mgr = FailoverManager::new();
        mgr.initiate_failover("v1", "a1");
        mgr.initiate_failover("v1", "a1");
        assert_eq!(mgr.pending_reauth.len(), 1);
    }
}
