#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationJournal {
    pub vault_id: String,
    pub from_agent: String,
    pub to_agent: String,
    pub step: RotationStep,
    pub completed_steps: Vec<RotationStep>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RotationStep {
    PolicyRevoke,
    PolicyGrant,
    VaultReassign,
    StateSnapshot,
    Confirm,
}

const ALL_STEPS: &[RotationStep] = &[
    RotationStep::PolicyRevoke,
    RotationStep::PolicyGrant,
    RotationStep::VaultReassign,
    RotationStep::StateSnapshot,
    RotationStep::Confirm,
];

impl RotationJournal {
    pub fn new(vault_id: String, from: String, to: String) -> Self {
        Self {
            vault_id,
            from_agent: from,
            to_agent: to,
            step: RotationStep::PolicyRevoke,
            completed_steps: Vec::new(),
        }
    }

    pub fn can_resume(&self) -> bool {
        !self.is_complete()
    }

    pub fn next_step(&self) -> Option<RotationStep> {
        if self.is_complete() {
            return None;
        }
        Some(ALL_STEPS[self.completed_steps.len()].clone())
    }

    pub fn complete_step(&mut self, step: RotationStep) {
        if self.next_step() == Some(step.clone()) {
            self.completed_steps.push(step.clone());
            if let Some(idx) = ALL_STEPS.iter().position(|s| s == &step) {
                let next_idx = idx + 1;
                if next_idx < ALL_STEPS.len() {
                    self.step = ALL_STEPS[next_idx].clone();
                }
            }
        }
    }

    pub fn is_complete(&self) -> bool {
        self.completed_steps.len() >= ALL_STEPS.len()
    }

    pub fn progress_pct(&self) -> f64 {
        (self.completed_steps.len() as f64 / ALL_STEPS.len() as f64) * 100.0
    }
}

pub struct RotationJournalStore {
    journals: Vec<RotationJournal>,
}

impl RotationJournalStore {
    pub fn new() -> Self {
        Self {
            journals: Vec::new(),
        }
    }

    pub fn save(&mut self, journal: RotationJournal) {
        if let Some(existing) = self
            .journals
            .iter_mut()
            .find(|j| j.vault_id == journal.vault_id)
        {
            *existing = journal;
        } else {
            self.journals.push(journal);
        }
    }

    pub fn get_incomplete(&self) -> Vec<&RotationJournal> {
        self.journals
            .iter()
            .filter(|j| j.can_resume())
            .collect()
    }

    pub fn resume(&mut self, vault_id: &str) -> Option<&mut RotationJournal> {
        self.journals
            .iter_mut()
            .find(|j| j.vault_id == vault_id && j.can_resume())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_journal_lifecycle() {
        let mut journal =
            RotationJournal::new("vault1".to_string(), "agent_a".to_string(), "agent_b".to_string());
        assert_eq!(journal.progress_pct(), 0.0);

        let steps = vec![
            RotationStep::PolicyRevoke,
            RotationStep::PolicyGrant,
            RotationStep::VaultReassign,
            RotationStep::StateSnapshot,
            RotationStep::Confirm,
        ];

        for step in &steps {
            assert_eq!(journal.next_step(), Some(step.clone()));
            journal.complete_step(step.clone());
        }

        assert!(journal.is_complete());
        assert!(journal.next_step().is_none());
        assert_eq!(journal.progress_pct(), 100.0);
    }

    #[test]
    fn test_store_resume() {
        let mut store = RotationJournalStore::new();
        let journal =
            RotationJournal::new("vault1".to_string(), "a".to_string(), "b".to_string());
        store.save(journal);
        assert_eq!(store.get_incomplete().len(), 1);
        assert!(store.resume("vault1").is_some());
    }
}
