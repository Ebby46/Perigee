//! Fix for #77: enforce a plan-based quota on vault provisioning so a
//! single manager cannot exhaust resources by creating unlimited vaults.
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub enum Plan { Basic, Pro }
impl Plan {
    fn max_vaults(self) -> u32 {
        match self { Plan::Basic => 5, Plan::Pro => 50 }
    }
}

pub struct VaultQuota {
    plans: HashMap<String, Plan>,
    counts: HashMap<String, u32>,
}
impl VaultQuota {
    pub fn new() -> Self {
        Self { plans: HashMap::new(), counts: HashMap::new() }
    }
    pub fn set_plan(&mut self, manager_id: &str, plan: Plan) {
        self.plans.insert(manager_id.to_string(), plan);
    }
    pub fn try_provision(&mut self, manager_id: &str) -> Result<(), &'static str> {
        let plan = *self.plans.get(manager_id).unwrap_or(&Plan::Basic);
        let count = self.counts.entry(manager_id.to_string()).or_insert(0);
        if *count >= plan.max_vaults() {
            return Err("vault quota exceeded for manager's plan");
        }
        *count += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_plan_is_capped() {
        let mut quota = VaultQuota::new();
        quota.set_plan("mgr-1", Plan::Basic);
        for _ in 0..5 {
            quota.try_provision("mgr-1").unwrap();
        }
        assert!(quota.try_provision("mgr-1").is_err());
    }
}
