//! White-label vault records with optimistic locking (API-37).
//!
//! Concurrent updates to the same vault must supply the current `version`.
//! The store bumps `version` only when `WHERE id = ? AND version = ?` matches;
//! otherwise the caller gets a conflict and must reload.

use crate::errors::AppError;
use axum::{
    extract::{Path, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
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

/// Persisted vault record. `version` is the optimistic-lock token.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct VaultRecord {
    pub id: String,
    pub manager_id: String,
    pub name: String,
    pub status: String,
    pub config_json: String,
    pub version: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateVaultRequest {
    pub manager_id: String,
    pub name: String,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default = "default_config")]
    pub config_json: String,
}

fn default_status() -> String {
    "active".to_string()
}

fn default_config() -> String {
    "{}".to_string()
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateVaultRequest {
    /// Required optimistic-lock version from the last GET/create response.
    pub version: i64,
    pub name: Option<String>,
    pub status: Option<String>,
    pub config_json: Option<String>,
}

pub struct VaultStore {
    pool: SqlitePool,
}

impl VaultStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
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

        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO vaults (id, manager_id, name, status, config_json, version, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7)
            "#,
        )
        .bind(&id)
        .bind(req.manager_id.trim())
        .bind(req.name.trim())
        .bind(req.status.trim())
        .bind(&req.config_json)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        self.get(&id).await
    }

    pub async fn get(&self, id: &str) -> Result<VaultRecord, VaultStoreError> {
        sqlx::query_as::<_, VaultRecord>(
            r#"
            SELECT id, manager_id, name, status, config_json, version, created_at, updated_at
            FROM vaults
            WHERE id = ?1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| VaultStoreError::NotFound(id.to_string()))
    }

    /// Apply an update inside a transaction using optimistic locking.
    ///
    /// `UPDATE … WHERE id = ? AND version = ?` guarantees only one concurrent
    /// writer succeeds for a given version snapshot.
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

        let mut tx = self.pool.begin().await?;

        // Ensure the row exists before attempting the CAS update.
        let exists: Option<(String,)> =
            sqlx::query_as(r#"SELECT id FROM vaults WHERE id = ?1"#)
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?;
        if exists.is_none() {
            return Err(VaultStoreError::NotFound(id.to_string()));
        }

        let name = req.name.as_deref().map(str::trim).filter(|s| !s.is_empty());
        let status = req
            .status
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());
        let config_json = req.config_json.as_deref();
        let now = Utc::now();

        // Single atomic CAS: bump version only when the expected version still matches.
        let result = sqlx::query(
            r#"
            UPDATE vaults
            SET name = COALESCE(?1, name),
                status = COALESCE(?2, status),
                config_json = COALESCE(?3, config_json),
                version = version + 1,
                updated_at = ?4
            WHERE id = ?5 AND version = ?6
            "#,
        )
        .bind(name)
        .bind(status)
        .bind(config_json)
        .bind(now)
        .bind(id)
        .bind(req.version)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() != 1 {
            return Err(VaultStoreError::Conflict {
                vault_id: id.to_string(),
                expected_version: req.version,
            });
        }

        tx.commit().await?;

        self.get(id).await
    }
}

// ── HTTP handlers ─────────────────────────────────────────────────────────────

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
    Json(payload): Json<CreateVaultRequest>,
) -> Result<Json<VaultRecord>, AppError> {
    let vault = state.vault_store.create(&payload).await?;
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
    Path(id): Path<String>,
) -> Result<Json<VaultRecord>, AppError> {
    let vault = state.vault_store.get(&id).await?;
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
    Path(id): Path<String>,
    Json(payload): Json<UpdateVaultRequest>,
) -> Result<Json<VaultRecord>, AppError> {
    let vault = state.vault_store.update(&id, &payload).await?;
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
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();
        VaultStore::new(pool)
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
}
