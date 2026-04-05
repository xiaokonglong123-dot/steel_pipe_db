use chrono::Local;
use rusqlite::{params, params_from_iter, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use thiserror::Error;
use calamine::{Reader, Xlsx, open_workbook};
use rust_xlsxwriter::Workbook;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Insufficient stock: current={current}, requested={requested}")]
    InsufficientStock { current: i32, requested: i32 },
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, DbError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteelPipe {
    pub id: Option<i64>,
    pub pipe_id: String,
    pub diameter: f64,
    pub thickness: f64,
    pub length: f64,
    pub material: String,
    pub quantity: i32,
    pub location: Option<String>,
    pub supplier: Option<String>,
    pub entry_date: String,
    pub last_update: Option<String>,
    pub status: String,
}

impl SteelPipe {
    pub fn validate(&self) -> Result<()> {
        if self.pipe_id.trim().is_empty() {
            return Err(DbError::Validation("钢管编号不能为空".to_string()));
        }
        if self.diameter <= 0.0 || self.diameter > 10000.0 {
            return Err(DbError::Validation("直径必须在0-10000mm之间".to_string()));
        }
        if self.thickness <= 0.0 || self.thickness > 500.0 {
            return Err(DbError::Validation("壁厚必须在0-500mm之间".to_string()));
        }
        if self.length <= 0.0 || self.length > 1000.0 {
            return Err(DbError::Validation("长度必须在0-1000m之间".to_string()));
        }
        if self.material.trim().is_empty() {
            return Err(DbError::Validation("材质不能为空".to_string()));
        }
        if self.quantity <= 0 {
            return Err(DbError::Validation("数量必须大于0".to_string()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryRecord {
    pub id: Option<i64>,
    pub pipe_id: String,
    pub operation_type: String,
    pub quantity: i32,
    pub operation_date: String,
    pub operator: String,
    pub remarks: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Statistics {
    pub total_types: i32,
    pub total_quantity: i32,
    pub total_in: i32,
    pub total_out: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialStats {
    pub material: String,
    pub type_count: i32,
    pub total_quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationLog {
    pub id: i64,
    pub operation_type: String,
    pub target_type: String,
    pub target_id: String,
    pub snapshot_before: String,
    pub snapshot_after: String,
    pub operator: String,
    pub timestamp: String,
    pub remarks: String,
}

#[derive(Clone)]
pub struct Database {
    conn: std::sync::Arc<Mutex<Connection>>,
}

fn row_to_pipe(row: &rusqlite::Row) -> rusqlite::Result<SteelPipe> {
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

fn row_to_record(row: &rusqlite::Row) -> rusqlite::Result<InventoryRecord> {
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

fn escape_csv_field(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
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

    fn upsert_pipe_tx(&self, pipe: &SteelPipe) -> Result<()> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let conn = self.conn.lock().unwrap();
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

    pub async fn add_pipe(&self, pipe: &SteelPipe) -> Result<()> {
        let pipe = pipe.clone();
        let db = self.clone();
        tokio::task::spawn_blocking(move || db.upsert_pipe_tx(&pipe)).await.unwrap()
    }

    pub fn blocking_add_pipe(&self, pipe: &SteelPipe) -> Result<()> {
        self.upsert_pipe_tx(pipe)
    }

    pub fn blocking_add_inventory_record(&self, record: &InventoryRecord) -> Result<()> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO inventory_records (pipe_id, operation_type, quantity, operation_date, operator, remarks) VALUES (?1,?2,?3,?4,?5,?6)",
            params![record.pipe_id, record.operation_type, record.quantity, now, record.operator, record.remarks],
        )?;
        Ok(())
    }

    pub async fn get_pipes(
        &self, page: i64, per_page: i64,
        search: Option<&str>, material: Option<&str>, status: Option<&str>,
        min_d: Option<f64>, max_d: Option<f64>, min_l: Option<f64>, max_l: Option<f64>,
    ) -> Result<(Vec<SteelPipe>, i64)> {
        let search = search.map(String::from);
        let material = material.map(String::from);
        let status = status.map(String::from);
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();

            let mut where_parts: Vec<String> = vec![];
            let mut str_args: Vec<String> = vec![];
            let mut f64_args: Vec<f64> = vec![];

            if let Some(s) = &search {
                where_parts.push("(pipe_id LIKE ? OR material LIKE ? OR location LIKE ? OR supplier LIKE ?)".to_string());
                let p = format!("%{}%", s);
                str_args.push(p.clone()); str_args.push(p.clone()); str_args.push(p.clone()); str_args.push(p);
            }
            if let Some(m) = &material {
                where_parts.push("material LIKE ?".to_string());
                str_args.push(format!("%{}%", m));
            }
            if let Some(s) = &status {
                where_parts.push("status = ?".to_string());
                str_args.push(s.to_string());
            }
            if let Some(v) = min_d { where_parts.push("diameter >= ?".to_string()); f64_args.push(v); }
            if let Some(v) = max_d { where_parts.push("diameter <= ?".to_string()); f64_args.push(v); }
            if let Some(v) = min_l { where_parts.push("length >= ?".to_string()); f64_args.push(v); }
            if let Some(v) = max_l { where_parts.push("length <= ?".to_string()); f64_args.push(v); }

            let where_sql = if where_parts.is_empty() {
                String::new()
            } else {
                format!(" WHERE {}", where_parts.join(" AND "))
            };

            let mut count_params: Vec<Box<dyn rusqlite::ToSql>> = vec![];
            for s in &str_args { count_params.push(Box::new(s.clone())); }
            for v in &f64_args { count_params.push(Box::new(*v)); }
            let count_sql = format!("SELECT COUNT(*) FROM pipes{}", where_sql);
            let count: i64 = conn.query_row(&count_sql, params_from_iter(count_params.iter().map(|p| p.as_ref())), |row| row.get(0))?;

            let offset = (page - 1) * per_page;
            let mut query_params: Vec<Box<dyn rusqlite::ToSql>> = vec![];
            for s in &str_args { query_params.push(Box::new(s.clone())); }
            for v in &f64_args { query_params.push(Box::new(*v)); }
            query_params.push(Box::new(per_page));
            query_params.push(Box::new(offset));

            let query_sql = format!(
                "SELECT id, pipe_id, diameter, thickness, length, material, quantity, location, supplier, entry_date, last_update, status FROM pipes{} ORDER BY entry_date DESC LIMIT ? OFFSET ?",
                where_sql
            );
            let mut stmt = conn.prepare(&query_sql)?;
            let pipes: Vec<SteelPipe> = stmt.query_map(
                params_from_iter(query_params.iter().map(|p| p.as_ref())),
                row_to_pipe,
            )?.map(|r| r.map_err(DbError::from)).collect::<Result<Vec<_>>>()?;

            Ok((pipes, count))
        }).await.unwrap()
    }

    pub async fn get_pipe_by_id(&self, pipe_id: &str) -> Result<Option<SteelPipe>> {
        let pipe_id = pipe_id.to_string();
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, pipe_id, diameter, thickness, length, material, quantity, location, supplier, entry_date, last_update, status FROM pipes WHERE pipe_id = ?",
            )?;
            let mut rows = stmt.query_map(params![pipe_id], row_to_pipe)?;
            if let Some(row) = rows.next() {
                return Ok(Some(row?));
            }
            Ok(None)
        }).await.unwrap()
    }

    pub async fn delete_pipe(&self, pipe_id: &str) -> Result<()> {
        let pipe_id = pipe_id.to_string();
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            let tx = conn.unchecked_transaction()?;
            tx.execute("DELETE FROM inventory_records WHERE pipe_id = ?", params![pipe_id])?;
            tx.execute("DELETE FROM pipes WHERE pipe_id = ?", params![pipe_id])?;
            tx.commit()?;
            Ok(())
        }).await.unwrap()
    }

    pub async fn batch_delete_pipes(&self, pipe_ids: &[String]) -> Result<()> {
        let pipe_ids = pipe_ids.to_vec();
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            let tx = conn.unchecked_transaction()?;
            for pipe_id in pipe_ids {
                tx.execute("DELETE FROM inventory_records WHERE pipe_id = ?", params![pipe_id]).ok();
                tx.execute("DELETE FROM pipes WHERE pipe_id = ?", params![pipe_id]).ok();
            }
            tx.commit()?;
            Ok(())
        }).await.unwrap()
    }

    pub async fn update_pipe(&self, pipe: &SteelPipe) -> Result<()> {
        let pipe = pipe.clone();
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "UPDATE pipes SET diameter=?, thickness=?, length=?, material=?, quantity=?, location=?, supplier=?, last_update=?, status=? WHERE pipe_id=?",
                params![pipe.diameter, pipe.thickness, pipe.length, pipe.material, pipe.quantity, pipe.location, pipe.supplier, now, pipe.status, pipe.pipe_id],
            )?;
            Ok(())
        }).await.unwrap()
    }

    pub async fn update_pipe_quantity(&self, pipe_id: &str, change: i32) -> Result<()> {
        let pipe_id = pipe_id.to_string();
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let conn = db.conn.lock().unwrap();
            let tx = conn.unchecked_transaction()?;
            let current: i32 = tx.query_row("SELECT quantity FROM pipes WHERE pipe_id = ?", params![pipe_id], |row| row.get(0))?;
            if current + change < 0 {
                return Err(DbError::InsufficientStock { current, requested: change });
            }
            tx.execute("UPDATE pipes SET quantity = quantity + ?, last_update = ? WHERE pipe_id = ?", params![change, now, pipe_id])?;
            tx.commit()?;
            Ok(())
        }).await.unwrap()
    }

    pub async fn add_inventory_record(&self, record: &InventoryRecord) -> Result<()> {
        let record = record.clone();
        let db = self.clone();
        tokio::task::spawn_blocking(move || db.blocking_add_inventory_record(&record)).await.unwrap()
    }

    pub async fn get_inventory_records(
        &self, pipe_id: Option<&str>, op_type: Option<&str>, start: Option<&str>, end: Option<&str>,
    ) -> Result<Vec<InventoryRecord>> {
        let pipe_id = pipe_id.map(String::from);
        let op_type = op_type.map(String::from);
        let start = start.map(String::from);
        let end = end.map(String::from);
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            let mut where_parts: Vec<String> = vec![];
            let mut args: Vec<Box<dyn rusqlite::ToSql>> = vec![];
            if let Some(v) = &pipe_id { where_parts.push("pipe_id = ?".to_string()); args.push(Box::new(v.clone())); }
            if let Some(v) = &op_type { where_parts.push("operation_type = ?".to_string()); args.push(Box::new(v.clone())); }
            if let Some(v) = &start { where_parts.push("operation_date >= ?".to_string()); args.push(Box::new(v.clone())); }
            if let Some(v) = &end { where_parts.push("operation_date <= ?".to_string()); args.push(Box::new(v.clone())); }
            let where_sql = if where_parts.is_empty() { String::new() } else { format!(" WHERE {}", where_parts.join(" AND ")) };
            let sql = format!("SELECT id, pipe_id, operation_type, quantity, operation_date, operator, remarks FROM inventory_records{} ORDER BY operation_date DESC", where_sql);
            let mut stmt = conn.prepare(&sql)?;
            let records: Vec<InventoryRecord> = stmt.query_map(
                params_from_iter(args.iter().map(|p| p.as_ref())),
                row_to_record,
            )?.map(|r| r.map_err(DbError::from)).collect::<Result<Vec<_>>>()?;
            Ok(records)
        }).await.unwrap()
    }

    pub async fn get_statistics(&self) -> Result<Statistics> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            let mut stats = Statistics::default();
            stats.total_types = conn.query_row("SELECT COUNT(*) FROM pipes", [], |row| row.get(0))?;
            stats.total_quantity = conn.query_row("SELECT COALESCE(SUM(quantity),0) FROM pipes", [], |row| row.get(0))?;
            stats.total_in = conn.query_row("SELECT COALESCE(SUM(quantity),0) FROM inventory_records WHERE operation_type='入库'", [], |row| row.get(0))?;
            stats.total_out = conn.query_row("SELECT COALESCE(SUM(quantity),0) FROM inventory_records WHERE operation_type='出库'", [], |row| row.get(0))?;
            Ok(stats)
        }).await.unwrap()
    }

    pub async fn get_material_stats(&self) -> Result<Vec<MaterialStats>> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT material, COUNT(*) as type_count, SUM(quantity) as total_quantity FROM pipes GROUP BY material ORDER BY total_quantity DESC",
            )?;
            let stats: Vec<MaterialStats> = stmt.query_map([], |row| {
                Ok(MaterialStats { material: row.get(0)?, type_count: row.get(1)?, total_quantity: row.get(2)? })
            })?.map(|r| r.map_err(DbError::from)).collect::<Result<Vec<_>>>()?;
            Ok(stats)
        }).await.unwrap()
    }

    pub async fn get_low_stock(&self, threshold: i32) -> Result<Vec<SteelPipe>> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, pipe_id, diameter, thickness, length, material, quantity, location, supplier, entry_date, last_update, status FROM pipes WHERE quantity <= ? ORDER BY quantity ASC",
            )?;
            let pipes: Vec<SteelPipe> = stmt.query_map(params![threshold], row_to_pipe)?.map(|r| r.map_err(DbError::from)).collect::<Result<Vec<_>>>()?;
            Ok(pipes)
        }).await.unwrap()
    }

    pub async fn get_daily_report(&self, start_date: &str, end_date: &str) -> Result<serde_json::Value> {
        let start = start_date.to_string();
        let end = end_date.to_string();
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            
            let entry_count: i32 = conn.query_row(
                "SELECT COALESCE(SUM(quantity),0) FROM inventory_records WHERE operation_type='入库' AND operation_date LIKE ?",
                params![format!("{}%", start)], |row| row.get(0)
            )?;
            
            let exit_count: i32 = conn.query_row(
                "SELECT COALESCE(SUM(quantity),0) FROM inventory_records WHERE operation_type='出库' AND operation_date LIKE ?",
                params![format!("{}%", end)], |row| row.get(0)
            )?;
            
            let total_in: i32 = conn.query_row(
                "SELECT COALESCE(SUM(quantity),0) FROM inventory_records WHERE operation_type='入库'", [], |row| row.get(0)
            )?;
            
            let total_out: i32 = conn.query_row(
                "SELECT COALESCE(SUM(quantity),0) FROM inventory_records WHERE operation_type='出库'", [], |row| row.get(0)
            )?;
            
            let current_stock: i32 = conn.query_row(
                "SELECT COALESCE(SUM(quantity),0) FROM pipes", [], |row| row.get(0)
            )?;
            
            Ok(serde_json::json!({
                "date": start,
                "entry_count": entry_count,
                "exit_count": exit_count,
                "total_in": total_in,
                "total_out": total_out,
                "current_stock": current_stock
            }))
        }).await.unwrap()
    }

    pub async fn get_monthly_report(&self, start_date: &str, end_date: &str) -> Result<serde_json::Value> {
        let start = start_date.to_string();
        let end = end_date.to_string();
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            
            let entry_count: i32 = conn.query_row(
                "SELECT COALESCE(SUM(quantity),0) FROM inventory_records WHERE operation_type='入库' AND operation_date >= ? AND operation_date <= ?",
                params![start, end], |row| row.get(0)
            )?;
            
            let exit_count: i32 = conn.query_row(
                "SELECT COALESCE(SUM(quantity),0) FROM inventory_records WHERE operation_type='出库' AND operation_date >= ? AND operation_date <= ?",
                params![start, end], |row| row.get(0)
            )?;
            
            let entry_records: Vec<(String, String, i32, String)> = {
                let mut stmt = conn.prepare(
                    "SELECT pipe_id, operation_date, quantity, operator FROM inventory_records WHERE operation_type='入库' AND operation_date >= ? AND operation_date <= ? ORDER BY operation_date DESC LIMIT 10"
                )?;
                let result = stmt.query_map(params![start, end], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
                })?.collect::<std::result::Result<Vec<_>, _>>()?;
                result
            };
            
            let exit_records: Vec<(String, String, i32, String)> = {
                let mut stmt = conn.prepare(
                    "SELECT pipe_id, operation_date, quantity, operator FROM inventory_records WHERE operation_type='出库' AND operation_date >= ? AND operation_date <= ? ORDER BY operation_date DESC LIMIT 10"
                )?;
                let result = stmt.query_map(params![start, end], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
                })?.collect::<std::result::Result<Vec<_>, _>>()?;
                result
            };
            
            Ok(serde_json::json!({
                "period": {"start": start, "end": end},
                "entry_count": entry_count,
                "exit_count": exit_count,
                "recent_entries": entry_records,
                "recent_exits": exit_records
            }))
        }).await.unwrap()
    }

    pub async fn get_recent_records(&self, limit: usize) -> Result<Vec<InventoryRecord>> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, pipe_id, operation_type, quantity, operation_date, operator, remarks FROM inventory_records ORDER BY operation_date DESC LIMIT ?",
            )?;
            let records: Vec<InventoryRecord> = stmt.query_map(params![limit as i64], row_to_record)?.map(|r| r.map_err(DbError::from)).collect::<Result<Vec<_>>>()?;
            Ok(records)
        }).await.unwrap()
    }

    pub async fn log_operation(&self, op_type: &str, target_type: &str, target_id: &str, before: &str, after: &str, operator: &str, remarks: &str) -> Result<()> {
        let op_type = op_type.to_string();
        let target_type = target_type.to_string();
        let target_id = target_id.to_string();
        let before = before.to_string();
        let after = after.to_string();
        let operator = operator.to_string();
        let remarks = remarks.to_string();
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO operation_logs (operation_type, target_type, target_id, snapshot_before, snapshot_after, operator, timestamp, remarks) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                params![op_type, target_type, target_id, before, after, operator, now, remarks],
            )?;
            Ok(())
        }).await.unwrap()
    }

    pub async fn get_operation_logs(&self, limit: usize) -> Result<Vec<OperationLog>> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, operation_type, target_type, target_id, snapshot_before, snapshot_after, operator, timestamp, remarks FROM operation_logs ORDER BY timestamp DESC LIMIT ?",
            )?;
            let logs: Vec<OperationLog> = stmt.query_map(params![limit as i64], |row| {
                Ok(OperationLog {
                    id: row.get(0)?, operation_type: row.get(1)?, target_type: row.get(2)?,
                    target_id: row.get(3)?, snapshot_before: row.get(4)?, snapshot_after: row.get(5)?,
                    operator: row.get(6)?, timestamp: row.get(7)?, remarks: row.get(8)?,
                })
            })?.map(|r| r.map_err(DbError::from)).collect::<Result<Vec<_>>>()?;
            Ok(logs)
        }).await.unwrap()
    }

    pub async fn import_pipes_from_csv(&self, csv_content: &str, operator: &str) -> Result<(usize, usize)> {
        let csv_content = csv_content.to_string();
        let operator = operator.to_string();
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let mut success = 0;
            let mut fail = 0;
            let conn = db.conn.lock().unwrap();
            let tx = conn.unchecked_transaction()?;

            for (i, line) in csv_content.lines().enumerate() {
                if i == 0 || line.trim().is_empty() { continue; }
                let fields: Vec<&str> = line.split(',').collect();
                if fields.len() < 6 { fail += 1; continue; }
                let pipe_id = fields[0].trim().to_string();
                let diameter: f64 = match fields[1].trim().parse() { Ok(v) => v, Err(_) => { fail += 1; continue; } };
                let thickness: f64 = match fields[2].trim().parse() { Ok(v) => v, Err(_) => { fail += 1; continue; } };
                let length: f64 = match fields[3].trim().parse() { Ok(v) => v, Err(_) => { fail += 1; continue; } };
                let material = fields[4].trim().to_string();
                let quantity: i32 = match fields[5].trim().parse() { Ok(v) => v, Err(_) => { fail += 1; continue; } };
                let location = if fields.len() > 6 && !fields[6].trim().is_empty() { Some(fields[6].trim().to_string()) } else { None };
                let supplier = if fields.len() > 7 && !fields[7].trim().is_empty() { Some(fields[7].trim().to_string()) } else { None };
                let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

                let existing = tx.query_row("SELECT id FROM pipes WHERE pipe_id = ?", params![pipe_id], |row| row.get::<_, i64>(0));
                match existing {
                    Ok(_) => {
                        tx.execute(
                            "UPDATE pipes SET diameter=?, thickness=?, length=?, material=?, quantity=quantity+?, location=?, supplier=?, last_update=? WHERE pipe_id=?",
                            params![diameter, thickness, length, material, quantity, location, supplier, now, pipe_id],
                        ).ok();
                    }
                    Err(rusqlite::Error::QueryReturnedNoRows) => {
                        tx.execute(
                            "INSERT INTO pipes (pipe_id, diameter, thickness, length, material, quantity, location, supplier, entry_date, status) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
                            params![pipe_id, diameter, thickness, length, material, quantity, location, supplier, now, "在库"],
                        ).ok();
                    }
                    Err(_) => { fail += 1; continue; }
                }

                tx.execute(
                    "INSERT INTO inventory_records (pipe_id, operation_type, quantity, operation_date, operator, remarks) VALUES (?1,?2,?3,?4,?5,?6)",
                    params![pipe_id, "入库", quantity, now, operator, "批量导入"],
                ).ok();
                success += 1;
            }

            tx.commit()?;
            Ok((success, fail))
        }).await.unwrap()
    }

    pub async fn export_inventory_csv(&self) -> Result<String> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT pipe_id, diameter, thickness, length, material, quantity, location, supplier, entry_date, status FROM pipes ORDER BY entry_date DESC",
            )?;
            let pipes: Vec<(String, f64, f64, f64, String, i32, Option<String>, Option<String>, String, String)> =
                stmt.query_map([], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?, row.get(8)?, row.get(9)?))
                })?.collect::<std::result::Result<Vec<_>, _>>()?;

            let mut csv = String::from("钢管编号,直径(mm),壁厚(mm),长度(m),材质,数量,存放位置,供应商,入库日期,状态\n");
            for (pipe_id, diameter, thickness, length, material, quantity, location, supplier, entry_date, status) in pipes {
                csv.push_str(&format!("{},{:.2},{:.2},{:.2},{},{},{},{},{},{}\n",
                    escape_csv_field(&pipe_id), diameter, thickness, length,
                    escape_csv_field(&material), quantity,
                    escape_csv_field(location.as_deref().unwrap_or("")),
                    escape_csv_field(supplier.as_deref().unwrap_or("")),
                    escape_csv_field(&entry_date), escape_csv_field(&status)));
            }
            Ok(csv)
        }).await.unwrap()
    }

    pub async fn export_pipes_by_filter(
        &self,
        search: Option<&str>,
        material: Option<&str>,
        status: Option<&str>,
        min_d: Option<f64>,
        max_d: Option<f64>,
        min_l: Option<f64>,
        max_l: Option<f64>,
    ) -> Result<String> {
        let search = search.map(String::from);
        let material = material.map(String::from);
        let status = status.map(String::from);
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            let mut where_parts: Vec<String> = vec![];
            let mut args: Vec<Box<dyn rusqlite::ToSql>> = vec![];

            if let Some(s) = &search {
                where_parts.push("(pipe_id LIKE ? OR material LIKE ? OR location LIKE ? OR supplier LIKE ?)".to_string());
                let p = format!("%{}%", s);
                args.push(Box::new(p.clone())); args.push(Box::new(p.clone()));
                args.push(Box::new(p.clone())); args.push(Box::new(p));
            }
            if let Some(m) = &material {
                where_parts.push("material LIKE ?".to_string());
                args.push(Box::new(format!("%{}%", m)));
            }
            if let Some(s) = &status {
                where_parts.push("status = ?".to_string());
                args.push(Box::new(s.clone()));
            }
            if let Some(v) = min_d { where_parts.push("diameter >= ?".to_string()); args.push(Box::new(v)); }
            if let Some(v) = max_d { where_parts.push("diameter <= ?".to_string()); args.push(Box::new(v)); }
            if let Some(v) = min_l { where_parts.push("length >= ?".to_string()); args.push(Box::new(v)); }
            if let Some(v) = max_l { where_parts.push("length <= ?".to_string()); args.push(Box::new(v)); }

            let where_sql = if where_parts.is_empty() {
                String::new()
            } else {
                format!(" WHERE {}", where_parts.join(" AND "))
            };

            let sql = format!(
                "SELECT pipe_id, diameter, thickness, length, material, quantity, location, supplier, entry_date, status FROM pipes{} ORDER BY entry_date DESC",
                where_sql
            );
            let mut stmt = conn.prepare(&sql)?;
            let pipes: Vec<(String, f64, f64, f64, String, i32, Option<String>, Option<String>, String, String)> =
                stmt.query_map(params_from_iter(args.iter().map(|p| p.as_ref())), |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?, row.get(8)?, row.get(9)?))
                })?.collect::<std::result::Result<Vec<_>, _>>()?;

            let mut csv = String::from("钢管编号,直径(mm),壁厚(mm),长度(m),材质,数量,存放位置,供应商,入库日期,状态\n");
            for (pipe_id, diameter, thickness, length, material, quantity, location, supplier, entry_date, status) in pipes {
                csv.push_str(&format!("{},{:.2},{:.2},{:.2},{},{},{},{},{},{}\n",
                    escape_csv_field(&pipe_id), diameter, thickness, length,
                    escape_csv_field(&material), quantity,
                    escape_csv_field(location.as_deref().unwrap_or("")),
                    escape_csv_field(supplier.as_deref().unwrap_or("")),
                    escape_csv_field(&entry_date), escape_csv_field(&status)));
            }
            Ok(csv)
        }).await.unwrap()
    }

    pub async fn export_records_csv(&self, pipe_id: Option<&str>, op_type: Option<&str>, start: Option<&str>, end: Option<&str>) -> Result<String> {
        let pipe_id = pipe_id.map(String::from);
        let op_type = op_type.map(String::from);
        let start = start.map(String::from);
        let end = end.map(String::from);
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let mut where_parts: Vec<String> = vec![];
            let mut args: Vec<Box<dyn rusqlite::ToSql>> = vec![];
            if let Some(v) = &pipe_id { where_parts.push("pipe_id = ?".to_string()); args.push(Box::new(v.clone())); }
            if let Some(v) = &op_type { where_parts.push("operation_type = ?".to_string()); args.push(Box::new(v.clone())); }
            if let Some(v) = &start { where_parts.push("operation_date >= ?".to_string()); args.push(Box::new(v.clone())); }
            if let Some(v) = &end { where_parts.push("operation_date <= ?".to_string()); args.push(Box::new(v.clone())); }
            let where_sql = if where_parts.is_empty() { String::new() } else { format!(" WHERE {}", where_parts.join(" AND ")) };

            let conn = db.conn.lock().unwrap();
            let sql = format!("SELECT pipe_id, operation_type, quantity, operation_date, operator, remarks FROM inventory_records{} ORDER BY operation_date DESC", where_sql);
            let mut stmt = conn.prepare(&sql)?;
            let records: Vec<(String, String, i32, String, String, Option<String>)> =
                stmt.query_map(params_from_iter(args.iter().map(|p| p.as_ref())), |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?))
                })?.collect::<std::result::Result<Vec<_>, _>>()?;

            let mut csv = String::from("钢管编号,操作类型,数量,操作日期,操作员,备注\n");
            for (pipe_id, op_type, quantity, op_date, operator, remarks) in records {
                csv.push_str(&format!("{},{},{},{},{},{}\n",
                    escape_csv_field(&pipe_id), escape_csv_field(&op_type), quantity,
                    escape_csv_field(&op_date), escape_csv_field(&operator),
                    escape_csv_field(remarks.as_deref().unwrap_or(""))));
            }
            Ok(csv)
        }).await.unwrap()
    }

    pub async fn import_pipes_from_excel(&self, excel_path: &str, operator: &str) -> Result<(usize, usize)> {
        let excel_path = excel_path.to_string();
        let operator = operator.to_string();
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let mut success = 0;
            let mut fail = 0;

            let mut workbook: Xlsx<std::io::BufReader<std::fs::File>> = open_workbook(&excel_path).map_err(|e: calamine::XlsxError| DbError::Validation(e.to_string()))?;
            let sheet = workbook.worksheet_range_at(0);
            if let Some(Ok(range)) = sheet {
                let conn = db.conn.lock().unwrap();
                let tx = conn.unchecked_transaction()?;

                for (i, row) in range.rows().enumerate() {
                    if i == 0 || row.is_empty() { continue; }
                    let fields: Vec<String> = row.iter().map(|c| c.to_string()).collect();
                    if fields.len() < 6 { fail += 1; continue; }
                    let pipe_id = fields[0].trim().to_string();
                    if pipe_id.is_empty() { fail += 1; continue; }
                    let diameter: f64 = match fields[1].trim().parse() { Ok(v) => v, Err(_) => { fail += 1; continue; } };
                    let thickness: f64 = match fields[2].trim().parse() { Ok(v) => v, Err(_) => { fail += 1; continue; } };
                    let length: f64 = match fields[3].trim().parse() { Ok(v) => v, Err(_) => { fail += 1; continue; } };
                    let material = fields[4].trim().to_string();
                    let quantity: i32 = match fields[5].trim().parse() { Ok(v) => v, Err(_) => { fail += 1; continue; } };
                    let location = if fields.len() > 6 && !fields[6].trim().is_empty() { Some(fields[6].trim().to_string()) } else { None };
                    let supplier = if fields.len() > 7 && !fields[7].trim().is_empty() { Some(fields[7].trim().to_string()) } else { None };
                    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

                    let existing = tx.query_row("SELECT id FROM pipes WHERE pipe_id = ?", params![pipe_id], |row| row.get::<_, i64>(0));
                    match existing {
                        Ok(_) => {
                            tx.execute(
                                "UPDATE pipes SET diameter=?, thickness=?, length=?, material=?, quantity=quantity+?, location=?, supplier=?, last_update=? WHERE pipe_id=?",
                                params![diameter, thickness, length, material, quantity, location, supplier, now, pipe_id],
                            ).ok();
                        }
                        Err(rusqlite::Error::QueryReturnedNoRows) => {
                            tx.execute(
                                "INSERT INTO pipes (pipe_id, diameter, thickness, length, material, quantity, location, supplier, entry_date, status) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
                                params![pipe_id, diameter, thickness, length, material, quantity, location, supplier, now, "在库"],
                            ).ok();
                        }
                        Err(_) => { fail += 1; continue; }
                    }

                    tx.execute(
                        "INSERT INTO inventory_records (pipe_id, operation_type, quantity, operation_date, operator, remarks) VALUES (?1,?2,?3,?4,?5,?6)",
                        params![pipe_id, "入库", quantity, now, operator, "批量导入"],
                    ).ok();
                    success += 1;
                }

                tx.commit()?;
            }

            std::fs::remove_file(&excel_path).ok();
            Ok((success, fail))
        }).await.unwrap()
    }

    pub async fn export_inventory_to_excel(&self, path: &str) -> Result<()> {
        let path = path.to_string();
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT pipe_id, diameter, thickness, length, material, quantity, location, supplier, entry_date, status FROM pipes ORDER BY entry_date DESC",
            )?;
            let pipes: Vec<(String, f64, f64, f64, String, i32, Option<String>, Option<String>, String, String)> =
                stmt.query_map([], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?, row.get(8)?, row.get(9)?))
                })?.collect::<std::result::Result<Vec<_>, _>>()?;

            let mut workbook = Workbook::new();
            let worksheet = workbook.add_worksheet();
            let headers = ["钢管编号", "直径(mm)", "壁厚(mm)", "长度(m)", "材质", "数量", "存放位置", "供应商", "入库日期", "状态"];
            for (i, h) in headers.iter().enumerate() {
                worksheet.write_string(0, i as u16, *h).ok();
            }
            for (i, (pipe_id, diameter, thickness, length, material, quantity, location, supplier, entry_date, status)) in pipes.iter().enumerate() {
                let row = i as u32 + 1;
                worksheet.write_string(row, 0, pipe_id).ok();
                worksheet.write_number(row, 1, *diameter).ok();
                worksheet.write_number(row, 2, *thickness).ok();
                worksheet.write_number(row, 3, *length).ok();
                worksheet.write_string(row, 4, material).ok();
                worksheet.write_number(row, 5, *quantity as f64).ok();
                worksheet.write_string(row, 6, location.as_deref().unwrap_or("")).ok();
                worksheet.write_string(row, 7, supplier.as_deref().unwrap_or("")).ok();
                worksheet.write_string(row, 8, entry_date).ok();
                worksheet.write_string(row, 9, status).ok();
            }
            workbook.save(&path).map_err(|e| DbError::Validation(e.to_string()))?;
            Ok(())
        }).await.unwrap()
    }

    pub async fn export_records_to_excel(&self, path: &str, pipe_id: Option<&str>, op_type: Option<&str>, start: Option<&str>, end: Option<&str>) -> Result<()> {
        let path = path.to_string();
        let pipe_id = pipe_id.map(String::from);
        let op_type = op_type.map(String::from);
        let start = start.map(String::from);
        let end = end.map(String::from);
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let mut where_parts: Vec<String> = vec![];
            let mut args: Vec<Box<dyn rusqlite::ToSql>> = vec![];
            if let Some(v) = &pipe_id { where_parts.push("pipe_id = ?".to_string()); args.push(Box::new(v.clone())); }
            if let Some(v) = &op_type { where_parts.push("operation_type = ?".to_string()); args.push(Box::new(v.clone())); }
            if let Some(v) = &start { where_parts.push("operation_date >= ?".to_string()); args.push(Box::new(v.clone())); }
            if let Some(v) = &end { where_parts.push("operation_date <= ?".to_string()); args.push(Box::new(v.clone())); }
            let where_sql = if where_parts.is_empty() { String::new() } else { format!(" WHERE {}", where_parts.join(" AND ")) };

            let conn = db.conn.lock().unwrap();
            let sql = format!("SELECT pipe_id, operation_type, quantity, operation_date, operator, remarks FROM inventory_records{} ORDER BY operation_date DESC", where_sql);
            let mut stmt = conn.prepare(&sql)?;
            let records: Vec<(String, String, i32, String, String, Option<String>)> =
                stmt.query_map(params_from_iter(args.iter().map(|p| p.as_ref())), |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?))
                })?.collect::<std::result::Result<Vec<_>, _>>()?;

            let mut workbook = Workbook::new();
            let worksheet = workbook.add_worksheet();
            let headers = ["钢管编号", "操作类型", "数量", "操作日期", "操作员", "备注"];
            for (i, h) in headers.iter().enumerate() {
                worksheet.write_string(0, i as u16, *h).ok();
            }
            for (i, (pipe_id, op_type, quantity, op_date, operator, remarks)) in records.iter().enumerate() {
                let row = i as u32 + 1;
                worksheet.write_string(row, 0, pipe_id).ok();
                worksheet.write_string(row, 1, op_type).ok();
                worksheet.write_number(row, 2, *quantity as f64).ok();
                worksheet.write_string(row, 3, op_date).ok();
                worksheet.write_string(row, 4, operator).ok();
                worksheet.write_string(row, 5, remarks.as_deref().unwrap_or("")).ok();
            }
            workbook.save(&path).map_err(|e| DbError::Validation(e.to_string()))?;
            Ok(())
        }).await.unwrap()
    }
}
