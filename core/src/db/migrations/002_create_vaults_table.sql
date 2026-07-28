CREATE TABLE IF NOT EXISTS vaults (
    id TEXT PRIMARY KEY,
    manager_id TEXT NOT NULL,
    name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    config_json TEXT NOT NULL DEFAULT '{}',
    version INTEGER NOT NULL DEFAULT 1,
    idempotency_key TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (manager_id) REFERENCES managers(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_vaults_idempotency ON vaults(manager_id, idempotency_key) WHERE idempotency_key IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_vaults_manager_id ON vaults(manager_id);
CREATE INDEX IF NOT EXISTS idx_vaults_status ON vaults(status);