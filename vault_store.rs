use crate::errors::AppError;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;
use utoipa::ToSchema;

/// The maximum permissible fee markup in basis points (bps).
/// 100 bps = 1%.
/// A value of 500 means a 5% maximum markup.
pub const MAX_MARKUP_BPS: i64 = 500;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct VaultRecord {
    pub id: String,
    pub client_name: String,
    /// Fee markup in basis points.
    pub markup_bps: i64,
    pub stellar_address: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Optimistic locking version.
    pub version: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateVaultRequest {
    pub client_name: String,
    /// Fee markup in basis points (0-500).
    #[schema(example = 100)]
    pub markup_bps: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateVaultRequest {
    pub client_name: Option<String>,
    pub markup_bps: Option<i64>,
    /// Required for optimistic locking.
    pub version: i64,
}

#[derive(Clone)]
pub struct VaultStore {
    pool: SqlitePool,
}

impl VaultStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, req: &CreateVaultRequest) -> Result<VaultRecord, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        // In a real scenario, this would be derived from a key or provided.
        let stellar_address = format!("GVAULT_{}", id.split('-').next().unwrap().to_uppercase());

        let record = sqlx::query_as::<_, VaultRecord>(
            r#"
            INSERT INTO vaults (id, client_name, markup_bps, stellar_address, version)
            VALUES (?1, ?2, ?3, ?4, 1)
            RETURNING *
            "#,
        )
        .bind(&id)
        .bind(&req.client_name)
        .bind(req.markup_bps)
        .bind(&stellar_address)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create vault: {}", e)))?;

        Ok(record)
    }
}

/// Handler to create a new white-label vault record.
#[utoipa::path(
    post,
    path = "/vaults",
    request_body = CreateVaultRequest,
    responses(
        (status = 201, description = "Vault created successfully", body = VaultRecord),
        (status = 400, description = "Invalid input, e.g., markup exceeds ceiling"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Vaults"
)]
pub async fn create_vault_handler(
    State(state): State<Arc<crate::AppState>>,
    Json(payload): Json<CreateVaultRequest>,
) -> Result<(StatusCode, Json<VaultRecord>), AppError> {
    // --- SERVER-SIDE VALIDATION ---
    if payload.client_name.is_empty() {
        return Err(AppError::BadRequest("Client name cannot be empty.".into()));
    }
    if payload.markup_bps < 0 {
        return Err(AppError::BadRequest("Markup cannot be negative.".into()));
    }
    if payload.markup_bps > MAX_MARKUP_BPS {
        return Err(AppError::BadRequest(format!(
            "Markup cannot exceed {} basis points.",
            MAX_MARKUP_BPS
        )));
    }

    let vault = state.vault_store.create(&payload).await?;
    Ok((StatusCode::CREATED, Json(vault)))
}

/// Handler to get a vault record by ID.
#[utoipa::path(
    get,
    path = "/vaults/{id}",
    params(("id" = String, Path, description = "Vault ID")),
    responses(
        (status = 200, body = VaultRecord),
        (status = 404, description = "Vault not found")
    ),
    tag = "Vaults"
)]
pub async fn get_vault_handler(
    State(_state): State<Arc<crate::AppState>>,
    Path(_id): Path<String>,
) -> Result<Json<VaultRecord>, AppError> {
    // Placeholder: Implementation for fetching a vault would go here.
    Err(AppError::NotFound("Not implemented".into()))
}

/// Handler to update a vault record.
#[utoipa::path(
    patch,
    path = "/vaults/{id}",
    request_body = UpdateVaultRequest,
    params(("id" = String, Path, description = "Vault ID")),
    responses(
        (status = 200, body = VaultRecord),
        (status = 400, description = "Invalid input or version mismatch"),
        (status = 404, description = "Vault not found")
    ),
    tag = "Vaults"
)]
pub async fn update_vault_handler(
    State(_state): State<Arc<crate::AppState>>,
    Path(_id): Path<String>,
    Json(_payload): Json<UpdateVaultRequest>,
) -> Result<Json<VaultRecord>, AppError> {
    // Placeholder: Implementation for updating a vault would go here.
    Err(AppError::NotFound("Not implemented".into()))
}