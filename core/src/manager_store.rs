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
pub enum ManagerStoreError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Manager not found: {0}")]
    NotFound(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Stellar address already registered: {0}")]
    DuplicateAddress(String),
}

impl From<ManagerStoreError> for AppError {
    fn from(err: ManagerStoreError) -> Self {
        match err {
            ManagerStoreError::NotFound(msg) => AppError::NotFound(msg),
            ManagerStoreError::InvalidData(msg) => AppError::BadRequest(msg),
            ManagerStoreError::DuplicateAddress(msg) => AppError::Conflict(msg),
            ManagerStoreError::Database(e) => AppError::Internal(e.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ManagerRecord {
    pub id: String,
    pub stellar_address: String,
    pub name: String,
    pub email: String,
    pub status: String,
    pub kyc_document_ref: String,
    pub notes: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterManagerRequest {
    pub stellar_address: String,
    pub name: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub kyc_document_ref: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ApproveManagerRequest {
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ManagerStatusResponse {
    pub id: String,
    pub status: String,
    pub message: String,
}

pub struct ManagerStore {
    pool: SqlitePool,
}

impl ManagerStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn register(&self, req: &RegisterManagerRequest) -> Result<ManagerRecord, ManagerStoreError> {
        if req.stellar_address.trim().is_empty() {
            return Err(ManagerStoreError::InvalidData(
                "stellar_address must not be empty".into(),
            ));
        }
        if req.name.trim().is_empty() {
            return Err(ManagerStoreError::InvalidData(
                "name must not be empty".into(),
            ));
        }

        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let result = sqlx::query(
            r#"
            INSERT INTO managers (id, stellar_address, name, email, status, kyc_document_ref, notes, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, 'pending', ?5, '', ?6, ?7)
            "#,
        )
        .bind(&id)
        .bind(req.stellar_address.trim())
        .bind(req.name.trim())
        .bind(req.email.trim())
        .bind(&req.kyc_document_ref)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => self.get(&id).await,
            Err(sqlx::Error::Database(db_err)) => {
                if db_err.message().contains("UNIQUE") {
                    Err(ManagerStoreError::DuplicateAddress(
                        req.stellar_address.trim().to_string(),
                    ))
                } else {
                    Err(ManagerStoreError::Database(sqlx::Error::Database(db_err)))
                }
            }
            Err(e) => Err(ManagerStoreError::Database(e)),
        }
    }

    pub async fn get(&self, id: &str) -> Result<ManagerRecord, ManagerStoreError> {
        sqlx::query_as::<_, ManagerRecord>(
            r#"
            SELECT id, stellar_address, name, email, status, kyc_document_ref, notes, created_at, updated_at
            FROM managers
            WHERE id = ?1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ManagerStoreError::NotFound(id.to_string()))
    }

    pub async fn find_by_stellar_address(&self, address: &str) -> Result<Option<ManagerRecord>, ManagerStoreError> {
        let record = sqlx::query_as::<_, ManagerRecord>(
            r#"
            SELECT id, stellar_address, name, email, status, kyc_document_ref, notes, created_at, updated_at
            FROM managers
            WHERE stellar_address = ?1
            "#,
        )
        .bind(address)
        .fetch_optional(&self.pool)
        .await?;
        Ok(record)
    }

    pub async fn is_approved(&self, stellar_address: &str) -> Result<bool, ManagerStoreError> {
        let record = self.find_by_stellar_address(stellar_address).await?;
        match record {
            Some(r) => Ok(r.status == "approved"),
            None => Ok(false),
        }
    }

    pub async fn list(&self, status_filter: Option<&str>) -> Result<Vec<ManagerRecord>, ManagerStoreError> {
        let records = if let Some(status) = status_filter {
            sqlx::query_as::<_, ManagerRecord>(
                r#"
                SELECT id, stellar_address, name, email, status, kyc_document_ref, notes, created_at, updated_at
                FROM managers
                WHERE status = ?1
                ORDER BY created_at DESC
                "#,
            )
            .bind(status)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, ManagerRecord>(
                r#"
                SELECT id, stellar_address, name, email, status, kyc_document_ref, notes, created_at, updated_at
                FROM managers
                ORDER BY created_at DESC
                "#,
            )
            .fetch_all(&self.pool)
            .await?
        };
        Ok(records)
    }

    pub async fn approve(
        &self,
        id: &str,
        req: &ApproveManagerRequest,
    ) -> Result<ManagerRecord, ManagerStoreError> {
        let now = Utc::now();
        let result = sqlx::query(
            r#"
            UPDATE managers
            SET status = 'approved', notes = ?1, updated_at = ?2
            WHERE id = ?3 AND status = 'pending'
            "#,
        )
        .bind(&req.notes)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            let record = self.get(id).await?;
            if record.status == "approved" {
                return Err(ManagerStoreError::InvalidData(
                    "Manager is already approved".into(),
                ));
            }
            if record.status == "rejected" {
                return Err(ManagerStoreError::InvalidData(
                    "Cannot approve a rejected manager".into(),
                ));
            }
            return Err(ManagerStoreError::NotFound(id.to_string()));
        }

        self.get(id).await
    }

    pub async fn reject(
        &self,
        id: &str,
        req: &ApproveManagerRequest,
    ) -> Result<ManagerRecord, ManagerStoreError> {
        let now = Utc::now();
        let result = sqlx::query(
            r#"
            UPDATE managers
            SET status = 'rejected', notes = ?1, updated_at = ?2
            WHERE id = ?3 AND status = 'pending'
            "#,
        )
        .bind(&req.notes)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            let record = self.get(id).await?;
            if record.status == "rejected" {
                return Err(ManagerStoreError::InvalidData(
                    "Manager is already rejected".into(),
                ));
            }
            if record.status == "approved" {
                return Err(ManagerStoreError::InvalidData(
                    "Cannot reject an approved manager".into(),
                ));
            }
            return Err(ManagerStoreError::NotFound(id.to_string()));
        }

        self.get(id).await
    }
}

#[utoipa::path(
    post,
    path = "/managers/register",
    request_body = RegisterManagerRequest,
    responses(
        (status = 200, description = "Manager registered (pending approval)", body = ManagerRecord),
        (status = 400, description = "Invalid request"),
        (status = 409, description = "Stellar address already registered")
    ),
    tag = "Managers"
)]
pub async fn register_manager_handler(
    State(state): State<Arc<crate::AppState>>,
    Json(payload): Json<RegisterManagerRequest>,
) -> Result<Json<ManagerRecord>, AppError> {
    let manager = state.manager_store.register(&payload).await?;
    Ok(Json(manager))
}

#[utoipa::path(
    get,
    path = "/managers",
    params(
        ("status" = Option<String>, Query, description = "Filter by status: pending, approved, rejected")
    ),
    responses(
        (status = 200, description = "List of manager records", body = Vec<ManagerRecord>)
    ),
    tag = "Managers"
)]
pub async fn list_managers_handler(
    State(state): State<Arc<crate::AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<ManagerRecord>>, AppError> {
    let status_filter = params.get("status").map(|s| s.as_str());
    let managers = state.manager_store.list(status_filter).await?;
    Ok(Json(managers))
}

#[utoipa::path(
    get,
    path = "/managers/{id}",
    params(("id" = String, Path, description = "Manager ID")),
    responses(
        (status = 200, description = "Manager record", body = ManagerRecord),
        (status = 404, description = "Manager not found")
    ),
    tag = "Managers"
)]
pub async fn get_manager_handler(
    State(state): State<Arc<crate::AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ManagerRecord>, AppError> {
    let manager = state.manager_store.get(&id).await?;
    Ok(Json(manager))
}

#[utoipa::path(
    post,
    path = "/managers/{id}/approve",
    params(("id" = String, Path, description = "Manager ID")),
    request_body = ApproveManagerRequest,
    responses(
        (status = 200, description = "Manager approved", body = ManagerRecord),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "Manager not found")
    ),
    tag = "Managers"
)]
pub async fn approve_manager_handler(
    State(state): State<Arc<crate::AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<ApproveManagerRequest>,
) -> Result<Json<ManagerRecord>, AppError> {
    let manager = state.manager_store.approve(&id, &payload).await?;
    Ok(Json(manager))
}

#[utoipa::path(
    post,
    path = "/managers/{id}/reject",
    params(("id" = String, Path, description = "Manager ID")),
    request_body = ApproveManagerRequest,
    responses(
        (status = 200, description = "Manager rejected", body = ManagerRecord),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "Manager not found")
    ),
    tag = "Managers"
)]
pub async fn reject_manager_handler(
    State(state): State<Arc<crate::AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<ApproveManagerRequest>,
) -> Result<Json<ManagerRecord>, AppError> {
    let manager = state.manager_store.reject(&id, &payload).await?;
    Ok(Json(manager))
}

#[utoipa::path(
    get,
    path = "/managers/status/{stellar_address}",
    params(("stellar_address" = String, Path, description = "Stellar address")),
    responses(
        (status = 200, description = "Manager approval status", body = ManagerStatusResponse)
    ),
    tag = "Managers"
)]
pub async fn check_manager_status_handler(
    State(state): State<Arc<crate::AppState>>,
    Path(stellar_address): Path<String>,
) -> Result<Json<ManagerStatusResponse>, AppError> {
    let record = state
        .manager_store
        .find_by_stellar_address(&stellar_address)
        .await?;
    match record {
        Some(m) => Ok(Json(ManagerStatusResponse {
            id: m.id,
            status: m.status,
            message: match m.status.as_str() {
                "approved" => "Manager is approved and active".into(),
                "rejected" => "Manager registration was rejected".into(),
                _ => "Manager registration is pending approval".into(),
            },
        })),
        None => Ok(Json(ManagerStatusResponse {
            id: String::new(),
            status: "unregistered".into(),
            message: "No manager registration found for this address".into(),
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn test_store() -> ManagerStore {
        let url = "sqlite::memory:".to_string();
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await
            .unwrap();
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS managers (
                id TEXT PRIMARY KEY,
                stellar_address TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                email TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'pending'
                    CHECK (status IN ('pending', 'approved', 'rejected')),
                kyc_document_ref TEXT NOT NULL DEFAULT '',
                notes TEXT NOT NULL DEFAULT '',
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();
        ManagerStore::new(pool)
    }

    #[tokio::test]
    async fn register_and_get() {
        let store = test_store().await;
        let created = store
            .register(&RegisterManagerRequest {
                stellar_address: "GABC123".into(),
                name: "Alice".into(),
                email: "alice@example.com".into(),
                kyc_document_ref: "doc-123".into(),
            })
            .await
            .unwrap();

        assert_eq!(created.status, "pending");
        assert_eq!(created.name, "Alice");

        let fetched = store.get(&created.id).await.unwrap();
        assert_eq!(fetched.stellar_address, "GABC123");
    }

    #[tokio::test]
    async fn duplicate_address_rejected() {
        let store = test_store().await;
        store
            .register(&RegisterManagerRequest {
                stellar_address: "GABC123".into(),
                name: "Alice".into(),
                email: "".into(),
                kyc_document_ref: "".into(),
            })
            .await
            .unwrap();

        let err = store
            .register(&RegisterManagerRequest {
                stellar_address: "GABC123".into(),
                name: "Bob".into(),
                email: "".into(),
                kyc_document_ref: "".into(),
            })
            .await
            .unwrap_err();

        match err {
            ManagerStoreError::DuplicateAddress(addr) => assert_eq!(addr, "GABC123"),
            other => panic!("expected DuplicateAddress, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn approve_pending_manager() {
        let store = test_store().await;
        let created = store
            .register(&RegisterManagerRequest {
                stellar_address: "GABC123".into(),
                name: "Alice".into(),
                email: "".into(),
                kyc_document_ref: "".into(),
            })
            .await
            .unwrap();

        let approved = store
            .approve(&created.id, &ApproveManagerRequest { notes: "KYC verified".into() })
            .await
            .unwrap();
        assert_eq!(approved.status, "approved");
        assert_eq!(approved.notes, "KYC verified");

        assert!(store.is_approved("GABC123").await.unwrap());
    }

    #[tokio::test]
    async fn reject_pending_manager() {
        let store = test_store().await;
        let created = store
            .register(&RegisterManagerRequest {
                stellar_address: "GABC123".into(),
                name: "Alice".into(),
                email: "".into(),
                kyc_document_ref: "".into(),
            })
            .await
            .unwrap();

        let rejected = store
            .reject(&created.id, &ApproveManagerRequest { notes: "Failed KYC".into() })
            .await
            .unwrap();
        assert_eq!(rejected.status, "rejected");

        assert!(!store.is_approved("GABC123").await.unwrap());
    }

    #[tokio::test]
    async fn list_filters_by_status() {
        let store = test_store().await;
        store
            .register(&RegisterManagerRequest {
                stellar_address: "GA1".into(),
                name: "A".into(),
                email: "".into(),
                kyc_document_ref: "".into(),
            })
            .await
            .unwrap();
        let m2 = store
            .register(&RegisterManagerRequest {
                stellar_address: "GA2".into(),
                name: "B".into(),
                email: "".into(),
                kyc_document_ref: "".into(),
            })
            .await
            .unwrap();
        store
            .approve(&m2.id, &ApproveManagerRequest { notes: "".into() })
            .await
            .unwrap();

        let all = store.list(None).await.unwrap();
        assert_eq!(all.len(), 2);

        let pending = store.list(Some("pending")).await.unwrap();
        assert_eq!(pending.len(), 1);

        let approved = store.list(Some("approved")).await.unwrap();
        assert_eq!(approved.len(), 1);
    }
}
