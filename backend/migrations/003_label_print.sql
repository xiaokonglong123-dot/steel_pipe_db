CREATE TABLE IF NOT EXISTS print_logs (
    id TEXT PRIMARY KEY,
    template_id TEXT NOT NULL,
    template_name TEXT NOT NULL DEFAULT '',
    pipe_numbers_json TEXT NOT NULL,
    total_labels INTEGER NOT NULL DEFAULT 0,
    printed_by TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_print_logs_created_at ON print_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_print_logs_template_id ON print_logs(template_id);
