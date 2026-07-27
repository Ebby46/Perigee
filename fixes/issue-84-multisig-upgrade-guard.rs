//! Fix for #84: gate contract upgrades behind multisig approval instead of
//! a single deploy key.

pub const MIN_APPROVALS: usize = 2;

pub struct MultisigUpgradeGuard {
    pub required_signers: Vec<String>,
    approvals: Vec<String>,
}

impl MultisigUpgradeGuard {
    pub fn new(required_signers: Vec<String>) -> Self {
        Self { required_signers, approvals: Vec::new() }
    }

    pub fn approve(&mut self, signer: &str) -> Result<(), &'static str> {
        if !self.required_signers.iter().any(|s| s == signer) {
            return Err("signer not authorized");
        }
        if self.approvals.iter().any(|s| s == signer) {
            return Err("signer already approved");
        }
        self.approvals.push(signer.to_string());
        Ok(())
    }

    pub fn can_deploy(&self) -> bool {
        self.approvals.len() >= MIN_APPROVALS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_min_approvals_before_deploy() {
        let mut guard = MultisigUpgradeGuard::new(vec!["a".into(), "b".into(), "c".into()]);
        assert!(!guard.can_deploy());
        guard.approve("a").unwrap();
        assert!(!guard.can_deploy());
        guard.approve("b").unwrap();
        assert!(guard.can_deploy());
    }
}
