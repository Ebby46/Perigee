use thiserror::Error;

#[derive(Error, Debug)]
pub enum MathError {
    #[error("Overflow in operation")]
    Overflow,
    #[error("Underflow in operation")]
    Underflow,
    #[error("Division by zero")]
    DivisionByZero,
}

pub fn checked_squared_return(price_change: f64) -> Result<f64, MathError> {
    price_change
        .mul_add(price_change, 0.0)
        .is_finite()
        .then_some(price_change * price_change)
        .ok_or(MathError::Overflow)
}

pub fn saturating_squared_return(price_change: f64) -> f64 {
    let result = price_change * price_change;
    if result.is_finite() {
        result
    } else {
        f64::MAX
    }
}

pub fn accumulator_volatility(prices: &[f64]) -> Result<f64, MathError> {
    if prices.len() < 2 {
        return Err(MathError::Underflow);
    }
    let mut sum_sq: f64 = 0.0;
    let mut prev = prices[0];
    for &p in &prices[1..] {
        let change = (p - prev) / prev;
        let sq = checked_squared_return(change)?;
        let new_sum = sum_sq + sq;
        if !new_sum.is_finite() {
            return Err(MathError::Overflow);
        }
        sum_sq = new_sum;
        prev = p;
    }
    Ok((sum_sq / (prices.len() - 1) as f64).sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checked_squared_return_normal() {
        assert!(checked_squared_return(5.0).unwrap() == 25.0);
        assert!(checked_squared_return(-3.0).unwrap() == 9.0);
        assert!(checked_squared_return(0.0).unwrap() == 0.0);
    }

    #[test]
    fn test_checked_squared_return_overflow() {
        let result = checked_squared_return(f64::MAX);
        assert!(result.is_err());
    }

    #[test]
    fn test_saturating_squared_return_normal() {
        assert!(saturating_squared_return(5.0) == 25.0);
        assert!(saturating_squared_return(-3.0) == 9.0);
    }

    #[test]
    fn test_saturating_squared_return_overflow() {
        assert!(saturating_squared_return(f64::MAX) == f64::MAX);
    }

    #[test]
    fn test_accumulator_volatility_insufficient_prices() {
        assert!(accumulator_volatility(&[1.0]).is_err());
        assert!(accumulator_volatility(&[]).is_err());
    }

    #[test]
    fn test_accumulator_volatility_normal() {
        let prices = vec![100.0, 102.0, 101.0, 105.0];
        let result = accumulator_volatility(&prices);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }
}
