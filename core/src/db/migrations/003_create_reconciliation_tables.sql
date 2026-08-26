CREATE TABLE IF NOT EXISTS reconciliation_reports (
    id TEXT PRIMARY KEY,
    from_ledger INTEGER NOT NULL,
    to_ledger INTEGER NOT NULL,
    tolerance_pct REAL NOT NULL,
    total_ledgers INTEGER NOT NULL,
    discrepancies_count INTEGER NOT NULL,
    avg_delta_pct REAL NOT NULL,
    max_delta_pct REAL NOT NULL,
    summary TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS reconciliation_discrepancies (
    id TEXT PRIMARY KEY,
    report_id TEXT NOT NULL,
    ledger_sequence INTEGER NOT NULL,
    expected_fee INTEGER NOT NULL,
    actual_fee INTEGER NOT NULL,
    delta INTEGER NOT NULL,
    delta_pct REAL NOT NULL,
    severity TEXT NOT NULL,
    FOREIGN KEY (report_id) REFERENCES reconciliation_reports(id)
);

CREATE INDEX IF NOT EXISTS idx_discrepancies_report_id ON reconciliation_discrepancies(report_id);
CREATE INDEX IF NOT EXISTS idx_reports_created_at ON reconciliation_reports(created_at);