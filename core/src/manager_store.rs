use crate::errors::AppError;
use crate::db;
use axum::{
    extract::{Path, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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

/// Manager DTOs live in [`crate::db::models`]; the typed DB layer returns them
/// directly. They were duplicated here field-for-field, so every method that
/// forwarded a row from the DB returned a type the signature did not accept.
pub use crate::db::models::{
    ApproveManagerRequest, ManagerRecord, ManagerStatusResponse, RegisterManagerRequest,
};

pub struct ManagerStore {
    managers: db::schema::ManagersTable,
}

impl ManagerStore {
    pub fn new(managers: db::schema::ManagersTable) -> Self {
        Self { managers }
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
        let stellar = req.stellar_address.trim();
        let name = req.name.trim();
        let email = req.email.trim();
        let kyc = req.kyc_document_ref.trim();

        self.managers
            .insert(&id, stellar, name, email, kyc, now)
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(db_err) => {
                    if db_err.message().contains("UNIQUE") {
                        ManagerStoreError::DuplicateAddress(stellar.to_string())
                    } else {
                        ManagerStoreError::Database(sqlx::Error::Database(db_err))
                    }
                }
                other => ManagerStoreError::Database(other),
            })
    }

    pub async fn get(&self, id: &str) -> Result<ManagerRecord, ManagerStoreError> {
        self.managers
            .find_by_id(id)
            .await
            .map_err(ManagerStoreError::Database)?
            .ok_or_else(|| ManagerStoreError::NotFound(id.to_string()))
    }

    pub async fn find_by_stellar_address(&self, address: &str) -> Result<Option<ManagerRecord>, ManagerStoreError> {
        self.managers
            .find_by_stellar_address(address)
            .await
            .map_err(ManagerStoreError::Database)
    }

    pub async fn is_approved(&self, stellar_address: &str) -> Result<bool, ManagerStoreError> {
        let record = self.find_by_stellar_address(stellar_address).await?;
        match record {
            Some(r) => Ok(r.status == "approved"),
            None => Ok(false),
        }
    }

    pub async fn list(&self, status_filter: Option<&str>) -> Result<Vec<ManagerRecord>, ManagerStoreError> {
        self.managers
            .list(status_filter)
            .await
            .map_err(ManagerStoreError::Database)
    }

    pub async fn approve(
        &self,
        id: &str,
        req: &ApproveManagerRequest,
    ) -> Result<ManagerRecord, ManagerStoreError> {
        self.managers
            .update_status(id, "approved", &req.notes, Utc::now())
            .await
            .map_err(ManagerStoreError::Database)
    }

    pub async fn reject(
        &self,
        id: &str,
        req: &ApproveManagerRequest,
    ) -> Result<ManagerRecord, ManagerStoreError> {
        self.managers
            .update_status(id, "rejected", &req.notes, Utc::now())
            .await
            .map_err(ManagerStoreError::Database)
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
        Some(m) => {
            // Derive the message before moving `status` into the response.
            let message: String = match m.status.as_str() {
                "approved" => "Manager is approved and active".into(),
                "rejected" => "Manager registration was rejected".into(),
                _ => "Manager registration is pending approval".into(),
            };

            Ok(Json(ManagerStatusResponse {
                id: m.id,
                status: m.status,
                message,
            }))
        }
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
        ManagerStore::new(db::schema::TypedSchema::new(std::sync::Arc::new(pool)).managers())
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
