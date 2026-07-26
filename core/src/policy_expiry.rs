use chrono::{DateTime, Duration, Utc};

pub struct ApprovalEntry {
    pub agent_id: String,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl ApprovalEntry {
    pub fn new(agent_id: String, ttl: Option<Duration>) -> Self {
        let now = Utc::now();
        Self {
            agent_id,
            granted_at: now,
            expires_at: ttl.map(|d| now + d),
        }
    }

    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(exp) => Utc::now() >= exp,
            None => false,
        }
    }

    pub fn remaining(&self) -> Option<Duration> {
        self.expires_at.map(|exp| exp - Utc::now())
    }
}

pub struct PolicyAllowList {
    approvals: Vec<ApprovalEntry>,
}

impl PolicyAllowList {
    pub fn new() -> Self {
        Self {
            approvals: Vec::new(),
        }
    }

    pub fn approve(&mut self, agent_id: String, ttl: Option<Duration>) {
        self.approvals
            .retain(|a| a.agent_id != agent_id);
        self.approvals.push(ApprovalEntry::new(agent_id, ttl));
    }

    pub fn revoke(&mut self, agent_id: &str) {
        self.approvals.retain(|a| a.agent_id != agent_id);
    }

    pub fn is_approved(&self, agent_id: &str) -> bool {
        self.approvals
            .iter()
            .any(|a| a.agent_id == agent_id && !a.is_expired())
    }

    pub fn cleanup_expired(&mut self) -> usize {
        let before = self.approvals.len();
        self.approvals.retain(|a| !a.is_expired());
        before - self.approvals.len()
    }
}
