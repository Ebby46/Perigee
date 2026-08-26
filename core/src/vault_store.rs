//! White-label vault records with optimistic locking (API-37).
//!
//! Concurrent updates to the same vault must supply the current `version`.
//! The store bumps `version` only when `WHERE id = ? AND version = ?` matches;
//! otherwise the caller gets a conflict and must reload.

use crate::auth::AuthenticatedUser;
use crate::errors::AppError;
use crate::db;
use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum VaultStoreError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Vault not found: {0}")]
    NotFound(String),

    #[error("Version conflict for vault {vault_id}: expected {expected_version}")]
    Conflict {
        vault_id: String,
        expected_version: i64,
    },

    #[error("Invalid data: {0}")]
    InvalidData(String),
}

impl From<VaultStoreError> for AppError {
    fn from(err: VaultStoreError) -> Self {
        match err {
            VaultStoreError::NotFound(msg) => AppError::NotFound(msg),
            VaultStoreError::Conflict {
                vault_id,
                expected_version,
            } => AppError::Conflict(format!(
                "Vault '{vault_id}' was updated by another request (expected version {expected_version}); reload and retry"
            )),
            VaultStoreError::InvalidData(msg) => AppError::BadRequest(msg),
            VaultStoreError::Database(e) => AppError::Internal(e.to_string()),
        }
    }
}

/// Vault DTOs live in [`crate::db::models`]: the typed DB layer returns them
/// directly, so re-exporting is what keeps this store's signatures compatible
/// with what the DB hands back. They were previously duplicated here
/// field-for-field, which is what made `VaultStore::get` return one type while
/// the DB produced another.
pub use crate::db::models::{CreateVaultRequest, UpdateVaultRequest, VaultRecord};

pub struct VaultStore {
    vaults: db::schema::VaultsTable,
}

impl VaultStore {
    pub fn new(vaults: db::schema::VaultsTable) -> Self {
        Self { vaults }
    }

    pub async fn create(&self, req: &CreateVaultRequest) -> Result<VaultRecord, VaultStoreError> {
        if req.manager_id.trim().is_empty() {
            return Err(VaultStoreError::InvalidData(
                "manager_id must not be empty".into(),
            ));
        }
        if req.name.trim().is_empty() {
            return Err(VaultStoreError::InvalidData(
                "name must not be empty".into(),
            ));
        }

        // Idempotency: if a key is provided and non-empty, return the existing vault
        // for the same (manager_id, idempotency_key) pair.
        let idempotency_key = req
            .idempotency_key
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());

        if let Some(key) = idempotency_key {
            if let Some(vault) = self
                .vaults
                .find_by_idempotency_key(req.manager_id.trim(), key)
                .await
                .map_err(VaultStoreError::Database)?
            {
                return Ok(vault);
            }
        }

        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let manager_id = req.manager_id.trim();
        let name = req.name.trim();
        let status = req.status.trim();
        let config_json = req.config_json.trim();

        self.vaults
            .insert(&id, manager_id, name, status, config_json, idempotency_key, now)
            .await
            .map_err(VaultStoreError::Database)
    }

    pub async fn list_by_manager(&self, manager_id: &str) -> Result<Vec<VaultRecord>, VaultStoreError> {
        sqlx::query_as::<_, VaultRecord>(
            r#"
            SELECT id, manager_id, name, status, config_json, version,
                   idempotency_key, created_at, updated_at
            FROM vaults
            WHERE manager_id = ?1
            ORDER BY created_at DESC
            "#,
        )
        .bind(manager_id)
        .fetch_all(self.vaults.pool())
        .await
        .map_err(VaultStoreError::Database)
    }

    pub async fn get(&self, id: &str) -> Result<VaultRecord, VaultStoreError> {
        self.vaults
            .find_by_id(id)
            .await
            .map_err(VaultStoreError::Database)?
            .ok_or_else(|| VaultStoreError::NotFound(id.to_string()))
    }

    /// Apply an update inside a transaction using optimistic locking.
    ///
    /// The typed schema's update method uses `WHERE id = ? AND version = ?`
    /// to guarantee only one concurrent writer succeeds for a given version
    /// snapshot.
    pub async fn update(
        &self,
        id: &str,
        req: &UpdateVaultRequest,
    ) -> Result<VaultRecord, VaultStoreError> {
        if req.version < 1 {
            return Err(VaultStoreError::InvalidData(
                "version must be >= 1".into(),
            ));
        }

        let name = req.name.as_deref().map(str::trim).filter(|s| !s.is_empty());
        let status = req.status.as_deref().map(str::trim).filter(|s| !s.is_empty());
        let config_json = req.config_json.as_deref();
        let now = Utc::now();

        let result = self
            .vaults
            .update(id, req.version, name, status, config_json, now)
            .await;

        match result {
            Ok(record) => Ok(record),

            // The typed update matches on `id AND version`, so "no rows" means
            // either the vault is gone or the caller held a stale version.
            // Those are different answers — 404 versus 409 — and the caller
            // needs to know which, so ask the store which one it was.
            Err(sqlx::Error::RowNotFound) => {
                if self.vaults.find_by_id(id).await.ok().flatten().is_some() {
                    Err(VaultStoreError::Conflict {
                        vault_id: id.to_string(),
                        expected_version: req.version,
                    })
                } else {
                    Err(VaultStoreError::NotFound(id.to_string()))
                }
            }

            Err(sqlx::Error::Database(db_err)) => {
                if db_err.message().contains("UNIQUE") {
                    Err(VaultStoreError::InvalidData(
                        "A vault with this idempotency key already exists".into(),
                    ))
                } else {
                    Err(VaultStoreError::Database(sqlx::Error::Database(db_err)))
                }
            }

            Err(other) => Err(VaultStoreError::Database(other)),
        }
    }
}

/// Verify that `manager_id` (UUID) belongs to the authenticated Stellar address.
async fn verify_ownership(
    state: &crate::AppState,
    user: &AuthenticatedUser,
    manager_id: &str,
) -> Result<(), AppError> {
    let manager = state
        .manager_store
        .find_by_stellar_address(&user.stellar_address)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| {
            AppError::Unauthorized("Manager not found for authenticated user".into())
        })?;
    if manager.id != manager_id {
        return Err(AppError::Unauthorized(
            "You can only access vaults belonging to your own manager account".into(),
        ));
    }
    Ok(())
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListVaultsQuery {
    pub manager_id: String,
}

// ── HTTP handlers ─────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/vaults",
    params(
        ("manager_id" = String, Query, description = "Manager ID to list vaults for")
    ),
    responses(
        (status = 200, description = "List of vaults for the manager", body = Vec<VaultRecord>),
        (status = 401, description = "Unauthorized")
    ),
    tag = "Vaults"
)]
pub async fn list_vaults_handler(
    State(state): State<Arc<crate::AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(query): Query<ListVaultsQuery>,
) -> Result<Json<Vec<VaultRecord>>, AppError> {
    verify_ownership(&state, &user, &query.manager_id).await?;
    let vaults = state.vault_store.list_by_manager(&query.manager_id).await?;
    Ok(Json(vaults))
}

#[utoipa::path(
    post,
    path = "/vaults",
    request_body = CreateVaultRequest,
    responses(
        (status = 200, description = "Vault created", body = VaultRecord),
        (status = 400, description = "Invalid request")
    ),
    tag = "Vaults"
)]
pub async fn create_vault_handler(
    State(state): State<Arc<crate::AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<CreateVaultRequest>,
) -> Result<Json<VaultRecord>, AppError> {
    verify_ownership(&state, &user, &payload.manager_id).await?;
    let approved = state
        .manager_store
        .is_approved(&user.stellar_address)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if !approved {
        return Err(AppError::BadRequest(
            "Manager is not approved. Only approved managers may create vaults. Register via /managers/register and wait for approval.".into(),
        ));
    }
    let vault = state.vault_store.create(&payload).await?;
    crate::audit_log::log_audit_event(&payload.manager_id, "vault_provisioning", &payload.manager_id);
    Ok(Json(vault))
}

#[utoipa::path(
    get,
    path = "/vaults/{id}",
    params(("id" = String, Path, description = "Vault ID")),
    responses(
        (status = 200, description = "Vault record", body = VaultRecord),
        (status = 404, description = "Vault not found")
    ),
    tag = "Vaults"
)]
pub async fn get_vault_handler(
    State(state): State<Arc<crate::AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
) -> Result<Json<VaultRecord>, AppError> {
    let vault = state.vault_store.get(&id).await?;
    verify_ownership(&state, &user, &vault.manager_id).await?;
    Ok(Json(vault))
}

#[utoipa::path(
    patch,
    path = "/vaults/{id}",
    params(("id" = String, Path, description = "Vault ID")),
    request_body = UpdateVaultRequest,
    responses(
        (status = 200, description = "Vault updated (version bumped)", body = VaultRecord),
        (status = 404, description = "Vault not found"),
        (status = 409, description = "Optimistic lock conflict — reload and retry")
    ),
    tag = "Vaults"
)]
pub async fn update_vault_handler(
    State(state): State<Arc<crate::AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateVaultRequest>,
) -> Result<Json<VaultRecord>, AppError> {
    let vault = state.vault_store.get(&id).await?;
    verify_ownership(&state, &user, &vault.manager_id).await?;
    let vault = state.vault_store.update(&id, &payload).await?;
    if payload.config_json.is_some() {
        crate::audit_log::log_audit_event(&vault.manager_id, "fee_split_change", &vault.manager_id);
    } else {
        crate::audit_log::log_audit_event(&vault.manager_id, "vault_update", &vault.manager_id);
    }
    Ok(Json(vault))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn test_store() -> VaultStore {
        let db_name = format!("vault_ol_{}", Uuid::new_v4());
        let url = format!("file:{db_name}?mode=memory&cache=shared");
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await
            .unwrap();
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS vaults (
                id TEXT PRIMARY KEY,
                manager_id TEXT NOT NULL,
                name TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'active',
                config_json TEXT NOT NULL DEFAULT '{}',
                version INTEGER NOT NULL DEFAULT 1,
                idempotency_key TEXT,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();
        VaultStore::new(db::schema::TypedSchema::new(std::sync::Arc::new(pool)).vaults())
    }

    #[tokio::test]
    async fn create_and_get_starts_at_version_one() {
        let store = test_store().await;
        let created = store
            .create(&CreateVaultRequest {
                manager_id: "mgr-1".into(),
                name: "Alpha".into(),
                status: "active".into(),
                config_json: "{}".into(),
                idempotency_key: None,
            })
            .await
            .unwrap();

        assert_eq!(created.version, 1);
        assert_eq!(created.name, "Alpha");

        let fetched = store.get(&created.id).await.unwrap();
        assert_eq!(fetched.version, 1);
    }

    #[tokio::test]
    async fn successful_update_bumps_version() {
        let store = test_store().await;
        let created = store
            .create(&CreateVaultRequest {
                manager_id: "mgr-1".into(),
                name: "Alpha".into(),
                status: "active".into(),
                config_json: "{}".into(),
                idempotency_key: None,
            })
            .await
            .unwrap();

        let updated = store
            .update(
                &created.id,
                &UpdateVaultRequest {
                    version: 1,
                    name: Some("Beta".into()),
                    status: None,
                    config_json: None,
                },
            )
            .await
            .unwrap();

        assert_eq!(updated.version, 2);
        assert_eq!(updated.name, "Beta");
    }

    #[tokio::test]
    async fn stale_version_is_rejected() {
        let store = test_store().await;
        let created = store
            .create(&CreateVaultRequest {
                manager_id: "mgr-1".into(),
                name: "Alpha".into(),
                status: "active".into(),
                config_json: "{}".into(),
                idempotency_key: None,
            })
            .await
            .unwrap();

        store
            .update(
                &created.id,
                &UpdateVaultRequest {
                    version: 1,
                    name: Some("First writer".into()),
                    status: None,
                    config_json: None,
                },
            )
            .await
            .unwrap();

        let err = store
            .update(
                &created.id,
                &UpdateVaultRequest {
                    version: 1, // stale
                    name: Some("Second writer".into()),
                    status: None,
                    config_json: None,
                },
            )
            .await
            .unwrap_err();

        match err {
            VaultStoreError::Conflict {
                expected_version, ..
            } => assert_eq!(expected_version, 1),
            other => panic!("expected Conflict, got {other:?}"),
        }

        let current = store.get(&created.id).await.unwrap();
        assert_eq!(current.version, 2);
        assert_eq!(current.name, "First writer");
    }

    #[tokio::test]
    async fn concurrent_updates_only_one_wins() {
        let store = Arc::new(test_store().await);
        let created = store
            .create(&CreateVaultRequest {
                manager_id: "mgr-1".into(),
                name: "Race".into(),
                status: "active".into(),
                config_json: "{}".into(),
                idempotency_key: None,
            })
            .await
            .unwrap();

        let id = created.id.clone();
        let a = {
            let store = Arc::clone(&store);
            let id = id.clone();
            tokio::spawn(async move {
                store
                    .update(
                        &id,
                        &UpdateVaultRequest {
                            version: 1,
                            name: Some("Writer-A".into()),
                            status: None,
                            config_json: None,
                        },
                    )
                    .await
            })
        };
        let b = {
            let store = Arc::clone(&store);
            let id = id.clone();
            tokio::spawn(async move {
                store
                    .update(
                        &id,
                        &UpdateVaultRequest {
                            version: 1,
                            name: Some("Writer-B".into()),
                            status: None,
                            config_json: None,
                        },
                    )
                    .await
            })
        };

        let (ra, rb) = tokio::join!(a, b);
        let ra = ra.expect("task A join");
        let rb = rb.expect("task B join");

        let wins = [&ra, &rb].iter().filter(|r| r.is_ok()).count();
        let conflicts = [&ra, &rb]
            .iter()
            .filter(|r| matches!(r, Err(VaultStoreError::Conflict { .. })))
            .count();

        assert!(
            wins >= 1,
            "at least one writer must commit; results={ra:?} {rb:?}"
        );
        assert!(
            wins + conflicts == 2,
            "losers must be conflicts; results={ra:?} {rb:?}"
        );

        let final_vault = store.get(&id).await.unwrap();
        assert_eq!(final_vault.version, 1 + wins as i64);
        assert!(final_vault.name == "Writer-A" || final_vault.name == "Writer-B");
    }

    #[tokio::test]
    async fn idempotency_key_returns_same_vault_on_duplicate() {
        let store = test_store().await;
        let req = CreateVaultRequest {
            manager_id: "mgr-1".into(),
            name: "Alpha".into(),
            status: "active".into(),
            config_json: "{}".into(),
            idempotency_key: Some("key-abc".into()),
        };

        let first = store.create(&req).await.unwrap();
        let second = store.create(&req).await.unwrap();

        assert_eq!(first.id, second.id);
        assert_eq!(first.name, second.name);

        // Only one vault should exist for this manager.
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM vaults WHERE manager_id = ?1")
            .bind("mgr-1")
            .fetch_one(store.vaults.pool())
            .await
            .unwrap();
        assert_eq!(count.0, 1);
    }

    #[tokio::test]
    async fn different_idempotency_keys_create_separate_vaults() {
        let store = test_store().await;

        let a = store
            .create(&CreateVaultRequest {
                manager_id: "mgr-1".into(),
                name: "Alpha".into(),
                status: "active".into(),
                config_json: "{}".into(),
                idempotency_key: Some("key-a".into()),
            })
            .await
            .unwrap();

        let b = store
            .create(&CreateVaultRequest {
                manager_id: "mgr-1".into(),
                name: "Beta".into(),
                status: "active".into(),
                config_json: "{}".into(),
                idempotency_key: Some("key-b".into()),
            })
            .await
            .unwrap();

        assert_ne!(a.id, b.id);
        assert_eq!(a.name, "Alpha");
        assert_eq!(b.name, "Beta");

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM vaults WHERE manager_id = ?1")
            .bind("mgr-1")
            .fetch_one(store.vaults.pool())
            .await
            .unwrap();
        assert_eq!(count.0, 2);
    }

    #[tokio::test]
    async fn create_without_idempotency_key_allows_duplicates() {
        let store = test_store().await;

        let a = store
            .create(&CreateVaultRequest {
                manager_id: "mgr-1".into(),
                name: "Alpha".into(),
                status: "active".into(),
                config_json: "{}".into(),
                idempotency_key: None,
            })
            .await
            .unwrap();

        let b = store
            .create(&CreateVaultRequest {
                manager_id: "mgr-1".into(),
                name: "Alpha".into(),
                status: "active".into(),
                config_json: "{}".into(),
                idempotency_key: None,
            })
            .await
            .unwrap();

        assert_ne!(a.id, b.id);

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM vaults WHERE manager_id = ?1")
            .bind("mgr-1")
            .fetch_one(store.vaults.pool())
            .await
            .unwrap();
        assert_eq!(count.0, 2);
    }
}
