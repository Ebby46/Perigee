use chrono::{DateTime, Utc};
use serde::Serialize;
use tracing::info;

#[derive(Debug, Serialize)]
pub struct AuditEvent {
    pub manager_id: String,
    pub action: String,
    pub actor: String,
    pub timestamp: DateTime<Utc>,
}

pub fn log_audit_event(manager_id: &str, action: &str, actor: &str) {
    let event = AuditEvent {
        manager_id: manager_id.to_string(),
        action: action.to_string(),
        actor: actor.to_string(),
        timestamp: Utc::now(),
    };
    
    info!(
        target: "audit_log",
        manager_id = %event.manager_id,
        action = %event.action,
        actor = %event.actor,
        timestamp = %event.timestamp.to_rfc3339(),
        "AUDIT: {} by {}",
        event.action,
        event.actor
    );
}
