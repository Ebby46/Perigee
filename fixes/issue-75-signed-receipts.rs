//! Fix for #75: require a signed receipt from the receiving agent
//! instead of trusting an agent-supplied micropayment amount.

pub struct SignedReceipt {
    pub amount: u64,
    pub receiver_signature: String,
}

/// Stand-in for a real signature scheme: the receiver "signs" by
/// producing a deterministic tag over the amount it actually received.
fn expected_signature(amount: u64, receiver_key: &str) -> String {
    format!("{receiver_key}:{amount}")
}

pub fn verify_receipt(
    claimed_amount: u64,
    receiver_key: &str,
    receipt: &SignedReceipt,
) -> Result<(), &'static str> {
    if receipt.amount != claimed_amount {
        return Err("receipt amount does not match agent-supplied amount");
    }
    if receipt.receiver_signature != expected_signature(receipt.amount, receiver_key) {
        return Err("receipt signature invalid");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn over_reported_amount_is_rejected() {
        let receiver_key = "receiver-1";
        let receipt = SignedReceipt {
            amount: 100,
            receiver_signature: expected_signature(100, receiver_key),
        };
        assert!(verify_receipt(100, receiver_key, &receipt).is_ok());
        assert!(verify_receipt(150, receiver_key, &receipt).is_err());
    }
}
