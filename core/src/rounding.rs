/// Bankers rounding utilities to fix protocol-favoring bias on small balances.

/// Round half to even (banker's rounding).
///
/// When the fractional part is exactly 0.5, rounds to the nearest even integer
/// instead of always rounding up, which biases fee calculations against users
/// with small balances.
pub fn bankers_round(value: f64) -> f64 {
    let floor = value.floor();
    let frac = value - floor;
    if frac == 0.5 {
        if floor as i64 % 2 == 0 {
            floor
        } else {
            floor + 1.0
        }
    } else {
        (value + 0.5).floor()
    }
}

/// Apply fee with banker's rounding and a minimum fee threshold.
///
/// `amount`: the raw amount in smallest units (e.g. stroops).
/// `fee_bps`: fee rate in basis points (0..=10_000).
/// `min_fee`: minimum fee that must always be charged (in same units as amount).
///
/// Returns the fee amount after banker's rounding, clamped to at least `min_fee`.
pub fn calculate_fee_with_bankers_round(amount: u64, fee_bps: u64, min_fee: u64) -> u64 {
    let raw = (amount as f64) * (fee_bps as f64) / 10_000.0;
    let rounded = bankers_round(raw);
    let fee = rounded.max(min_fee as f64) as u64;
    fee
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bankers_round_half_to_even() {
        assert_eq!(bankers_round(0.5), 0.0);
        assert_eq!(bankers_round(1.5), 2.0);
        assert_eq!(bankers_round(2.5), 2.0);
        assert_eq!(bankers_round(3.5), 4.0);
        assert_eq!(bankers_round(4.5), 4.0);
    }

    #[test]
    fn test_bankers_round_non_half() {
        assert_eq!(bankers_round(0.1), 0.0);
        assert_eq!(bankers_round(0.9), 1.0);
        assert_eq!(bankers_round(1.0), 1.0);
        assert_eq!(bankers_round(1.4), 1.0);
        assert_eq!(bankers_round(1.6), 2.0);
    }

    #[test]
    fn test_bankers_round_negative() {
        assert_eq!(bankers_round(-0.5), 0.0);
        assert_eq!(bankers_round(-1.5), -2.0);
    }

    #[test]
    fn test_calculate_fee_with_bankers_round_basic() {
        // 1000 * 250 bps = 25.0 stroops
        assert_eq!(calculate_fee_with_bankers_round(1000, 250, 0), 25);
    }

    #[test]
    fn test_calculate_fee_with_bankers_round_min_fee() {
        // 100 * 10 bps = 0.1 → rounds to 0, but min_fee is 1
        assert_eq!(calculate_fee_with_bankers_round(100, 10, 1), 1);
    }

    #[test]
    fn test_calculate_fee_with_bankers_round_zero_bps() {
        assert_eq!(calculate_fee_with_bankers_round(1000, 0, 0), 0);
    }

    #[test]
    fn test_calculate_fee_with_bankers_round_small_balance() {
        // 10 stroops, 100 bps (1%) → 0.1, rounds to 0, min_fee = 1
        assert_eq!(calculate_fee_with_bankers_round(10, 100, 1), 1);
    }
}
