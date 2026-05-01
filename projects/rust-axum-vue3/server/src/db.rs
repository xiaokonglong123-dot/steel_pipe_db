use chrono::Local;
use rusqlite::{params, params_from_iter, Connection};
use std::sync::{Arc, Mutex};
use calamine::{Reader, Xlsx, open_workbook};
use rust_xlsxwriter::Workbook;
use crate::models::*;
use crate::error::{AppError, Result};

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
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
        furnace_number: row.get(12)?,
        heat_treatment_batch: row.get(13)?,
        sample_number: row.get(14)?,
        production_count: row.get(15)?,
        material_rack: row.get(16)?,
        remarks: row.get(17)?,
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
            conn: Arc::new(Mutex::new(conn)),
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
                status TEXT NOT NULL DEFAULT '在库',
                furnace_number TEXT,
                heat_treatment_batch TEXT,
                sample_number TEXT,
                production_count INTEGER,
                material_rack TEXT,
                remarks TEXT
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
            CREATE TABLE IF NOT EXISTS productions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                furnace_number TEXT NOT NULL,
                heat_treatment_batch TEXT,
                material_batch TEXT,
                production_count INTEGER NOT NULL,
                sample TEXT,
                supplier TEXT,
                operator TEXT NOT NULL,
                production_date TEXT NOT NULL,
                remarks TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_pipes_pipe_id ON pipes(pipe_id);
            CREATE INDEX IF NOT EXISTS idx_records_pipe_id ON inventory_records(pipe_id);
            CREATE INDEX IF NOT EXISTS idx_records_date ON inventory_records(operation_date);
            CREATE INDEX IF NOT EXISTS idx_logs_timestamp ON operation_logs(timestamp);

            CREATE TABLE IF NOT EXISTS heat_treatment_orders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                order_number TEXT UNIQUE NOT NULL,
                pipe_id TEXT NOT NULL,
                furnace_number TEXT NOT NULL,
                heat_treatment_type TEXT NOT NULL,
                process_parameters TEXT,
                start_time TEXT NOT NULL,
                end_time TEXT,
                operator TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT '进行中',
                temperature_curve TEXT,
                cooling_method TEXT,
                remarks TEXT,
                FOREIGN KEY (pipe_id) REFERENCES pipes(pipe_id)
            );

            CREATE TABLE IF NOT EXISTS heat_treatment_processes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                order_id INTEGER NOT NULL,
                stage TEXT NOT NULL,
                target_temperature REAL NOT NULL,
                actual_temperature REAL,
                heating_rate REAL,
                holding_time INTEGER,
                cooling_rate REAL,
                start_time TEXT NOT NULL,
                end_time TEXT,
                operator TEXT NOT NULL,
                remarks TEXT,
                FOREIGN KEY (order_id) REFERENCES heat_treatment_orders(id)
            );

            CREATE TABLE IF NOT EXISTS quality_inspections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                order_id INTEGER NOT NULL,
                inspection_type TEXT NOT NULL,
                hardness_hb REAL,
                hardness_hrc REAL,
                tensile_strength REAL,
                yield_strength REAL,
                elongation REAL,
                metallographic_structure TEXT,
                inspector TEXT NOT NULL,
                inspection_date TEXT NOT NULL,
                result TEXT NOT NULL,
                remarks TEXT,
                FOREIGN KEY (order_id) REFERENCES heat_treatment_orders(id)
            );

            CREATE TABLE IF NOT EXISTS sampling_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                order_id INTEGER NOT NULL,
                sample_number TEXT NOT NULL,
                sampling_position TEXT NOT NULL,
                sampling_time TEXT NOT NULL,
                sampler TEXT NOT NULL,
                sample_description TEXT,
                sample_status TEXT DEFAULT '待检测',
                remarks TEXT,
                FOREIGN KEY (order_id) REFERENCES heat_treatment_orders(id)
            );

            CREATE TABLE IF NOT EXISTS marking_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                order_id INTEGER NOT NULL,
                marking_number TEXT NOT NULL,
                marking_content TEXT NOT NULL,
                marking_position TEXT NOT NULL,
                marking_time TEXT NOT NULL,
                marker TEXT NOT NULL,
                marking_method TEXT,
                marking_status TEXT DEFAULT '待确认',
                remarks TEXT,
                FOREIGN KEY (order_id) REFERENCES heat_treatment_orders(id)
            );

            CREATE TABLE IF NOT EXISTS furnace_status (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                furnace_number TEXT UNIQUE NOT NULL,
                status TEXT NOT NULL DEFAULT '空闲',
                current_temperature REAL,
                target_temperature REAL,
                load_count INTEGER,
                last_maintenance TEXT,
                operator TEXT NOT NULL,
                update_time TEXT NOT NULL,
                remarks TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_ht_orders_pipe_id ON heat_treatment_orders(pipe_id);
            CREATE INDEX IF NOT EXISTS idx_ht_orders_status ON heat_treatment_orders(status);
            CREATE INDEX IF NOT EXISTS idx_ht_processes_order_id ON heat_treatment_processes(order_id);
            CREATE INDEX IF NOT EXISTS idx_quality_inspections_order_id ON quality_inspections(order_id);
            CREATE INDEX IF NOT EXISTS idx_sampling_order_id ON sampling_records(order_id);
            CREATE INDEX IF NOT EXISTS idx_marking_order_id ON marking_records(order_id);",
        )?;
        Ok(())
    }

    pub async fn add_pipe(&self, pipe: &SteelPipe) -> Result<()> {
        let pipe = pipe.clone();
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let conn = db.conn.lock().unwrap();
            let tx = conn.unchecked_transaction()?;
            let existing = tx.query_row(
                "SELECT id FROM pipes WHERE pipe_id = ?",
                params![pipe.pipe_id],
                |row| row.get::<_, i64>(0),
            );
            match existing {
                Ok(_) => {
                    tx.execute(
                        "UPDATE pipes SET diameter=?, thickness=?, length=?, material=?, quantity=quantity+?, location=?, supplier=?, last_update=?, furnace_number=?, heat_treatment_batch=?, sample_number=?, production_count=?, material_rack=?, remarks=? WHERE pipe_id=?",
                        params![pipe.diameter, pipe.thickness, pipe.length, pipe.material, pipe.quantity, pipe.location, pipe.supplier, now, pipe.furnace_number, pipe.heat_treatment_batch, pipe.sample_number, pipe.production_count, pipe.material_rack, pipe.remarks, pipe.pipe_id],
                    )?;
                }
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    tx.execute(
                        "INSERT INTO pipes (pipe_id, diameter, thickness, length, material, quantity, location, supplier, entry_date, status, furnace_number, heat_treatment_batch, sample_number, production_count, material_rack, remarks) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
                        params![pipe.pipe_id, pipe.diameter, pipe.thickness, pipe.length, pipe.material, pipe.quantity, pipe.location, pipe.supplier, now, "在库", pipe.furnace_number, pipe.heat_treatment_batch, pipe.sample_number, pipe.production_count, pipe.material_rack, pipe.remarks],
                    )?;
                }
                Err(e) => return Err(e.into()),
            }
            tx.commit()?;
            Ok(())
        }).await?
    }

    pub async fn get_pipes(
        &self, page: i64, per_page: i64,
        search: Option<String>, material: Option<String>, status: Option<String>,
        min_d: Option<f64>, max_d: Option<f64>, min_l: Option<f64>, max_l: Option<f64>,
    ) -> Result<(Vec<SteelPipe>, i64)> {
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
                "SELECT id, pipe_id, diameter, thickness, length, material, quantity, location, supplier, entry_date, last_update, status, furnace_number, heat_treatment_batch, sample_number, production_count, material_rack, remarks FROM pipes{} ORDER BY entry_date DESC LIMIT ? OFFSET ?",
                where_sql
            );
            let mut stmt = conn.prepare(&query_sql)?;
            let pipes: Vec<SteelPipe> = stmt.query_map(
                params_from_iter(query_params.iter().map(|p| p.as_ref())),
                row_to_pipe,
            )?.map(|r| r.map_err(AppError::from)).collect::<std::result::Result<Vec<_>, AppError>>()?;

            Ok((pipes, count))
        }).await?
    }

    pub async fn get_pipe_by_id(&self, pipe_id: String) -> Result<Option<SteelPipe>> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, pipe_id, diameter, thickness, length, material, quantity, location, supplier, entry_date, last_update, status, furnace_number, heat_treatment_batch, sample_number, production_count, material_rack, remarks FROM pipes WHERE pipe_id = ?",
            )?;
            let mut rows = stmt.query_map(params![pipe_id], row_to_pipe)?;
            if let Some(row) = rows.next() {
                return Ok(Some(row?));
            }
            Ok(None)
        }).await?
    }

    pub async fn delete_pipe(&self, pipe_id: String) -> Result<()> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            let tx = conn.unchecked_transaction()?;
            tx.execute("DELETE FROM inventory_records WHERE pipe_id = ?", params![pipe_id])?;
            tx.execute("DELETE FROM pipes WHERE pipe_id = ?", params![pipe_id])?;
            tx.commit()?;
            Ok(())
        }).await?
    }

    pub async fn batch_delete_pipes(&self, pipe_ids: Vec<String>) -> Result<()> {
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
        }).await?
    }

    pub async fn update_pipe(&self, pipe: SteelPipe) -> Result<()> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "UPDATE pipes SET diameter=?, thickness=?, length=?, material=?, quantity=?, location=?, supplier=?, last_update=?, status=?, furnace_number=?, heat_treatment_batch=?, sample_number=?, production_count=?, material_rack=?, remarks=? WHERE pipe_id=?",
                params![pipe.diameter, pipe.thickness, pipe.length, pipe.material, pipe.quantity, pipe.location, pipe.supplier, now, pipe.status, pipe.furnace_number, pipe.heat_treatment_batch, pipe.sample_number, pipe.production_count, pipe.material_rack, pipe.remarks, pipe.pipe_id],
            )?;
            Ok(())
        }).await?
    }

    pub async fn add_inventory_record(&self, record: InventoryRecord) -> Result<()> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO inventory_records (pipe_id, operation_type, quantity, operation_date, operator, remarks) VALUES (?1,?2,?3,?4,?5,?6)",
                params![record.pipe_id, record.operation_type, record.quantity, now, record.operator, record.remarks],
            )?;
            Ok(())
        }).await?
    }

    pub async fn process_entry(&self, pipe: SteelPipe, operator: String, remarks: Option<String>) -> Result<()> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let mut conn = db.conn.lock().unwrap();
            let tx = conn.transaction()?;
            
            // 1. Upsert Pipe
            let existing = tx.query_row(
                "SELECT id FROM pipes WHERE pipe_id = ?",
                params![pipe.pipe_id],
                |row| row.get::<_, i64>(0),
            );
            
            match existing {
                Ok(_) => {
                    tx.execute(
                        "UPDATE pipes SET diameter=?, thickness=?, length=?, material=?, quantity=quantity+?, location=?, supplier=?, last_update=?, furnace_number=?, heat_treatment_batch=?, sample_number=?, production_count=?, material_rack=?, remarks=? WHERE pipe_id=?",
                        params![pipe.diameter, pipe.thickness, pipe.length, pipe.material, pipe.quantity, pipe.location, pipe.supplier, now, pipe.furnace_number, pipe.heat_treatment_batch, pipe.sample_number, pipe.production_count, pipe.material_rack, pipe.remarks, pipe.pipe_id],
                    )?;
                }
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    tx.execute(
                        "INSERT INTO pipes (pipe_id, diameter, thickness, length, material, quantity, location, supplier, entry_date, status, furnace_number, heat_treatment_batch, sample_number, production_count, material_rack, remarks) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
                        params![pipe.pipe_id, pipe.diameter, pipe.thickness, pipe.length, pipe.material, pipe.quantity, pipe.location, pipe.supplier, now, "在库", pipe.furnace_number, pipe.heat_treatment_batch, pipe.sample_number, pipe.production_count, pipe.material_rack, pipe.remarks],
                    )?;
                }
                Err(e) => return Err(e.into()),
            }
            
            // 2. Add Inventory Record
            tx.execute(
                "INSERT INTO inventory_records (pipe_id, operation_type, quantity, operation_date, operator, remarks) VALUES (?1,?2,?3,?4,?5,?6)",
                params![pipe.pipe_id, "入库", pipe.quantity, now, operator, remarks],
            )?;
            
            tx.commit()?;
            Ok(())
        }).await?
    }

    pub async fn process_exit(&self, pipe_id: String, quantity: i32, operator: String, remarks: Option<String>) -> Result<(i32, i32)> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let mut conn = db.conn.lock().unwrap();
            let tx = conn.transaction()?;
            
            // 1. Check and Update Quantity
            let before: i32 = tx.query_row("SELECT quantity FROM pipes WHERE pipe_id = ?", params![pipe_id], |row| row.get(0))?;
            if before < quantity {
                return Err(AppError::InsufficientStock { current: before, requested: quantity });
            }
            
            let after = before - quantity;
            tx.execute("UPDATE pipes SET quantity = ?, last_update = ? WHERE pipe_id = ?", params![after, now, pipe_id])?;
            
            // 2. Add Inventory Record
            tx.execute(
                "INSERT INTO inventory_records (pipe_id, operation_type, quantity, operation_date, operator, remarks) VALUES (?1,?2,?3,?4,?5,?6)",
                params![pipe_id, "出库", quantity, now, operator, remarks],
            )?;
            
            tx.commit()?;
            Ok((before, after))
        }).await?
    }

    pub async fn get_inventory_records(
        &self, pipe_id: Option<String>, op_type: Option<String>, start: Option<String>, end: Option<String>,
    ) -> Result<Vec<InventoryRecord>> {
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
            )?.map(|r| r.map_err(AppError::from)).collect::<std::result::Result<Vec<_>, AppError>>()?;
            Ok(records)
        }).await?
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
        }).await?
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
            })?.map(|r| r.map_err(AppError::from)).collect::<std::result::Result<Vec<_>, AppError>>()?;
            Ok(stats)
        }).await?
    }

    pub async fn get_low_stock(&self, threshold: i32) -> Result<Vec<SteelPipe>> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, pipe_id, diameter, thickness, length, material, quantity, location, supplier, entry_date, last_update, status, furnace_number, heat_treatment_batch, sample_number, production_count, material_rack, remarks FROM pipes WHERE quantity <= ? ORDER BY quantity ASC",
            )?;
            let pipes: Vec<SteelPipe> = stmt.query_map(params![threshold], row_to_pipe)?.map(|r| r.map_err(AppError::from)).collect::<std::result::Result<Vec<_>, AppError>>()?;
            Ok(pipes)
        }).await?
    }

    pub async fn get_daily_report(&self, start_date: String, end_date: String) -> Result<serde_json::Value> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            
            let entry_count: i32 = conn.query_row(
                "SELECT COALESCE(SUM(quantity),0) FROM inventory_records WHERE operation_type='入库' AND operation_date LIKE ?",
                params![format!("{}%", start_date)], |row| row.get(0)
            )?;
            
            let exit_count: i32 = conn.query_row(
                "SELECT COALESCE(SUM(quantity),0) FROM inventory_records WHERE operation_type='出库' AND operation_date LIKE ?",
                params![format!("{}%", end_date)], |row| row.get(0)
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
                "date": start_date,
                "entry_count": entry_count,
                "exit_count": exit_count,
                "total_in": total_in,
                "total_out": total_out,
                "current_stock": current_stock
            }))
        }).await?
    }

    pub async fn get_monthly_report(&self, start_date: String, end_date: String) -> Result<serde_json::Value> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            
            let entry_count: i32 = conn.query_row(
                "SELECT COALESCE(SUM(quantity),0) FROM inventory_records WHERE operation_type='入库' AND operation_date >= ? AND operation_date <= ?",
                params![start_date, end_date], |row| row.get(0)
            )?;
            
            let exit_count: i32 = conn.query_row(
                "SELECT COALESCE(SUM(quantity),0) FROM inventory_records WHERE operation_type='出库' AND operation_date >= ? AND operation_date <= ?",
                params![start_date, end_date], |row| row.get(0)
            )?;
            
            let entry_records: Vec<(String, String, i32, String)> = {
                let mut stmt = conn.prepare(
                    "SELECT pipe_id, operation_date, quantity, operator FROM inventory_records WHERE operation_type='入库' AND operation_date >= ? AND operation_date <= ? ORDER BY operation_date DESC LIMIT 10"
                )?;
                let result = stmt.query_map(params![start_date, end_date], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
                })?.collect::<std::result::Result<Vec<_>, _>>()?;
                result
            };
            
            let exit_records: Vec<(String, String, i32, String)> = {
                let mut stmt = conn.prepare(
                    "SELECT pipe_id, operation_date, quantity, operator FROM inventory_records WHERE operation_type='出库' AND operation_date >= ? AND operation_date <= ? ORDER BY operation_date DESC LIMIT 10"
                )?;
                let result = stmt.query_map(params![start_date, end_date], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
                })?.collect::<std::result::Result<Vec<_>, _>>()?;
                result
            };
            
            Ok(serde_json::json!({
                "period": {"start": start_date, "end": end_date},
                "entry_count": entry_count,
                "exit_count": exit_count,
                "recent_entries": entry_records,
                "recent_exits": exit_records
            }))
        }).await?
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
        }).await?
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
            })?.map(|r| r.map_err(AppError::from)).collect::<std::result::Result<Vec<_>, AppError>>()?;
            Ok(logs)
        }).await?
    }

    pub async fn import_pipes_from_csv(&self, csv_content: String, operator: String) -> Result<(usize, usize)> {
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
                            "UPDATE pipes SET diameter=?, thickness=?, length=?, material=?, quantity=quantity+?, location=?, supplier=?, last_update=?, furnace_number=?, heat_treatment_batch=?, sample_number=?, production_count=?, material_rack=?, remarks=? WHERE pipe_id=?",
                            params![diameter, thickness, length, material, quantity, location, supplier, now, None as Option<String>, None as Option<String>, None as Option<String>, None as Option<i32>, None as Option<String>, None as Option<String>, pipe_id],
                        ).ok();
                    }
                    Err(rusqlite::Error::QueryReturnedNoRows) => {
                        tx.execute(
                            "INSERT INTO pipes (pipe_id, diameter, thickness, length, material, quantity, location, supplier, entry_date, status, furnace_number, heat_treatment_batch, sample_number, production_count, material_rack, remarks) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
                            params![pipe_id, diameter, thickness, length, material, quantity, location, supplier, now, "在库", None as Option<String>, None as Option<String>, None as Option<String>, None as Option<i32>, None as Option<String>, None as Option<String>],
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
        }).await?
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
        }).await?
    }

    pub async fn export_pipes_by_filter(
        &self,
        search: Option<String>,
        material: Option<String>,
        status: Option<String>,
        min_d: Option<f64>,
        max_d: Option<f64>,
        min_l: Option<f64>,
        max_l: Option<f64>,
    ) -> Result<String> {
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
            "SELECT id, pipe_id, diameter, thickness, length, material, quantity, location, supplier, entry_date, last_update, status, furnace_number, heat_treatment_batch, sample_number, production_count, material_rack, remarks FROM pipes{} ORDER BY entry_date DESC",
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
        }).await?
    }

    pub async fn export_records_csv(&self, pipe_id: Option<String>, op_type: Option<String>, start: Option<String>, end: Option<String>) -> Result<String> {
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
        }).await?
    }

    pub async fn import_pipes_from_excel(&self, excel_path: String, operator: String) -> Result<(usize, usize)> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let mut success = 0;
            let mut fail = 0;

            let mut workbook: Xlsx<std::io::BufReader<std::fs::File>> = open_workbook(&excel_path).map_err(|e: calamine::XlsxError| AppError::Validation(e.to_string()))?;
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
                                "UPDATE pipes SET diameter=?, thickness=?, length=?, material=?, quantity=quantity+?, location=?, supplier=?, last_update=?, furnace_number=?, heat_treatment_batch=?, sample_number=?, production_count=?, material_rack=?, remarks=? WHERE pipe_id=?",
                                params![diameter, thickness, length, material, quantity, location, supplier, now, None as Option<String>, None as Option<String>, None as Option<String>, None as Option<i32>, None as Option<String>, None as Option<String>, pipe_id],
                            ).ok();
                        }
                        Err(rusqlite::Error::QueryReturnedNoRows) => {
                            tx.execute(
                                "INSERT INTO pipes (pipe_id, diameter, thickness, length, material, quantity, location, supplier, entry_date, status, furnace_number, heat_treatment_batch, sample_number, production_count, material_rack, remarks) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
                                params![pipe_id, diameter, thickness, length, material, quantity, location, supplier, now, "在库", None as Option<String>, None as Option<String>, None as Option<String>, None as Option<i32>, None as Option<String>, None as Option<String>],
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
        }).await?
    }

    pub async fn export_inventory_to_excel(&self, path: String) -> Result<()> {
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
            workbook.save(&path).map_err(|e| AppError::Validation(e.to_string()))?;
            Ok(())
        }).await?
    }

    pub async fn export_records_to_excel(&self, path: String, pipe_id: Option<String>, op_type: Option<String>, start: Option<String>, end: Option<String>) -> Result<()> {
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
            workbook.save(&path).map_err(|e| AppError::Validation(e.to_string()))?;
            Ok(())
        }).await?
    }

    pub async fn add_production(&self, production: Production) -> Result<i64> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO productions (furnace_number, heat_treatment_batch, material_batch, production_count, sample, supplier, operator, production_date, remarks) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                params![production.furnace_number, production.heat_treatment_batch, production.material_batch, production.production_count, production.sample, production.supplier, production.operator, now, production.remarks],
            )?;
            Ok(conn.last_insert_rowid())
        }).await?
    }

    pub async fn get_productions(&self) -> Result<Vec<Production>> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, furnace_number, heat_treatment_batch, material_batch, production_count, sample, supplier, operator, production_date, remarks FROM productions ORDER BY production_date DESC",
            )?;
            let productions: Vec<Production> = stmt.query_map([], |row| {
                Ok(Production {
                    id: Some(row.get(0)?),
                    furnace_number: row.get(1)?,
                    heat_treatment_batch: row.get(2)?,
                    material_batch: row.get(3)?,
                    production_count: row.get(4)?,
                    sample: row.get(5)?,
                    supplier: row.get(6)?,
                    operator: row.get(7)?,
                    production_date: row.get(8)?,
                    remarks: row.get(9)?,
                })
            })?.map(|r| r.map_err(AppError::from)).collect::<std::result::Result<Vec<_>, AppError>>()?;
            Ok(productions)
        }).await?
    }

    pub async fn get_data_dictionaries(&self) -> Result<DataDictionaries> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            
            let mut stmt = conn.prepare("SELECT DISTINCT material FROM pipes WHERE material IS NOT NULL AND material != ''")?;
            let materials = stmt.query_map([], |row| row.get(0))?.collect::<rusqlite::Result<Vec<String>>>()?;
            
            let mut stmt = conn.prepare("SELECT DISTINCT location FROM pipes WHERE location IS NOT NULL AND location != ''")?;
            let locations = stmt.query_map([], |row| row.get(0))?.collect::<rusqlite::Result<Vec<String>>>()?;
            
            let mut stmt = conn.prepare("SELECT DISTINCT status FROM pipes WHERE status IS NOT NULL AND status != ''")?;
            let statuses = stmt.query_map([], |row| row.get(0))?.collect::<rusqlite::Result<Vec<String>>>()?;
            
            Ok(DataDictionaries { materials, locations, statuses })
        }).await?
    }

    pub async fn get_inventory_trends(&self) -> Result<Vec<InventoryTrend>> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            let mut trends = Vec::new();
            
            for i in (0..7).rev() {
                let date = Local::now() - chrono::Duration::days(i);
                let date_str = date.format("%Y-%m-%d").to_string();
                
                let entry_count: i32 = conn.query_row(
                    "SELECT COALESCE(SUM(quantity), 0) FROM inventory_records WHERE operation_type = '入库' AND operation_date LIKE ?",
                    params![format!("{}%", date_str)],
                    |row| row.get(0),
                )?;
                
                let exit_count: i32 = conn.query_row(
                    "SELECT COALESCE(SUM(quantity), 0) FROM inventory_records WHERE operation_type = '出库' AND operation_date LIKE ?",
                    params![format!("{}%", date_str)],
                    |row| row.get(0),
                )?;
                
                trends.push(InventoryTrend {
                    date: date_str,
                    entry_count,
                    exit_count,
                });
            }
            
            Ok(trends)
        }).await?
    }

    pub async fn create_heat_treatment_order(&self, order: HeatTreatmentOrder) -> Result<i64> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO heat_treatment_orders (order_number, pipe_id, furnace_number, heat_treatment_type, process_parameters, start_time, operator, status, temperature_curve, cooling_method, remarks) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                params![order.order_number, order.pipe_id, order.furnace_number, order.heat_treatment_type, order.process_parameters, order.start_time, order.operator, order.status, order.temperature_curve, order.cooling_method, order.remarks],
            )?;
            Ok(conn.last_insert_rowid())
        }).await?
    }

    pub async fn get_heat_treatment_orders(&self, status: Option<String>, pipe_id: Option<String>) -> Result<Vec<HeatTreatmentOrder>> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            let mut where_parts = vec![];
            let mut args: Vec<Box<dyn rusqlite::ToSql>> = vec![];
            
            if let Some(s) = &status {
                where_parts.push("status = ?".to_string());
                args.push(Box::new(s.clone()));
            }
            if let Some(p) = &pipe_id {
                where_parts.push("pipe_id = ?".to_string());
                args.push(Box::new(p.clone()));
            }
            
            let where_sql = if where_parts.is_empty() {
                String::new()
            } else {
                format!(" WHERE {}", where_parts.join(" AND "))
            };
            
            let sql = format!("SELECT id, order_number, pipe_id, furnace_number, heat_treatment_type, process_parameters, start_time, end_time, operator, status, temperature_curve, cooling_method, remarks FROM heat_treatment_orders{} ORDER BY start_time DESC", where_sql);
            
            let mut stmt = conn.prepare(&sql)?;
            let orders: Vec<HeatTreatmentOrder> = stmt.query_map(
                params_from_iter(args.iter().map(|p| p.as_ref())),
                |row| {
                    Ok(HeatTreatmentOrder {
                        id: Some(row.get(0)?),
                        order_number: row.get(1)?,
                        pipe_id: row.get(2)?,
                        furnace_number: row.get(3)?,
                        heat_treatment_type: row.get(4)?,
                        process_parameters: row.get(5)?,
                        start_time: row.get(6)?,
                        end_time: row.get(7)?,
                        operator: row.get(8)?,
                        status: row.get(9)?,
                        temperature_curve: row.get(10)?,
                        cooling_method: row.get(11)?,
                        remarks: row.get(12)?,
                    })
                },
            )?.map(|r| r.map_err(AppError::from)).collect::<std::result::Result<Vec<_>, AppError>>()?;
            Ok(orders)
        }).await?
    }

    pub async fn update_heat_treatment_order_status(&self, order_id: i64, status: String, end_time: Option<String>) -> Result<()> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            if let Some(et) = &end_time {
                conn.execute(
                    "UPDATE heat_treatment_orders SET status = ?, end_time = ? WHERE id = ?",
                    params![status, et, order_id],
                )?;
            } else {
                conn.execute(
                    "UPDATE heat_treatment_orders SET status = ? WHERE id = ?",
                    params![status, order_id],
                )?;
            }
            Ok(())
        }).await?
    }

    pub async fn add_heat_treatment_process(&self, process: HeatTreatmentProcess) -> Result<i64> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO heat_treatment_processes (order_id, stage, target_temperature, actual_temperature, heating_rate, holding_time, cooling_rate, start_time, end_time, operator, remarks) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                params![process.order_id, process.stage, process.target_temperature, process.actual_temperature, process.heating_rate, process.holding_time, process.cooling_rate, process.start_time, process.end_time, process.operator, process.remarks],
            )?;
            Ok(conn.last_insert_rowid())
        }).await?
    }

    pub async fn add_quality_inspection(&self, inspection: QualityInspection) -> Result<i64> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO quality_inspections (order_id, inspection_type, hardness_hb, hardness_hrc, tensile_strength, yield_strength, elongation, metallographic_structure, inspector, inspection_date, result, remarks) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
                params![inspection.order_id, inspection.inspection_type, inspection.hardness_hb, inspection.hardness_hrc, inspection.tensile_strength, inspection.yield_strength, inspection.elongation, inspection.metallographic_structure, inspection.inspector, inspection.inspection_date, inspection.result, inspection.remarks],
            )?;
            Ok(conn.last_insert_rowid())
        }).await?
    }

    pub async fn get_quality_inspections(&self, order_id: i64) -> Result<Vec<QualityInspection>> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, order_id, inspection_type, hardness_hb, hardness_hrc, tensile_strength, yield_strength, elongation, metallographic_structure, inspector, inspection_date, result, remarks FROM quality_inspections WHERE order_id = ? ORDER BY inspection_date DESC"
            )?;
            let inspections: Vec<QualityInspection> = stmt.query_map(params![order_id], |row| {
                Ok(QualityInspection {
                    id: Some(row.get(0)?),
                    order_id: row.get(1)?,
                    inspection_type: row.get(2)?,
                    hardness_hb: row.get(3)?,
                    hardness_hrc: row.get(4)?,
                    tensile_strength: row.get(5)?,
                    yield_strength: row.get(6)?,
                    elongation: row.get(7)?,
                    metallographic_structure: row.get(8)?,
                    inspector: row.get(9)?,
                    inspection_date: row.get(10)?,
                    result: row.get(11)?,
                    remarks: row.get(12)?,
                })
            })?.map(|r| r.map_err(AppError::from)).collect::<std::result::Result<Vec<_>, AppError>>()?;
            Ok(inspections)
        }).await?
    }

    pub async fn add_sampling_record(&self, record: SamplingRecord) -> Result<i64> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO sampling_records (order_id, sample_number, sampling_position, sampling_time, sampler, sample_description, sample_status, remarks) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                params![record.order_id, record.sample_number, record.sampling_position, now, record.sampler, record.sample_description, "待检测".to_string(), record.remarks],
            )?;
            Ok(conn.last_insert_rowid())
        }).await?
    }

    pub async fn get_sampling_records(&self, order_id: i64) -> Result<Vec<SamplingRecord>> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, order_id, sample_number, sampling_position, sampling_time, sampler, sample_description, sample_status, remarks FROM sampling_records WHERE order_id = ? ORDER BY sampling_time DESC"
            )?;
            let records: Vec<SamplingRecord> = stmt.query_map(params![order_id], |row| {
                Ok(SamplingRecord {
                    id: Some(row.get(0)?),
                    order_id: row.get(1)?,
                    sample_number: row.get(2)?,
                    sampling_position: row.get(3)?,
                    sampling_time: row.get(4)?,
                    sampler: row.get(5)?,
                    sample_description: row.get(6)?,
                    sample_status: row.get(7)?,
                    remarks: row.get(8)?,
                })
            })?.map(|r| r.map_err(AppError::from)).collect::<std::result::Result<Vec<_>, AppError>>()?;
            Ok(records)
        }).await?
    }

    pub async fn update_sampling_status(&self, id: i64, status: String) -> Result<()> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            conn.execute("UPDATE sampling_records SET sample_status = ? WHERE id = ?", params![status, id])?;
            Ok(())
        }).await?
    }

    pub async fn add_marking_record(&self, record: MarkingRecord) -> Result<i64> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO marking_records (order_id, marking_number, marking_content, marking_position, marking_time, marker, marking_method, marking_status, remarks) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                params![record.order_id, record.marking_number, record.marking_content, record.marking_position, now, record.marker, record.marking_method, "待确认".to_string(), record.remarks],
            )?;
            Ok(conn.last_insert_rowid())
        }).await?
    }

    pub async fn get_marking_records(&self, order_id: i64) -> Result<Vec<MarkingRecord>> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, order_id, marking_number, marking_content, marking_position, marking_time, marker, marking_method, marking_status, remarks FROM marking_records WHERE order_id = ? ORDER BY marking_time DESC"
            )?;
            let records: Vec<MarkingRecord> = stmt.query_map(params![order_id], |row| {
                Ok(MarkingRecord {
                    id: Some(row.get(0)?),
                    order_id: row.get(1)?,
                    marking_number: row.get(2)?,
                    marking_content: row.get(3)?,
                    marking_position: row.get(4)?,
                    marking_time: row.get(5)?,
                    marker: row.get(6)?,
                    marking_method: row.get(7)?,
                    marking_status: row.get(8)?,
                    remarks: row.get(9)?,
                })
            })?.map(|r| r.map_err(AppError::from)).collect::<std::result::Result<Vec<_>, AppError>>()?;
            Ok(records)
        }).await?
    }

    pub async fn update_marking_status(&self, id: i64, status: String) -> Result<()> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            conn.execute("UPDATE marking_records SET marking_status = ? WHERE id = ?", params![status, id])?;
            Ok(())
        }).await?
    }

    pub async fn update_furnace_status(&self, furnace: FurnaceStatus) -> Result<()> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            let existing = conn.query_row(
                "SELECT id FROM furnace_status WHERE furnace_number = ?",
                params![furnace.furnace_number],
                |row| row.get::<_, i64>(0),
            );
            
            match existing {
                Ok(_) => {
                    conn.execute(
                        "UPDATE furnace_status SET status = ?, current_temperature = ?, target_temperature = ?, load_count = ?, last_maintenance = ?, operator = ?, update_time = ?, remarks = ? WHERE furnace_number = ?",
                        params![furnace.status, furnace.current_temperature, furnace.target_temperature, furnace.load_count, furnace.last_maintenance, furnace.operator, furnace.update_time, furnace.remarks, furnace.furnace_number],
                    )?;
                }
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    conn.execute(
                        "INSERT INTO furnace_status (furnace_number, status, current_temperature, target_temperature, load_count, last_maintenance, operator, update_time, remarks) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                        params![furnace.furnace_number, furnace.status, furnace.current_temperature, furnace.target_temperature, furnace.load_count, furnace.last_maintenance, furnace.operator, furnace.update_time, furnace.remarks],
                    )?;
                }
                Err(e) => return Err(e.into()),
            }
            Ok(())
        }).await?
    }

    pub async fn get_furnace_statuses(&self) -> Result<Vec<FurnaceStatus>> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, furnace_number, status, current_temperature, target_temperature, load_count, last_maintenance, operator, update_time, remarks FROM furnace_status ORDER BY furnace_number"
            )?;
            let statuses: Vec<FurnaceStatus> = stmt.query_map([], |row| {
                Ok(FurnaceStatus {
                    id: Some(row.get(0)?),
                    furnace_number: row.get(1)?,
                    status: row.get(2)?,
                    current_temperature: row.get(3)?,
                    target_temperature: row.get(4)?,
                    load_count: row.get(5)?,
                    last_maintenance: row.get(6)?,
                    operator: row.get(7)?,
                    update_time: row.get(8)?,
                    remarks: row.get(9)?,
                })
            })?.map(|r| r.map_err(AppError::from)).collect::<std::result::Result<Vec<_>, AppError>>()?;
            Ok(statuses)
        }).await?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    async fn setup_db() -> Database {
        let tmp_file = NamedTempFile::new().unwrap();
        let path = tmp_file.path().to_str().unwrap().to_string();
        Database::new(&path).unwrap()
    }

    #[tokio::test]
    async fn test_db_init() {
        let _db = setup_db().await;
    }

    #[tokio::test]
    async fn test_add_get_pipe() {
        let db = setup_db().await;
        let pipe = SteelPipe {
            id: None,
            pipe_id: "TEST001".to_string(),
            diameter: 10.0,
            thickness: 2.0,
            length: 5.0,
            material: "Steel".to_string(),
            quantity: 100,
            location: Some("A1".to_string()),
            supplier: Some("Supp1".to_string()),
            entry_date: "".to_string(),
            last_update: None,
            status: "在库".to_string(),
            furnace_number: None,
            heat_treatment_batch: None,
            sample_number: None,
            production_count: None,
            material_rack: None,
            remarks: None,
        };

        db.add_pipe(&pipe).await.unwrap();
        let result = db.get_pipe_by_id("TEST001".to_string()).await.unwrap();
        assert!(result.is_some());
        let found = result.unwrap();
        assert_eq!(found.pipe_id, "TEST001");
        assert_eq!(found.quantity, 100);
    }

    #[tokio::test]
    async fn test_update_quantity() {
        let db = setup_db().await;
        let pipe = SteelPipe {
            id: None,
            pipe_id: "TEST002".to_string(),
            diameter: 10.0,
            thickness: 2.0,
            length: 5.0,
            material: "Steel".to_string(),
            quantity: 50,
            location: None,
            supplier: None,
            entry_date: "".to_string(),
            last_update: None,
            status: "在库".to_string(),
            furnace_number: None,
            heat_treatment_batch: None,
            sample_number: None,
            production_count: None,
            material_rack: None,
            remarks: None,
        };

        db.add_pipe(&pipe).await.unwrap();
        let (before, after) = db.process_exit("TEST002".to_string(), 10, "admin".to_string(), None).await.unwrap();
        
        assert_eq!(before, 50);
        assert_eq!(after, 40);
        
        let found = db.get_pipe_by_id("TEST002".to_string()).await.unwrap().unwrap();
        assert_eq!(found.quantity, 40);

        // Test insufficient stock
        let err = db.process_exit("TEST002".to_string(), 50, "admin".to_string(), None).await;
        assert!(matches!(err, Err(AppError::InsufficientStock { .. })));
    }

    #[tokio::test]
    async fn test_data_dictionaries() {
        let db = setup_db().await;
        let p1 = SteelPipe {
            pipe_id: "P1".to_string(), material: "M1".to_string(), location: Some("L1".to_string()), status: "在库".to_string(),
            diameter: 1.0, thickness: 1.0, length: 1.0, quantity: 1, id: None, entry_date: "".to_string(), last_update: None,
            supplier: None, furnace_number: None, heat_treatment_batch: None, sample_number: None, production_count: None, material_rack: None, remarks: None,
        };
        let p2 = SteelPipe {
            pipe_id: "P2".to_string(), material: "M2".to_string(), location: Some("L2".to_string()), status: "已出库".to_string(),
            diameter: 1.0, thickness: 1.0, length: 1.0, quantity: 1, id: None, entry_date: "".to_string(), last_update: None,
            supplier: None, furnace_number: None, heat_treatment_batch: None, sample_number: None, production_count: None, material_rack: None, remarks: None,
        };
        db.add_pipe(&p1).await.unwrap();
        // 直接更新状态
        let mut p2_with_status = p2.clone();
        db.add_pipe(&p2).await.unwrap();
        p2_with_status.status = "已出库".to_string();
        db.update_pipe(p2_with_status).await.unwrap();

        let dicts = db.get_data_dictionaries().await.unwrap();
        assert!(dicts.materials.contains(&"M1".to_string()));
        assert!(dicts.materials.contains(&"M2".to_string()));
        assert!(dicts.locations.contains(&"L1".to_string()));
        assert!(dicts.statuses.contains(&"在库".to_string()));
        assert!(dicts.statuses.contains(&"已出库".to_string()));
    }

    #[tokio::test]
    async fn test_inventory_trends() {
        let db = setup_db().await;
        let p1 = SteelPipe {
            pipe_id: "TREND01".to_string(), diameter: 1.0, thickness: 1.0, length: 1.0, material: "M1".to_string(), quantity: 10,
            id: None, entry_date: "".to_string(), last_update: None, status: "在库".to_string(), location: None, supplier: None,
            furnace_number: None, heat_treatment_batch: None, sample_number: None, production_count: None, material_rack: None, remarks: None,
        };
        
        // 使用事务级方法增加记录
        db.process_entry(p1, "admin".to_string(), None).await.unwrap();
        db.process_exit("TREND01".to_string(), 3, "admin".to_string(), None).await.unwrap();

        let trends = db.get_inventory_trends().await.unwrap();
        assert_eq!(trends.len(), 7);
        let today_trend = trends.last().unwrap();
        assert_eq!(today_trend.entry_count, 10);
        assert_eq!(today_trend.exit_count, 3);
    }
}
