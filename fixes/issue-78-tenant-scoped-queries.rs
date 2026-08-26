//! Fix for #78: scope every reporting query by manager_id so a missing
//! tenant filter under certain sort params can't leak cross-manager data.

pub struct ClientRecord {
    pub manager_id: String,
    pub client_name: String,
}

/// All reporting reads must go through this helper so the manager_id
/// filter can never be forgotten regardless of sort/order params.
pub fn scoped_report(records: &[ClientRecord], manager_id: &str) -> Vec<&ClientRecord> {
    records.iter().filter(|r| r.manager_id == manager_id).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_never_leaks_other_managers_clients() {
        let records = vec![
            ClientRecord { manager_id: "mgr-a".into(), client_name: "alice".into() },
            ClientRecord { manager_id: "mgr-b".into(), client_name: "bob".into() },
        ];
        let result = scoped_report(&records, "mgr-a");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].client_name, "alice");
    }
}
