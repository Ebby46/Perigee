//! Fix for #79: reject a client-supplied fee split markup that exceeds
//! the server-configured ceiling instead of trusting it outright.

pub const MAX_MARKUP_BPS: u32 = 500; // 5%, server policy ceiling

pub fn validate_markup(requested_bps: u32) -> Result<u32, &'static str> {
    if requested_bps > MAX_MARKUP_BPS {
        return Err("requested markup exceeds server-configured ceiling");
    }
    Ok(requested_bps)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markup_within_ceiling_is_accepted() {
        assert_eq!(validate_markup(300), Ok(300));
    }

    #[test]
    fn markup_over_ceiling_is_rejected() {
        assert!(validate_markup(1000).is_err());
    }
}
