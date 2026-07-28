pub mod models;
pub mod schema;

use std::sync::Arc;
use sqlx::SqlitePool;
use crate::db::schema::{
    ManagersTable,
    ReconciliationDiscrepanciesTable,
    ReconciliationReportsTable,
    TypedSchema,
    VaultsTable,
};
use chrono::Utc;
use uuid::Uuid;

pub type Pool = SqlitePool;

pub async fn init_pool(database_url: &str) -> Result<Pool, sqlx::Error> {
    let pool = SqlitePool::connect(database_url).await?;
    sqlx::migrate!().run(&pool).await?;
    Ok(pool)
}

pub fn make_typed_schema(pool: Arc<SqlitePool>) -> TypedSchema {
    TypedSchema::new(pool)
}

pub mod manager {
    use super::*;
    use crate::db::models;

    pub struct ManagerRepo {
        table: ManagersTable,
    }

    impl ManagerRepo {
        pub fn new(table: ManagersTable) -> Self {
            Self { table }
        }

        pub async fn register(
            &self,
            stellar_address: &str,
            name: &str,
            email: &str,
            kyc_document_ref: &str,
        ) -> Result<models::ManagerRecord, sqlx::Error> {
            let id = Uuid::new_v4().to_string();
            let now = Utc::now();
            self.table.insert(&id, stellar_address, name, email, kyc_document_ref, now).await
        }

        pub async fn get(&self, id: &str) -> Result<Option<models::ManagerRecord>, sqlx::Error> {
            self.table.find_by_id(id).await
        }

        pub async fn find_by_stellar_address(
            &self,
            address: &str,
        ) -> Result<Option<models::ManagerRecord>, sqlx::Error> {
            self.table.find_by_stellar_address(address).await
        }

        pub async fn list(
            &self,
            status_filter: Option<&str>,
        ) -> Result<Vec<models::ManagerRecord>, sqlx::Error> {
            self.table.list(status_filter).await
        }

        pub async fn approve(
            &self,
            id: &str,
            notes: &str,
        ) -> Result<models::ManagerRecord, sqlx::Error> {
            self.table.update_status(id, "approved", notes, Utc::now()).await
        }

        pub async fn reject(
            &self,
            id: &str,
            notes: &str,
        ) -> Result<models::ManagerRecord, sqlx::Error> {
            self.table.update_status(id, "rejected", notes, Utc::now()).await
        }
    }
}

pub mod vault {
    use super::*;
    use crate::db::models;

    pub struct VaultRepo {
        table: VaultsTable,
    }

    impl VaultRepo {
        pub fn new(table: VaultsTable) -> Self {
            Self { table }
        }

        pub async fn create(
            &self,
            manager_id: &str,
            name: &str,
            status: &str,
            config_json: &str,
            idempotency_key: Option<&str>,
        ) -> Result<models::VaultRecord, sqlx::Error> {
            let id = Uuid::new_v4().to_string();
            let now = Utc::now();
            let key = idempotency_key.map(str::trim).filter(|s| !s.is_empty());
            if let Some(k) = key {
                if let Some(existing) = self.table.find_by_idempotency_key(manager_id, k).await? {
                    return Ok(existing);
                }
            }
            self.table.insert(&id, manager_id, name, status, config_json, key, now).await
        }

        pub async fn get(&self, id: &str) -> Result<Option<models::VaultRecord>, sqlx::Error> {
            self.table.find_by_id(id).await
        }

        pub async fn update(
            &self,
            id: &str,
            expected_version: i64,
            name: Option<&str>,
            status: Option<&str>,
            config_json: Option<&str>,
        ) -> Result<models::VaultRecord, sqlx::Error> {
            self.table.update(id, expected_version, name, status, config_json, Utc::now()).await
        }
    }
}

pub mod reconciliation {
    use super::*;
    use crate::db::models;

        #[derive(Clone)]
        pub struct ReconciliationRepo {
        report_table: ReconciliationReportsTable,
        disc_table: ReconciliationDiscrepanciesTable,
    }

    impl ReconciliationRepo {
        pub fn new(
            report_table: ReconciliationReportsTable,
            disc_table: ReconciliationDiscrepanciesTable,
        ) -> Self {
            Self {
                report_table,
                disc_table,
            }
        }

        pub async fn persist_report(
            &self,
            report: &models::ReconciliationReport,
            discrepancies: &[models::Discrepancy],
        ) -> Result<(), sqlx::Error> {
            let summary_json = report.summary.as_ref().map(|s| serde_json::to_value(s).unwrap_or_default());

            self.report_table.insert(
                &report.id,
                report.from_ledger,
                report.to_ledger,
                report.tolerance_pct,
                report.total_ledgers,
                report.discrepancies_count,
                report.avg_delta_pct,
                report.max_delta_pct,
                summary_json.as_ref(),
                &report.created_at,
            ).await?;

            self.disc_table.insert_for_report(&report.id, discrepancies).await?;

            Ok(())
        }

        pub async fn find_by_id(&self, id: &str) -> Result<Option<models::ReconciliationReport>, sqlx::Error> {
            self.report_table.find_by_id(id).await
        }

        pub async fn list(&self, limit: i64) -> Result<Vec<models::ReconciliationReport>, sqlx::Error> {
            self.report_table.list(limit).await
        }

        pub async fn find_discrepancies(
            &self,
            report_id: &str,
        ) -> Result<Vec<models::Discrepancy>, sqlx::Error> {
            self.disc_table.find_by_report_id(report_id).await
        }
    }
}