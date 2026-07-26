use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyState {
    pub vault_id: String,
    pub current_phase: String,
    pub last_evaluated_at: DateTime<Utc>,
    pub last_trigger: Option<String>,
    pub evaluation_count: u64,
}

pub struct StrategyStateManager {
    states: HashMap<String, StrategyState>,
}

impl StrategyStateManager {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    pub fn save_state(&mut self, state: StrategyState) {
        self.states.insert(state.vault_id.clone(), state);
    }

    pub fn load_state(&self, vault_id: &str) -> Option<&StrategyState> {
        self.states.get(vault_id)
    }

    pub fn recover_or_default(&self, vault_id: &str) -> StrategyState {
        self.states
            .get(vault_id)
            .cloned()
            .unwrap_or_else(|| StrategyState {
                vault_id: vault_id.to_string(),
                current_phase: String::from("default"),
                last_evaluated_at: Utc::now(),
                last_trigger: None,
                evaluation_count: 0,
            })
    }

    pub fn persist_to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(&self.states)
            .expect("serialization of StrategyState map should not fail")
    }

    pub fn restore_from_bytes(data: &[u8]) -> Result<Self, String> {
        let states: HashMap<String, StrategyState> =
            serde_json::from_slice(data).map_err(|e| e.to_string())?;
        Ok(Self { states })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_load() {
        let mut mgr = StrategyStateManager::new();
        let state = StrategyState {
            vault_id: "v1".into(),
            current_phase: "active".into(),
            last_evaluated_at: Utc::now(),
            last_trigger: None,
            evaluation_count: 5,
        };
        mgr.save_state(state.clone());
        let loaded = mgr.load_state("v1").unwrap();
        assert_eq!(loaded.current_phase, "active");
        assert_eq!(loaded.evaluation_count, 5);
    }

    #[test]
    fn test_recover_or_default_missing() {
        let mgr = StrategyStateManager::new();
        let default = mgr.recover_or_default("missing");
        assert_eq!(default.current_phase, "default");
        assert_eq!(default.evaluation_count, 0);
    }

    #[test]
    fn test_persist_roundtrip() {
        let mut mgr = StrategyStateManager::new();
        mgr.save_state(StrategyState {
            vault_id: "v1".into(),
            current_phase: "active".into(),
            last_evaluated_at: Utc::now(),
            last_trigger: Some("crossed".into()),
            evaluation_count: 10,
        });
        let bytes = mgr.persist_to_bytes();
        let restored = StrategyStateManager::restore_from_bytes(&bytes).unwrap();
        let s = restored.load_state("v1").unwrap();
        assert_eq!(s.evaluation_count, 10);
        assert_eq!(s.last_trigger.as_deref(), Some("crossed"));
    }
}
