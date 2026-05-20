CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    display_name TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'warehouse',
    email TEXT,
    phone TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS seamless_pipes (
    id TEXT PRIMARY KEY,
    pipe_number TEXT NOT NULL UNIQUE,
    grade TEXT NOT NULL,
    od REAL NOT NULL,
    wt REAL NOT NULL,
    length REAL NOT NULL,
    weight REAL NOT NULL,
    connection_type TEXT,
    heat_number TEXT,
    production_date TEXT,
    status TEXT NOT NULL DEFAULT 'in_stock',
    location TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS screen_pipes (
    id TEXT PRIMARY KEY,
    pipe_number TEXT NOT NULL UNIQUE,
    grade TEXT NOT NULL,
    od REAL NOT NULL,
    wt REAL NOT NULL,
    length REAL NOT NULL,
    weight REAL NOT NULL,
    screen_type TEXT NOT NULL,
    slot_width REAL,
    open_area REAL,
    connection_type TEXT,
    heat_number TEXT,
    production_date TEXT,
    status TEXT NOT NULL DEFAULT 'in_stock',
    location TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS inbound_records (
    id TEXT PRIMARY KEY,
    inbound_no TEXT NOT NULL UNIQUE,
    inbound_type TEXT NOT NULL,
    supplier_id TEXT,
    order_id TEXT,
    operator_id TEXT NOT NULL,
    total_items INTEGER NOT NULL DEFAULT 0,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS inbound_items (
    id TEXT PRIMARY KEY,
    inbound_id TEXT NOT NULL,
    pipe_type TEXT NOT NULL,
    pipe_id TEXT NOT NULL,
    confirmed INTEGER NOT NULL DEFAULT 0,
    notes TEXT
);

CREATE TABLE IF NOT EXISTS outbound_records (
    id TEXT PRIMARY KEY,
    outbound_no TEXT NOT NULL UNIQUE,
    outbound_type TEXT NOT NULL,
    customer_id TEXT,
    order_id TEXT,
    operator_id TEXT NOT NULL,
    total_items INTEGER NOT NULL DEFAULT 0,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS outbound_items (
    id TEXT PRIMARY KEY,
    outbound_id TEXT NOT NULL,
    pipe_type TEXT NOT NULL,
    pipe_id TEXT NOT NULL,
    confirmed INTEGER NOT NULL DEFAULT 0,
    notes TEXT
);

CREATE TABLE IF NOT EXISTS quality_certs (
    id TEXT PRIMARY KEY,
    cert_no TEXT NOT NULL,
    pipe_type TEXT NOT NULL,
    pipe_id TEXT NOT NULL,
    inspect_date TEXT NOT NULL,
    inspector TEXT NOT NULL,
    agency TEXT,
    result TEXT NOT NULL DEFAULT 'pending',
    items_json TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS pipe_attachments (
    id TEXT PRIMARY KEY,
    pipe_type TEXT NOT NULL,
    pipe_id TEXT NOT NULL,
    file_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_size INTEGER NOT NULL DEFAULT 0,
    mime_type TEXT NOT NULL DEFAULT 'application/octet-stream',
    uploaded_by TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS api_5ct_grade_ref (
    grade TEXT PRIMARY KEY,
    group_name TEXT NOT NULL,
    min_yield_strength REAL NOT NULL,
    max_yield_strength REAL NOT NULL,
    min_tensile_strength REAL NOT NULL,
    hardness_max REAL,
    description TEXT
);

CREATE TABLE IF NOT EXISTS suppliers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    contact_person TEXT,
    phone TEXT,
    email TEXT,
    address TEXT,
    cert_info TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS customers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    contact_person TEXT,
    phone TEXT,
    email TEXT,
    address TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS purchase_orders (
    id TEXT PRIMARY KEY,
    order_no TEXT NOT NULL UNIQUE,
    supplier_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft',
    total_amount REAL NOT NULL DEFAULT 0,
    notes TEXT,
    operator_id TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS purchase_order_items (
    id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL,
    pipe_type TEXT NOT NULL,
    grade TEXT NOT NULL,
    od REAL NOT NULL,
    wt REAL NOT NULL,
    quantity INTEGER NOT NULL,
    received_quantity INTEGER NOT NULL DEFAULT 0,
    unit_price REAL NOT NULL,
    subtotal REAL NOT NULL
);

CREATE TABLE IF NOT EXISTS sales_orders (
    id TEXT PRIMARY KEY,
    order_no TEXT NOT NULL UNIQUE,
    customer_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft',
    total_amount REAL NOT NULL DEFAULT 0,
    notes TEXT,
    operator_id TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS sales_order_items (
    id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL,
    pipe_type TEXT NOT NULL,
    grade TEXT NOT NULL,
    od REAL NOT NULL,
    wt REAL NOT NULL,
    quantity INTEGER NOT NULL,
    delivered_quantity INTEGER NOT NULL DEFAULT 0,
    unit_price REAL NOT NULL,
    subtotal REAL NOT NULL
);

CREATE TABLE IF NOT EXISTS contracts (
    id TEXT PRIMARY KEY,
    contract_no TEXT NOT NULL UNIQUE,
    contract_type TEXT NOT NULL,
    party_id TEXT NOT NULL,
    total_amount REAL NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'draft',
    sign_date TEXT,
    effective_date TEXT,
    expiry_date TEXT,
    notes TEXT,
    operator_id TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS contract_items (
    id TEXT PRIMARY KEY,
    contract_id TEXT NOT NULL,
    description TEXT NOT NULL,
    spec TEXT,
    quantity INTEGER NOT NULL,
    unit_price REAL NOT NULL,
    amount REAL NOT NULL,
    delivery_date TEXT
);

CREATE TABLE IF NOT EXISTS contract_payments (
    id TEXT PRIMARY KEY,
    contract_id TEXT NOT NULL,
    stage TEXT NOT NULL,
    amount REAL NOT NULL,
    due_date TEXT,
    paid INTEGER NOT NULL DEFAULT 0,
    notes TEXT
);

CREATE TABLE IF NOT EXISTS label_templates (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    width_mm REAL NOT NULL,
    height_mm REAL NOT NULL,
    config_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_seamless_pipes_pipe_number ON seamless_pipes(pipe_number);
CREATE INDEX IF NOT EXISTS idx_seamless_pipes_status ON seamless_pipes(status);
CREATE INDEX IF NOT EXISTS idx_seamless_pipes_grade ON seamless_pipes(grade);
CREATE INDEX IF NOT EXISTS idx_screen_pipes_pipe_number ON screen_pipes(pipe_number);
CREATE INDEX IF NOT EXISTS idx_screen_pipes_status ON screen_pipes(status);
CREATE INDEX IF NOT EXISTS idx_screen_pipes_grade ON screen_pipes(grade);
CREATE INDEX IF NOT EXISTS idx_inbound_records_created_at ON inbound_records(created_at);
CREATE INDEX IF NOT EXISTS idx_outbound_records_created_at ON outbound_records(created_at);
CREATE INDEX IF NOT EXISTS idx_inbound_items_pipe ON inbound_items(pipe_type, pipe_id);
CREATE INDEX IF NOT EXISTS idx_outbound_items_pipe ON outbound_items(pipe_type, pipe_id);
CREATE INDEX IF NOT EXISTS idx_quality_certs_pipe ON quality_certs(pipe_type, pipe_id);
CREATE INDEX IF NOT EXISTS idx_quality_certs_created_at ON quality_certs(created_at);
CREATE INDEX IF NOT EXISTS idx_purchase_orders_status ON purchase_orders(status);
CREATE INDEX IF NOT EXISTS idx_sales_orders_status ON sales_orders(status);
CREATE INDEX IF NOT EXISTS idx_contracts_status ON contracts(status);
CREATE INDEX IF NOT EXISTS idx_contracts_type ON contracts(contract_type);
