//! Fix for #74: two-phase commit for batch settlement so a partial
//! failure leaves an explicit rollback marker instead of a mixed state.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PaymentState {
    Prepared,
    Committed,
    RolledBack,
}

pub struct SettlementBatch {
    pub payments: Vec<PaymentState>,
}
impl SettlementBatch {
    pub fn prepare(count: usize) -> Self {
        Self { payments: vec![PaymentState::Prepared; count] }
    }

    /// Phase 2: only commits if every payment prepared successfully;
    /// otherwise rolls the whole batch back and marks it explicitly.
    pub fn commit_or_rollback(&mut self, prepared_ok: &[bool]) {
        if prepared_ok.iter().all(|ok| *ok) {
            for p in self.payments.iter_mut() {
                *p = PaymentState::Committed;
            }
        } else {
            for p in self.payments.iter_mut() {
                *p = PaymentState::RolledBack;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_failure_rolls_back_whole_batch() {
        let mut batch = SettlementBatch::prepare(3);
        batch.commit_or_rollback(&[true, false, true]);
        assert!(batch.payments.iter().all(|p| *p == PaymentState::RolledBack));
    }

    #[test]
    fn full_success_commits_whole_batch() {
        let mut batch = SettlementBatch::prepare(3);
        batch.commit_or_rollback(&[true, true, true]);
        assert!(batch.payments.iter().all(|p| *p == PaymentState::Committed));
    }
}
