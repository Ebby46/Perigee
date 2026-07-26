use thiserror::Error;

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("[VAULT-001] Insufficient balance")]
    InsufficientBalance,
    #[error("[VAULT-002] Asset not approved")]
    AssetNotApproved,
    #[error("[VAULT-003] Vault does not exist")]
    NotFound,
    #[error("[VAULT-004] Unauthorized access")]
    Unauthorized,
    #[error("[VAULT-005] Invalid decimals")]
    InvalidDecimals,
}

#[derive(Error, Debug)]
pub enum FeeError {
    #[error("[FEE-001] Fee calculation overflow")]
    Overflow,
    #[error("[FEE-002] Fee exceeds cap")]
    ExceedsCap,
    #[error("[FEE-003] Invalid fee split")]
    InvalidSplit,
    #[error("[FEE-004] Settlement not final")]
    SettlementPending,
    #[error("[FEE-005] Claim window expired")]
    ClaimExpired,
}

#[derive(Error, Debug)]
pub enum PolicyError {
    #[error("[POLICY-001] Approval expired")]
    ApprovalExpired,
    #[error("[POLICY-002] Policy not found")]
    NotFound,
    #[error("[POLICY-003] Quota exceeded")]
    QuotaExceeded,
}

pub fn error_code_string(code: &str) -> String {
    format!("[{}]", code)
}
