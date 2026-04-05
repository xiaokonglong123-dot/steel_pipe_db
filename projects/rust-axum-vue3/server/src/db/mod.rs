use rusqlite::{params, Connection};
use std::sync::Mutex;

use crate::db::error::{DbError, Result};

#[derive(Clone)]
pub struct Database {
    conn: std::sync::Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        if let Some(parent) = std::path::Path::new(db_path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let conn = Connection::open(db_path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        let db = Database {
            conn: std::sync::Arc::new(Mutex::new(conn)),
        };
        db.init_tables()?;
        Ok(db)
    }

    pub fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap()
    }

    fn init_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS pipes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pipe_id TEXT UNIQUE NOT NULL,
                diameter REAL NOT NULL CHECK(diameter > 0),
                thickness REAL NOT NULL CHECK(thickness > 0),
                length REAL NOT NULL CHECK(length > 0),
                material TEXT NOT NULL,
                quantity INTEGER NOT NULL CHECK(quantity > 0),
                location TEXT,
                supplier TEXT,
                entry_date TEXT NOT NULL,
                last_update TEXT,
                status TEXT NOT NULL DEFAULT '在库'
            );
            CREATE TABLE IF NOT EXISTS inventory_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pipe_id TEXT NOT NULL,
                operation_type TEXT NOT NULL CHECK(operation_type IN ('入库', '出库')),
                quantity INTEGER NOT NULL CHECK(quantity > 0),
                operation_date TEXT NOT NULL,
                operator TEXT NOT NULL,
                remarks TEXT,
                FOREIGN KEY (pipe_id) REFERENCES pipes(pipe_id)
            );
            CREATE TABLE IF NOT EXISTS operation_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                operation_type TEXT NOT NULL,
                target_type TEXT NOT NULL,
                target_id TEXT NOT NULL,
                snapshot_before TEXT NOT NULL DEFAULT '',
                snapshot_after TEXT NOT NULL DEFAULT '',
                operator TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                remarks TEXT NOT NULL DEFAULT ''
            );
            CREATE INDEX IF NOT EXISTS idx_pipes_pipe_id ON pipes(pipe_id);
            CREATE INDEX IF NOT EXISTS idx_records_pipe_id ON inventory_records(pipe_id);
            CREATE INDEX IF NOT EXISTS idx_records_date ON inventory_records(operation_date);
            CREATE INDEX IF NOT EXISTS idx_logs_timestamp ON operation_logs(timestamp);",
        )?;
        Ok(())
    }
}

pub fn row_to_pipe(row: &rusqlite::Row) -> rusqlite::Result<crate::db::models::SteelPipe> {
    use crate::db::models::SteelPipe;
    Ok(SteelPipe {
        id: Some(row.get(0)?),
        pipe_id: row.get(1)?,
        diameter: row.get(2)?,
        thickness: row.get(3)?,
        length: row.get(4)?,
        material: row.get(5)?,
        quantity: row.get(6)?,
        location: row.get(7)?,
        supplier: row.get(8)?,
        entry_date: row.get(9)?,
        last_update: row.get(10)?,
        status: row.get(11)?,
    })
}

pub fn row_to_record(row: &rusqlite::Row) -> rusqlite::Result<crate::db::models::InventoryRecord> {
    use crate::db::models::InventoryRecord;
    Ok(InventoryRecord {
        id: Some(row.get(0)?),
        pipe_id: row.get(1)?,
        operation_type: row.get(2)?,
        quantity: row.get(3)?,
        operation_date: row.get(4)?,
        operator: row.get(5)?,
        remarks: row.get(6)?,
    })
}

pub fn escape_csv_field(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

pub fn upsert_pipe_tx(conn: &Connection, pipe: &crate::db::models::SteelPipe) -> Result<()> {
    use chrono::Local;
    use crate::db::models::SteelPipe;
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let tx = conn.unchecked_transaction()?;
    let existing = tx.query_row(
        "SELECT id FROM pipes WHERE pipe_id = ?",
        params![pipe.pipe_id],
        |row| row.get::<_, i64>(0),
    );
    match existing {
        Ok(_) => {
            tx.execute(
                "UPDATE pipes SET diameter=?, thickness=?, length=?, material=?, quantity=quantity+?, location=?, supplier=?, last_update=? WHERE pipe_id=?",
                params![pipe.diameter, pipe.thickness, pipe.length, pipe.material, pipe.quantity, pipe.location, pipe.supplier, now, pipe.pipe_id],
            )?;
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            tx.execute(
                "INSERT INTO pipes (pipe_id, diameter, thickness, length, material, quantity, location, supplier, entry_date, status) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
                params![pipe.pipe_id, pipe.diameter, pipe.thickness, pipe.length, pipe.material, pipe.quantity, pipe.location, pipe.supplier, now, "在库"],
            )?;
        }
        Err(e) => return Err(e.into()),
    }
    tx.commit()?;
    Ok(())
}

pub fn blocking_add_inventory_record(conn: &Connection, record: &crate::db::models::InventoryRecord) -> Result<()> {
    use chrono::Local;
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO inventory_records (pipe_id, operation_type, quantity, operation_date, operator, remarks) VALUES (?1,?2,?3,?4,?5,?6)",
        params![record.pipe_id, record.operation_type, record.quantity, now, record.operator, record.remarks],
    )?;
    Ok(())
}
