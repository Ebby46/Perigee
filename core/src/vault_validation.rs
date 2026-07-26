use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VaultValidationError {
    #[error("Asset decimals {0} not in registry for {1}")]
    UnknownDecimals(u32, String),
    #[error("Decimal mismatch: expected {expected}, got {actual}")]
    DecimalMismatch { expected: u32, actual: u32 },
}

pub struct AssetDecimalRegistry {
    known_decimals: HashMap<String, u32>,
}

impl AssetDecimalRegistry {
    pub fn new() -> Self {
        Self {
            known_decimals: HashMap::new(),
        }
    }

    pub fn register(&mut self, asset: &str, decimals: u32) {
        self.known_decimals.insert(asset.to_string(), decimals);
    }

    pub fn validate_asset(&self, asset: &str, decimals: u32) -> Result<(), VaultValidationError> {
        match self.known_decimals.get(asset) {
            Some(&expected) if expected == decimals => Ok(()),
            Some(&expected) => Err(VaultValidationError::DecimalMismatch {
                expected,
                actual: decimals,
            }),
            None => Err(VaultValidationError::UnknownDecimals(
                decimals,
                asset.to_string(),
            )),
        }
    }

    pub fn get_decimals(&self, asset: &str) -> Option<u32> {
        self.known_decimals.get(asset).copied()
    }
}
