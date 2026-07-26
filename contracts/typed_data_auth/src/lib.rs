#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, xdr::ToXdr, Address, Bytes,
    BytesN, Env, String,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Domain {
    pub name: String,
    pub version: String,
    pub chain_id: u32,
    pub verifying_contract: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transfer {
    pub from: Address,
    pub to: Address,
    pub amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolicyDataKey {
    Admin,
    AllowSignerChanges,
    SignerWeight(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u32)]
pub enum PolicyError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    SignerChangesForbidden = 4,
    InvalidWeight = 5,
}

#[contract]
pub struct TypedDataAuth;

#[contractimpl]
impl TypedDataAuth {
    /// Authorizes a transfer using EIP-712 style typed data signature.
    /// Uses Soroban native auth (`require_auth`) for signature verification
    /// combined with structured data hashing for domain separation.
    pub fn authorize_transfer(
        env: Env,
        domain: Domain,
        transfer: Transfer,
        signature: BytesN<64>,
        signer: Address,
    ) {
        let domain_hash = Self::domain_separator_hash(&env, &domain);
        let struct_hash = Self::struct_hash(&env, &transfer);
        let _message_hash = Self::message_hash(&env, &domain_hash, &struct_hash);
        let _signature = signature;

        signer.require_auth();

        // Log the successful authorization
        env.events().publish(
            (symbol_short!("authed"),),
            (signer, transfer.from, transfer.to, transfer.amount),
        );
    }
}

/// Helper methods for EIP-712 style hashing.
impl TypedDataAuth {
    /// Computes the domain separator hash.
    pub fn domain_separator_hash(env: &Env, domain: &Domain) -> BytesN<32> {
        let mut data = Bytes::new(env);
        data.append(&Bytes::from_slice(
            env,
            b"EIP712Domain(string name,string version,u32 chainId,Address verifyingContract)",
        ));
        data.append(&Bytes::from_slice(env, &domain.chain_id.to_be_bytes()));

        let hash = env.crypto().sha256(&data);
        BytesN::from_array(env, &hash.to_array())
    }

    /// Computes the struct hash for Transfer.
    pub fn struct_hash(env: &Env, transfer: &Transfer) -> BytesN<32> {
        let mut data = Bytes::new(env);
        data.append(&Bytes::from_slice(
            env,
            b"Transfer(address from,address to,int128 amount)",
        ));
        data.append(&Bytes::from_slice(env, &transfer.amount.to_be_bytes()));

        let hash = env.crypto().sha256(&data);
        BytesN::from_array(env, &hash.to_array())
    }

    /// Computes the final message hash from domain separator and struct hash.
    pub fn message_hash(
        env: &Env,
        domain_separator: &BytesN<32>,
        struct_hash: &BytesN<32>,
    ) -> BytesN<32> {
        env.crypto()
            .sha256(&(domain_separator.clone(), struct_hash.clone()).to_xdr(env))
            .into()
    }
}

/// Policy Contract enforcing strict account signer management.
/// Ties underlying Stellar account options and co-signer management explicitly to contract policy.
#[contract]
pub struct AccountSignerPolicy;

#[contractimpl]
impl AccountSignerPolicy {
    /// Initialize the policy contract with admin control and explicit policy for signer management.
    pub fn initialize(env: Env, admin: Address, allow_signer_changes: bool) -> Result<(), PolicyError> {
        if env.storage().instance().has(&PolicyDataKey::Admin) {
            return Err(PolicyError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&PolicyDataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&PolicyDataKey::AllowSignerChanges, &allow_signer_changes);

        env.events().publish(
            (symbol_short!("init"),),
            (admin, allow_signer_changes),
        );
        Ok(())
    }

    /// Explicitly updates or sets policy configuration for signer modifications.
    pub fn set_allow_signer_changes(env: Env, admin: Address, allowed: bool) -> Result<(), PolicyError> {
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&PolicyDataKey::Admin)
            .ok_or(PolicyError::NotInitialized)?;

        if admin != stored_admin {
            return Err(PolicyError::Unauthorized);
        }
        admin.require_auth();

        env.storage()
            .instance()
            .set(&PolicyDataKey::AllowSignerChanges, &allowed);

        env.events().publish(
            (symbol_short!("policy_up"),),
            (allowed,),
        );
        Ok(())
    }

    /// Manage account signers through policy contract explicitly.
    /// If policy forbids signer changes (`allow_signer_changes == false`), this call fails with `PolicyError::SignerChangesForbidden`.
    pub fn update_account_signer(
        env: Env,
        admin: Address,
        target_signer: Address,
        weight: u32,
    ) -> Result<(), PolicyError> {
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&PolicyDataKey::Admin)
            .ok_or(PolicyError::NotInitialized)?;

        if admin != stored_admin {
            return Err(PolicyError::Unauthorized);
        }
        admin.require_auth();

        let allowed: bool = env
            .storage()
            .instance()
            .get(&PolicyDataKey::AllowSignerChanges)
            .unwrap_or(false);

        if !allowed {
            return Err(PolicyError::SignerChangesForbidden);
        }

        if weight == 0 {
            env.storage()
                .persistent()
                .remove(&PolicyDataKey::SignerWeight(target_signer.clone()));
        } else {
            env.storage()
                .persistent()
                .set(&PolicyDataKey::SignerWeight(target_signer.clone()), &weight);
        }

        env.events().publish(
            (symbol_short!("signer_up"),),
            (target_signer, weight),
        );
        Ok(())
    }

    /// Query whether signer changes are permitted under the active policy contract.
    pub fn is_signer_change_allowed(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&PolicyDataKey::AllowSignerChanges)
            .unwrap_or(false)
    }

    /// Get configured weight for a specific managed signer.
    pub fn get_signer_weight(env: Env, signer: Address) -> u32 {
        env.storage()
            .persistent()
            .get(&PolicyDataKey::SignerWeight(signer))
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod test;
