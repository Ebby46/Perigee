use crate::{AccountSignerPolicy, AccountSignerPolicyClient, Domain, PolicyError, Transfer, TypedDataAuth};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{symbol_short, Address, BytesN, Env, String};

#[test]
fn test_domain_separator_hash() {
    let env = Env::default();
    let contract_address = Address::generate(&env);
    let domain = Domain {
        name: String::from_str(&env, "TestContract"),
        version: String::from_str(&env, "1.0"),
        chain_id: 1,
        verifying_contract: contract_address,
    };
    let hash = TypedDataAuth::domain_separator_hash(&env, &domain);
    let zero = BytesN::from_array(&env, &[0u8; 32]);
    assert_ne!(hash, zero);
}

#[test]
fn test_struct_hash() {
    let env = Env::default();
    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let transfer = Transfer {
        from: from.clone(),
        to: to.clone(),
        amount: 1000,
    };

    let hash = TypedDataAuth::struct_hash(&env, &transfer);
    let zero = BytesN::from_array(&env, &[0u8; 32]);
    assert_ne!(hash, zero);
}

#[test]
fn test_message_hash() {
    let env = Env::default();
    let domain_hash = BytesN::from_array(&env, &[1u8; 32]);
    let struct_hash = BytesN::from_array(&env, &[2u8; 32]);

    let message_hash = TypedDataAuth::message_hash(&env, &domain_hash, &struct_hash);
    let zero = BytesN::from_array(&env, &[0u8; 32]);
    assert_ne!(message_hash, zero);
}

#[test]
fn test_domain_separator_consistency() {
    let env = Env::default();
    let contract_address = Address::generate(&env);
    let domain1 = Domain {
        name: String::from_str(&env, "TestContract"),
        version: String::from_str(&env, "1.0"),
        chain_id: 1,
        verifying_contract: contract_address.clone(),
    };
    let domain2 = Domain {
        name: String::from_str(&env, "TestContract"),
        version: String::from_str(&env, "1.0"),
        chain_id: 1,
        verifying_contract: contract_address,
    };
    assert_eq!(
        TypedDataAuth::domain_separator_hash(&env, &domain1),
        TypedDataAuth::domain_separator_hash(&env, &domain2),
    );
}

#[test]
fn test_different_domains_produce_different_hashes() {
    let env = Env::default();
    let contract_address = Address::generate(&env);
    let domain1 = Domain {
        name: String::from_str(&env, "TestContract"),
        version: String::from_str(&env, "1.0"),
        chain_id: 1,
        verifying_contract: contract_address.clone(),
    };
    // Different chain_id should produce a different hash
    let domain2 = Domain {
        name: String::from_str(&env, "TestContract"),
        version: String::from_str(&env, "1.0"),
        chain_id: 2,
        verifying_contract: contract_address,
    };

    let hash1 = TypedDataAuth::domain_separator_hash(&env, &domain1);
    let hash2 = TypedDataAuth::domain_separator_hash(&env, &domain2);

    assert_ne!(hash1, hash2);
}

#[test]
fn test_account_signer_policy_forbids_signer_changes() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AccountSignerPolicy);
    let client = AccountSignerPolicyClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let co_signer = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(&admin, &false);

    assert!(!client.is_signer_change_allowed());

    let res = client.try_update_account_signer(&admin, &co_signer, &100);
    assert_eq!(res, Err(Ok(PolicyError::SignerChangesForbidden)));
}

#[test]
fn test_account_signer_policy_allows_signer_changes_when_enabled() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AccountSignerPolicy);
    let client = AccountSignerPolicyClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let co_signer = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(&admin, &true);

    assert!(client.is_signer_change_allowed());

    client.update_account_signer(&admin, &co_signer, &100);
    assert_eq!(client.get_signer_weight(&co_signer), 100);

    client.update_account_signer(&admin, &co_signer, &0);
    assert_eq!(client.get_signer_weight(&co_signer), 0);
}

// PV-02: approved-asset list must be bounded, with expansion requiring an explicit migration.

#[test]
fn test_add_approved_asset_enforces_configured_cap() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AccountSignerPolicy);
    let client = AccountSignerPolicyClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let asset_a = Address::generate(&env);
    let asset_b = Address::generate(&env);
    let asset_c = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(&admin, &false);
    client.set_max_approved_assets(&admin, &2);

    client.add_approved_asset(&admin, &asset_a);
    client.add_approved_asset(&admin, &asset_b);

    let res = client.try_add_approved_asset(&admin, &asset_c);
    assert_eq!(res, Err(Ok(PolicyError::AssetListFull)));
    assert_eq!(client.get_approved_assets().len(), 2);
}

#[test]
fn test_add_approved_asset_rejects_duplicate() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AccountSignerPolicy);
    let client = AccountSignerPolicyClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let asset = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(&admin, &false);
    client.add_approved_asset(&admin, &asset);

    let res = client.try_add_approved_asset(&admin, &asset);
    assert_eq!(res, Err(Ok(PolicyError::AssetAlreadyApproved)));
}

#[test]
fn test_remove_approved_asset_then_readd_succeeds() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AccountSignerPolicy);
    let client = AccountSignerPolicyClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let asset = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(&admin, &false);
    client.add_approved_asset(&admin, &asset);
    assert_eq!(client.get_approved_assets().len(), 1);

    client.remove_approved_asset(&admin, &asset);
    assert_eq!(client.get_approved_assets().len(), 0);

    client.add_approved_asset(&admin, &asset);
    assert_eq!(client.get_approved_assets().len(), 1);
}

#[test]
fn test_expanding_asset_cap_requires_explicit_migration() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AccountSignerPolicy);
    let client = AccountSignerPolicyClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let asset_a = Address::generate(&env);
    let asset_b = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(&admin, &false);
    client.set_max_approved_assets(&admin, &1);
    client.add_approved_asset(&admin, &asset_a);

    // Still bounded until the admin explicitly migrates the cap upward.
    let res = client.try_add_approved_asset(&admin, &asset_b);
    assert_eq!(res, Err(Ok(PolicyError::AssetListFull)));

    client.set_max_approved_assets(&admin, &2);
    client.add_approved_asset(&admin, &asset_b);
    assert_eq!(client.get_approved_assets().len(), 2);
}

#[test]
fn test_max_approved_assets_cannot_exceed_ceiling() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AccountSignerPolicy);
    let client = AccountSignerPolicyClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(&admin, &false);

    let res = client.try_set_max_approved_assets(&admin, &501);
    assert_eq!(res, Err(Ok(PolicyError::InvalidMaxApprovedAssets)));
}

#[test]
fn test_max_approved_assets_cannot_drop_below_current_length() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AccountSignerPolicy);
    let client = AccountSignerPolicyClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let asset_a = Address::generate(&env);
    let asset_b = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(&admin, &false);
    client.add_approved_asset(&admin, &asset_a);
    client.add_approved_asset(&admin, &asset_b);

    let res = client.try_set_max_approved_assets(&admin, &1);
    assert_eq!(res, Err(Ok(PolicyError::InvalidMaxApprovedAssets)));
}

// PV-04: scoped calls must carry a per-vault, per-agent nonce so a captured
// authorization cannot be replayed within the same ledger.

#[test]
fn test_scoped_call_rejects_replayed_nonce() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AccountSignerPolicy);
    let client = AccountSignerPolicyClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let agent = Address::generate(&env);
    let action = symbol_short!("rebal");

    env.mock_all_auths();
    client.initialize(&admin, &false);

    assert_eq!(client.agent_nonce(&agent), 0);
    client.execute_scoped_call(&agent, &action, &0);
    assert_eq!(client.agent_nonce(&agent), 1);

    // Replaying the same (already-consumed) nonce must fail.
    let res = client.try_execute_scoped_call(&agent, &action, &0);
    assert_eq!(res, Err(Ok(PolicyError::InvalidNonce)));

    // The correct next nonce succeeds.
    client.execute_scoped_call(&agent, &action, &1);
    assert_eq!(client.agent_nonce(&agent), 2);
}

#[test]
fn test_scoped_call_rejects_skipped_nonce() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AccountSignerPolicy);
    let client = AccountSignerPolicyClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let agent = Address::generate(&env);
    let action = symbol_short!("rebal");

    env.mock_all_auths();
    client.initialize(&admin, &false);

    // Skipping ahead to nonce 5 without consuming 0-4 must fail.
    let res = client.try_execute_scoped_call(&agent, &action, &5);
    assert_eq!(res, Err(Ok(PolicyError::InvalidNonce)));
}

#[test]
fn test_scoped_call_nonces_are_independent_per_agent() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AccountSignerPolicy);
    let client = AccountSignerPolicyClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let agent_a = Address::generate(&env);
    let agent_b = Address::generate(&env);
    let action = symbol_short!("rotate");

    env.mock_all_auths();
    client.initialize(&admin, &false);

    client.execute_scoped_call(&agent_a, &action, &0);
    assert_eq!(client.agent_nonce(&agent_a), 1);
    assert_eq!(client.agent_nonce(&agent_b), 0);

    // agent_b's nonce sequence starts independently at 0.
    client.execute_scoped_call(&agent_b, &action, &0);
    assert_eq!(client.agent_nonce(&agent_b), 1);
}
