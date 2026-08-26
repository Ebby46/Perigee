ALTER TABLE vaults ADD COLUMN idempotency_key TEXT;
CREATE UNIQUE INDEX IF NOT EXISTS idx_vaults_idempotency
  ON vaults(manager_id, idempotency_key)
  WHERE idempotency_key IS NOT NULL;
