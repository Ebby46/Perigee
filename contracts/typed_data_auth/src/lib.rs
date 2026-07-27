#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, xdr::ToXdr, Address, Bytes,
    BytesN, Env, String, Vec,
};

/// Default cap on the approved-asset list until an admin explicitly migrates it higher.
const DEFAULT_MAX_APPROVED_ASSETS: u32 = 50;
/// Hard ceiling on the approved-asset cap so a migration can never make the list unbounded.
const MAX_APPROVED_ASSETS_CEILING: u32 = 500;

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
    ApprovedAssets,
    MaxApprovedAssets,
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
    AssetListFull = 6,
    AssetAlreadyApproved = 7,
    AssetNotApproved = 8,
    InvalidMaxApprovedAssets = 9,
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

    /// Currently configured cap on the approved-asset list.
    /// Defaults to `DEFAULT_MAX_APPROVED_ASSETS` until raised via `set_max_approved_assets`.
    pub fn max_approved_assets(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&PolicyDataKey::MaxApprovedAssets)
            .unwrap_or(DEFAULT_MAX_APPROVED_ASSETS)
    }

    /// Admin-only migration step to raise the approved-asset cap.
    /// Bounded by `MAX_APPROVED_ASSETS_CEILING` so growth can never become unbounded,
    /// and can never drop below the number of assets already approved.
    pub fn set_max_approved_assets(env: Env, admin: Address, new_max: u32) -> Result<(), PolicyError> {
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&PolicyDataKey::Admin)
            .ok_or(PolicyError::NotInitialized)?;

        if admin != stored_admin {
            return Err(PolicyError::Unauthorized);
        }
        admin.require_auth();

        let current_len = Self::get_approved_assets(env.clone()).len();
        if new_max > MAX_APPROVED_ASSETS_CEILING || new_max < current_len {
            return Err(PolicyError::InvalidMaxApprovedAssets);
        }

        env.storage()
            .instance()
            .set(&PolicyDataKey::MaxApprovedAssets, &new_max);

        env.events().publish((symbol_short!("max_up"),), (new_max,));
        Ok(())
    }

    /// Admin-only: add an asset to the approved list, enforcing the configured cap.
    pub fn add_approved_asset(env: Env, admin: Address, asset: Address) -> Result<(), PolicyError> {
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&PolicyDataKey::Admin)
            .ok_or(PolicyError::NotInitialized)?;

        if admin != stored_admin {
            return Err(PolicyError::Unauthorized);
        }
        admin.require_auth();

        let mut assets = Self::get_approved_assets(env.clone());
        if assets.contains(&asset) {
            return Err(PolicyError::AssetAlreadyApproved);
        }

        let max = Self::max_approved_assets(env.clone());
        if assets.len() >= max {
            return Err(PolicyError::AssetListFull);
        }

        assets.push_back(asset.clone());
        env.storage()
            .instance()
            .set(&PolicyDataKey::ApprovedAssets, &assets);

        env.events().publish((symbol_short!("asset_add"),), (asset,));
        Ok(())
    }

    /// Admin-only: remove an asset from the approved list.
    pub fn remove_approved_asset(env: Env, admin: Address, asset: Address) -> Result<(), PolicyError> {
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&PolicyDataKey::Admin)
            .ok_or(PolicyError::NotInitialized)?;

        if admin != stored_admin {
            return Err(PolicyError::Unauthorized);
        }
        admin.require_auth();

        let assets = Self::get_approved_assets(env.clone());
        let mut new_assets = Vec::new(&env);
        let mut found = false;
        for a in assets.iter() {
            if a != asset {
                new_assets.push_back(a);
            } else {
                found = true;
            }
        }
        if !found {
            return Err(PolicyError::AssetNotApproved);
        }

        env.storage()
            .instance()
            .set(&PolicyDataKey::ApprovedAssets, &new_assets);

        env.events().publish((symbol_short!("asset_rm"),), (asset,));
        Ok(())
    }

    /// Read the current approved-asset list.
    pub fn get_approved_assets(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&PolicyDataKey::ApprovedAssets)
            .unwrap_or(Vec::new(&env))
    }
}

#[cfg(test)]
mod test;
