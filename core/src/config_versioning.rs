use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedTriggerConfig {
    pub version: u64,
    pub trigger_type: String,
    pub threshold: f64,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

pub struct TriggerConfigVersioner {
    versions: HashMap<String, Vec<VersionedTriggerConfig>>,
}

impl TriggerConfigVersioner {
    pub fn new() -> Self {
        Self {
            versions: HashMap::new(),
        }
    }

    pub fn add_version(&mut self, trigger_type: &str, config: VersionedTriggerConfig) {
        self.versions
            .entry(trigger_type.to_string())
            .or_default()
            .push(config);
    }

    pub fn get_active_config(&self, trigger_type: &str) -> Option<&VersionedTriggerConfig> {
        self.versions.get(trigger_type)?.last()
    }

    pub fn get_config_at_time(
        &self,
        trigger_type: &str,
        time: DateTime<Utc>,
    ) -> Option<&VersionedTriggerConfig> {
        let configs = self.versions.get(trigger_type)?;
        configs.iter().filter(|c| c.created_at <= time).last()
    }

    pub fn snapshot_for_vault(
        &self,
        _vault_id: &str,
        trigger_type: &str,
    ) -> Option<VersionedTriggerConfig> {
        self.get_active_config(trigger_type).cloned()
    }
}
