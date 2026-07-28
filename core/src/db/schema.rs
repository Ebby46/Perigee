use sqlx::SqlitePool;
use std::sync::Arc;

pub type DbPool = SqlitePool;

/// Shared typed schema that provides table-level typed repositories.
pub struct TypedSchema {
    pool: Arc<DbPool>,
}

impl TypedSchema {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    pub fn managers(&self) -> ManagersTable {
        ManagersTable {
            pool: Arc::clone(&self.pool),
        }
    }

    pub fn vaults(&self) -> VaultsTable {
        VaultsTable {
            pool: Arc::clone(&self.pool),
        }
    }

    pub fn reconciliation_reports(&self) -> ReconciliationReportsTable {
        ReconciliationReportsTable {
            pool: Arc::clone(&self.pool),
        }
    }

    pub fn reconciliation_discrepancies(&self) -> ReconciliationDiscrepanciesTable {
        ReconciliationDiscrepanciesTable {
            pool: Arc::clone(&self.pool),
        }
    }
}

pub struct ManagersTable {
    pool: Arc<DbPool>,
}

impl ManagersTable {
    pub fn pool(&self) -> &DbPool {
        &self.pool
    }

    pub async fn find_by_id(
        &self,
        id: &str,
    ) -> Result<Option<crate::db::models::ManagerRecord>, sqlx::Error> {
        sqlx::query_as::<_, crate::db::models::ManagerRecord>(
            "SELECT id, stellar_address, name, email, status, kyc_document_ref, notes, created_at, updated_at FROM managers WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&*self.pool)
        .await
    }

    pub async fn find_by_stellar_address(
        &self,
        address: &str,
    ) -> Result<Option<crate::db::models::ManagerRecord>, sqlx::Error> {
        sqlx::query_as::<_, crate::db::models::ManagerRecord>(
            "SELECT id, stellar_address, name, email, status, kyc_document_ref, notes, created_at, updated_at FROM managers WHERE stellar_address = ?1",
        )
        .bind(address)
        .fetch_optional(&*self.pool)
        .await
    }

    pub async fn list(
        &self,
        status_filter: Option<&str>,
    ) -> Result<Vec<crate::db::models::ManagerRecord>, sqlx::Error> {
        if let Some(status) = status_filter {
            sqlx::query_as::<_, crate::db::models::ManagerRecord>(
                "SELECT id, stellar_address, name, email, status, kyc_document_ref, notes, created_at, updated_at FROM managers WHERE status = ?1 ORDER BY created_at DESC",
            )
            .bind(status)
            .fetch_all(&*self.pool)
            .await
        } else {
            sqlx::query_as::<_, crate::db::models::ManagerRecord>(
                "SELECT id, stellar_address, name, email, status, kyc_document_ref, notes, created_at, updated_at FROM managers ORDER BY created_at DESC",
            )
            .fetch_all(&*self.pool)
            .await
        }
    }

    pub async fn insert(
        &self,
        id: &str,
        stellar_address: &str,
        name: &str,
        email: &str,
        kyc_document_ref: &str,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<crate::db::models::ManagerRecord, sqlx::Error> {
        sqlx::query(
            "INSERT INTO managers (id, stellar_address, name, email, status, kyc_document_ref, notes, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, 'pending', ?5, '', ?6, ?7)",
        )
        .bind(id)
        .bind(stellar_address)
        .bind(name)
        .bind(email)
        .bind(kyc_document_ref)
        .bind(now)
        .bind(now)
        .execute(&*self.pool)
        .await?;

        self.find_by_id(id).await?.ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn update_status(
        &self,
        id: &str,
        status: &str,
        notes: &str,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<crate::db::models::ManagerRecord, sqlx::Error> {
        sqlx::query(
            "UPDATE managers SET status = ?1, notes = ?2, updated_at = ?3 WHERE id = ?4 AND status = 'pending'",
        )
        .bind(status)
        .bind(notes)
        .bind(now)
        .bind(id)
        .execute(&*self.pool)
        .await?;

        self.find_by_id(id).await?.ok_or(sqlx::Error::RowNotFound)
    }
}

pub struct VaultsTable {
    pool: Arc<DbPool>,
}

impl VaultsTable {
    pub fn pool(&self) -> &DbPool {
        &self.pool
    }

    pub async fn find_by_id(
        &self,
        id: &str,
    ) -> Result<Option<crate::db::models::VaultRecord>, sqlx::Error> {
        sqlx::query_as::<_, crate::db::models::VaultRecord>(
            "SELECT id, manager_id, name, status, config_json, version, idempotency_key, created_at, updated_at FROM vaults WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&*self.pool)
        .await
    }

    pub async fn find_by_idempotency_key(
        &self,
        manager_id: &str,
        key: &str,
    ) -> Result<Option<crate::db::models::VaultRecord>, sqlx::Error> {
        sqlx::query_as::<_, crate::db::models::VaultRecord>(
            "SELECT id, manager_id, name, status, config_json, version, idempotency_key, created_at, updated_at FROM vaults WHERE manager_id = ?1 AND idempotency_key = ?2",
        )
        .bind(manager_id)
        .bind(key)
        .fetch_optional(&*self.pool)
        .await
    }

    pub async fn insert(
        &self,
        id: &str,
        manager_id: &str,
        name: &str,
        status: &str,
        config_json: &str,
        idempotency_key: Option<&str>,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<crate::db::models::VaultRecord, sqlx::Error> {
        sqlx::query(
            "INSERT INTO vaults (id, manager_id, name, status, config_json, version, idempotency_key, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7, ?8)",
        )
        .bind(id)
        .bind(manager_id)
        .bind(name)
        .bind(status)
        .bind(config_json)
        .bind(idempotency_key)
        .bind(now)
        .bind(now)
        .execute(&*self.pool)
        .await?;

        self.find_by_id(id).await?.ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn update(
        &self,
        id: &str,
        expected_version: i64,
        name: Option<&str>,
        status: Option<&str>,
        config_json: Option<&str>,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<crate::db::models::VaultRecord, sqlx::Error> {
        sqlx::query(
            "UPDATE vaults SET name = COALESCE(?1, name), status = COALESCE(?2, status), config_json = COALESCE(?3, config_json), version = version + 1, updated_at = ?4 WHERE id = ?5 AND version = ?6",
        )
        .bind(name)
        .bind(status)
        .bind(config_json)
        .bind(now)
        .bind(id)
        .bind(expected_version)
        .execute(&*self.pool)
        .await?;

        self.find_by_id(id).await?.ok_or(sqlx::Error::RowNotFound)
    }
}

pub struct ReconciliationReportsTable {
    pool: Arc<DbPool>,
}

impl ReconciliationReportsTable {
    pub fn pool(&self) -> &DbPool {
        &self.pool
    }

    pub async fn find_by_id(
        &self,
        id: &str,
    ) -> Result<Option<crate::db::models::ReconciliationReport>, sqlx::Error> {
        let row = sqlx::query_as::<_, (
            String, i64, i64, f64, i32, i32, f64, f64, Option<serde_json::Value>, String,
        )>(
            "SELECT id, from_ledger, to_ledger, tolerance_pct, total_ledgers, discrepancies_count, avg_delta_pct, max_delta_pct, summary, created_at FROM reconciliation_reports WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&*self.pool)
        .await?;

        Ok(row.map(|r| crate::db::models::ReconciliationReport {
            id: r.0,
            from_ledger: r.1,
            to_ledger: r.2,
            tolerance_pct: r.3,
            total_ledgers: r.4,
            discrepancies_count: r.5,
            avg_delta_pct: r.6,
            max_delta_pct: r.7,
            summary: r.8.and_then(|v| serde_json::from_value(v).ok()),
            created_at: r.9,
        }))
    }

    pub async fn list(
        &self,
        limit: i64,
    ) -> Result<Vec<crate::db::models::ReconciliationReport>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (
            String, i64, i64, f64, i32, i32, f64, f64, Option<serde_json::Value>, String,
        )>(
            "SELECT id, from_ledger, to_ledger, tolerance_pct, total_ledgers, discrepancies_count, avg_delta_pct, max_delta_pct, summary, created_at FROM reconciliation_reports ORDER BY created_at DESC LIMIT ?1",
        )
        .bind(limit)
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| crate::db::models::ReconciliationReport {
            id: r.0,
            from_ledger: r.1,
            to_ledger: r.2,
            tolerance_pct: r.3,
            total_ledgers: r.4,
            discrepancies_count: r.5,
            avg_delta_pct: r.6,
            max_delta_pct: r.7,
            summary: r.8.and_then(|v| serde_json::from_value(v).ok()),
            created_at: r.9,
        }).collect())
    }

    pub async fn insert(
        &self,
        id: &str,
        from_ledger: i64,
        to_ledger: i64,
        tolerance_pct: f64,
        total_ledgers: i32,
        discrepancies_count: i32,
        avg_delta_pct: f64,
        max_delta_pct: f64,
        summary: Option<&serde_json::Value>,
        created_at: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO reconciliation_reports (id, from_ledger, to_ledger, tolerance_pct, total_ledgers, discrepancies_count, avg_delta_pct, max_delta_pct, summary, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        )
        .bind(id)
        .bind(from_ledger)
        .bind(to_ledger)
        .bind(tolerance_pct)
        .bind(total_ledgers)
        .bind(discrepancies_count)
        .bind(avg_delta_pct)
        .bind(max_delta_pct)
        .bind(summary)
        .bind(created_at)
        .execute(&*self.pool)
        .await?;

        Ok(())
    }
}

pub struct ReconciliationDiscrepanciesTable {
    pool: Arc<DbPool>,
}

impl ReconciliationDiscrepanciesTable {
    pub fn pool(&self) -> &DbPool {
        &self.pool
    }

    pub async fn insert_for_report(
        &self,
        report_id: &str,
        discrepancies: &[crate::db::models::Discrepancy],
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        for disc in discrepancies {
            sqlx::query(
                "INSERT INTO reconciliation_discrepancies (id, report_id, ledger_sequence, expected_fee, actual_fee, delta, delta_pct, severity) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            )
            .bind(&disc.id)
            .bind(report_id)
            .bind(disc.ledger_sequence)
            .bind(disc.expected_fee)
            .bind(disc.actual_fee)
            .bind(disc.delta)
            .bind(disc.delta_pct)
            .bind(&disc.severity)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_report_id(
        &self,
        report_id: &str,
    ) -> Result<Vec<crate::db::models::Discrepancy>, sqlx::Error> {
        sqlx::query_as::<_, crate::db::models::Discrepancy>(
            "SELECT id, report_id, ledger_sequence, expected_fee, actual_fee, delta, delta_pct, severity FROM reconciliation_discrepancies WHERE report_id = ?1 ORDER BY ledger_sequence",
        )
        .bind(report_id)
        .fetch_all(&*self.pool)
        .await
    }
}