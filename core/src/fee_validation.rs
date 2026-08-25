#![allow(dead_code)]

pub struct FeeValidator {
    max_fee_pct: f64,
}

impl FeeValidator {
    pub fn new(max_fee_pct: f64) -> Self {
        Self { max_fee_pct }
    }

    pub fn is_acceptable(&self, predicted_fee: i64, actual_fee: i64) -> bool {
        if predicted_fee == 0 {
            return true;
        }
        let diff_pct = ((actual_fee - predicted_fee).abs() as f64 / predicted_fee as f64) * 100.0;
        diff_pct <= self.max_fee_pct
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fee_within_tolerance() {
        let v = FeeValidator::new(10.0);
        assert!(v.is_acceptable(100, 105));
    }

    #[test]
    fn test_fee_exceeds_tolerance() {
        let v = FeeValidator::new(5.0);
        assert!(!v.is_acceptable(100, 120));
    }

    #[test]
    fn test_zero_predicted() {
        let v = FeeValidator::new(10.0);
        assert!(v.is_acceptable(0, 100));
    }
}
