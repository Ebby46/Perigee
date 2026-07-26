#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedReceipt {
    pub payment_id: String,
    pub amount: u64,
    pub receiver_agent: String,
    pub sender_agent: String,
    pub signature: Vec<u8>,
}

impl SignedReceipt {
    pub fn new(payment_id: String, amount: u64, receiver: String, sender: String) -> Self {
        Self {
            payment_id,
            amount,
            receiver_agent: receiver,
            sender_agent: sender,
            signature: Vec::new(),
        }
    }

    pub fn verify_signature(&self) -> bool {
        !self.signature.is_empty()
    }

    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.payment_id.as_bytes());
        bytes.extend_from_slice(&self.amount.to_be_bytes());
        bytes.extend_from_slice(self.receiver_agent.as_bytes());
        bytes.extend_from_slice(self.sender_agent.as_bytes());
        bytes
    }
}

pub struct ReceiptVerifier {
    trusted_keys: Vec<Vec<u8>>,
}

impl ReceiptVerifier {
    pub fn new() -> Self {
        Self {
            trusted_keys: Vec::new(),
        }
    }

    pub fn add_trusted_key(&mut self, key: Vec<u8>) {
        self.trusted_keys.push(key);
    }

    pub fn verify(&self, receipt: &SignedReceipt) -> bool {
        if self.trusted_keys.is_empty() {
            return false;
        }
        receipt.verify_signature()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receipt_creation() {
        let receipt = SignedReceipt::new(
            "pay1".to_string(),
            1000,
            "receiver_a".to_string(),
            "sender_b".to_string(),
        );
        assert_eq!(receipt.amount, 1000);
        assert_eq!(receipt.payment_id, "pay1");
    }

    #[test]
    fn test_canonical_bytes() {
        let receipt = SignedReceipt::new(
            "pay1".to_string(),
            1000,
            "receiver_a".to_string(),
            "sender_b".to_string(),
        );
        let bytes = receipt.to_canonical_bytes();
        assert!(!bytes.is_empty());
        assert!(bytes.windows(4).any(|w| w == b"pay1"));
    }

    #[test]
    fn test_verify_empty_sig() {
        let mut receipt = SignedReceipt::new(
            "pay1".to_string(),
            1000,
            "r".to_string(),
            "s".to_string(),
        );
        assert!(!receipt.verify_signature());
        receipt.signature = vec![1, 2, 3];
        assert!(receipt.verify_signature());
    }

    #[test]
    fn test_verifier_rejects_no_keys() {
        let verifier = ReceiptVerifier::new();
        let receipt = SignedReceipt::new(
            "pay1".to_string(),
            1000,
            "r".to_string(),
            "s".to_string(),
        );
        assert!(!verifier.verify(&receipt));
    }
}
