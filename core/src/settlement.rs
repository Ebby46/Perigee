use chrono::{DateTime, Duration, Utc};

pub struct SettlementRecord {
    pub vault_id: String,
    pub gain_realized_at: DateTime<Utc>,
    pub settlement_delay: Duration,
    pub is_settled: bool,
}

impl SettlementRecord {
    pub fn new(vault_id: String, gain_time: DateTime<Utc>, delay_secs: i64) -> Self {
        Self {
            vault_id,
            gain_realized_at: gain_time,
            settlement_delay: Duration::seconds(delay_secs),
            is_settled: false,
        }
    }

    pub fn is_final(&self, current_time: DateTime<Utc>) -> bool {
        if self.is_settled {
            return true;
        }
        current_time >= self.gain_realized_at + self.settlement_delay
    }

    pub fn time_until_finality(&self, current_time: DateTime<Utc>) -> Option<Duration> {
        if self.is_settled {
            return None;
        }
        let deadline = self.gain_realized_at + self.settlement_delay;
        if current_time >= deadline {
            None
        } else {
            Some(deadline - current_time)
        }
    }
}

pub struct SettlementGuard {
    default_delay: Duration,
    pending: Vec<SettlementRecord>,
}

impl SettlementGuard {
    pub fn new(default_delay_secs: i64) -> Self {
        Self {
            default_delay: Duration::seconds(default_delay_secs),
            pending: Vec::new(),
        }
    }

    pub fn register_gain(&mut self, vault_id: String, gain_time: DateTime<Utc>) {
        let record = SettlementRecord::new(vault_id, gain_time, self.default_delay.num_seconds());
        self.pending.push(record);
    }

    pub fn can_claim(&self, vault_id: &str, current_time: DateTime<Utc>) -> bool {
        self.pending
            .iter()
            .filter(|r| r.vault_id == vault_id)
            .all(|r| r.is_final(current_time))
    }

    pub fn finalize(&mut self, vault_id: &str) {
        for record in &mut self.pending {
            if record.vault_id == vault_id {
                record.is_settled = true;
            }
        }
    }

    pub fn pending_count(&self) -> usize {
        self.pending.iter().filter(|r| !r.is_settled).count()
    }
}
