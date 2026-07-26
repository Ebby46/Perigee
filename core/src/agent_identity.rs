use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIdentity {
    pub agent_id: String,
    pub reputation_score: u32,
    pub is_active: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl AgentIdentity {
    pub fn new(agent_id: String) -> Self {
        Self {
            agent_id,
            reputation_score: 0,
            is_active: true,
            revoked_at: None,
            created_at: Utc::now(),
        }
    }

    pub fn revoke(&mut self) {
        self.is_active = false;
        self.revoked_at = Some(Utc::now());
    }

    pub fn is_revoked(&self) -> bool {
        !self.is_active && self.revoked_at.is_some()
    }

    pub fn was_revoked_before(&self, dt: DateTime<Utc>) -> bool {
        match self.revoked_at {
            Some(revoked) => revoked < dt,
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_identity() {
        let id = AgentIdentity::new("agent-1".to_string());
        assert_eq!(id.agent_id, "agent-1");
        assert!(id.is_active);
        assert!(id.revoked_at.is_none());
        assert_eq!(id.reputation_score, 0);
    }

    #[test]
    fn test_revoke() {
        let mut id = AgentIdentity::new("agent-1".to_string());
        assert!(!id.is_revoked());
        id.revoke();
        assert!(id.is_revoked());
        assert!(!id.is_active);
        assert!(id.revoked_at.is_some());
    }

    #[test]
    fn test_was_revoked_before() {
        let mut id = AgentIdentity::new("agent-1".to_string());
        let now = Utc::now();
        assert!(!id.was_revoked_before(now));
        id.revoke();
        let future = Utc::now() + chrono::Duration::hours(1);
        assert!(id.was_revoked_before(future));
        let past = Utc::now() - chrono::Duration::hours(1);
        assert!(!id.was_revoked_before(past));
    }
}
