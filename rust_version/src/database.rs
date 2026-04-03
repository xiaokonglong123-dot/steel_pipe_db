use calamine::Reader;
use chrono::Local;
use rusqlite::{params, Connection};
use rust_xlsxwriter::{Color, Format, FormatBorder, Workbook, XlsxError};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// 自定义错误类型
#[derive(Error, Debug)]
pub enum DbError {
    #[error("数据库错误: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("验证错误: {0}")]
    Validation(String),

    #[allow(dead_code)]
    #[error("数据未找到: {0}")]
    NotFound(String),

    #[error("库存不足: 当前 {current}, 请求 {requested}")]
    InsufficientStock { current: i32, requested: i32 },

    #[error("IO错误: {0}")]
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
    /// 验证钢管数据的有效性
    pub fn validate(&self) -> Result<()> {
        if self.pipe_id.trim().is_empty() {
            return Err(DbError::Validation("钢管编号不能为空".to_string()));
        }
        if self.diameter <= 0.0 {
            return Err(DbError::Validation("直径必须大于0".to_string()));
        }
        if self.thickness <= 0.0 {
            return Err(DbError::Validation("壁厚必须大于0".to_string()));
        }
        if self.length <= 0.0 {
            return Err(DbError::Validation("长度必须大于0".to_string()));
        }
        if self.material.trim().is_empty() {
            return Err(DbError::Validation("材质不能为空".to_string()));
        }
        if self.quantity <= 0 {
            return Err(DbError::Validation("数量必须大于0".to_string()));
        }
        // 验证材质是否在允许的范围内
        let valid_materials = ["碳钢", "不锈钢", "合金钢", "无缝钢管", "焊接钢管"];
        if !valid_materials.contains(&self.material.as_str()) {
            return Err(DbError::Validation(format!(
                "无效的材质类型: {}",
                self.material
            )));
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

impl InventoryRecord {
    /// 验证出入库记录的有效性
    pub fn validate(&self) -> Result<()> {
        if self.pipe_id.trim().is_empty() {
            return Err(DbError::Validation("钢管编号不能为空".to_string()));
        }
        if self.operation_type != "入库" && self.operation_type != "出库" {
            return Err(DbError::Validation(
                "操作类型必须是'入库'或'出库'".to_string(),
            ));
        }
        if self.quantity <= 0 {
            return Err(DbError::Validation("数量必须大于0".to_string()));
        }
        if self.operator.trim().is_empty() {
            return Err(DbError::Validation("操作员不能为空".to_string()));
        }
        Ok(())
    }
}

/// 数据库结构体，使用Arc<Mutex<>>实现线程安全
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

unsafe impl Send for Database {}
unsafe impl Sync for Database {}

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        // 确保数据库目录存在
        if let Some(parent) = Path::new(db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(db_path)?;

        // 设置SQLite性能优化选项
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "cache_size", -10000i64)?;

        let db = Database {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        // 创建钢管表
        conn.execute(
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
            )",
            [],
        )?;

        // 创建出入库记录表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS inventory_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pipe_id TEXT NOT NULL,
                operation_type TEXT NOT NULL CHECK(operation_type IN ('入库', '出库')),
                quantity INTEGER NOT NULL CHECK(quantity > 0),
                operation_date TEXT NOT NULL,
                operator TEXT NOT NULL,
                remarks TEXT,
                FOREIGN KEY (pipe_id) REFERENCES pipes(pipe_id)
            )",
            [],
        )?;

        // 创建索引以提高查询性能
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pipes_pipe_id ON pipes(pipe_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_records_pipe_id ON inventory_records(pipe_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_records_date ON inventory_records(operation_date)",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS operation_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                operation_type TEXT NOT NULL,
                target_type TEXT NOT NULL,
                target_id TEXT NOT NULL,
                snapshot_before TEXT NOT NULL DEFAULT '',
                snapshot_after TEXT NOT NULL DEFAULT '',
                operator TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                remarks TEXT NOT NULL DEFAULT ''
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_logs_timestamp ON operation_logs(timestamp)",
            [],
        )?;

        Ok(())
    }

    /// 添加钢管到库存，使用事务确保数据一致性
    pub fn add_pipe(&self, pipe: &SteelPipe) -> Result<()> {
        // 验证数据
        pipe.validate()?;

        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let conn = self.conn.lock().unwrap();

        let tx = conn.unchecked_transaction()?;

        // 检查钢管ID是否已存在
        let existing = tx.query_row(
            "SELECT id, quantity FROM pipes WHERE pipe_id = ?",
            params![pipe.pipe_id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i32>(1)?)),
        );

        match existing {
            Ok((_id, _current_qty)) => {
                // 如果存在，更新数量
                tx.execute(
                    "UPDATE pipes SET 
                        diameter = ?, 
                        thickness = ?, 
                        length = ?, 
                        material = ?, 
                        quantity = quantity + ?, 
                        location = ?, 
                        supplier = ?, 
                        last_update = ? 
                     WHERE pipe_id = ?",
                    params![
                        pipe.diameter,
                        pipe.thickness,
                        pipe.length,
                        pipe.material,
                        pipe.quantity,
                        pipe.location,
                        pipe.supplier,
                        now,
                        pipe.pipe_id
                    ],
                )?;
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // 如果不存在，插入新记录
                tx.execute(
                    "INSERT INTO pipes (pipe_id, diameter, thickness, length, material, quantity, location, supplier, entry_date, status) 
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        pipe.pipe_id,
                        pipe.diameter,
                        pipe.thickness,
                        pipe.length,
                        pipe.material,
                        pipe.quantity,
                        pipe.location,
                        pipe.supplier,
                        now,
                        "在库"
                    ],
                )?;
            }
            Err(e) => return Err(e.into()),
        }

        tx.commit()?;
        Ok(())
    }

    /// 获取所有钢管信息
    pub fn get_pipes(&self) -> Result<Vec<SteelPipe>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, pipe_id, diameter, thickness, length, material, 
                    quantity, location, supplier, entry_date, last_update, status 
             FROM pipes ORDER BY entry_date DESC",
        )?;

        let pipe_iter = stmt.query_map([], |row| {
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
        })?;

        let mut pipes = Vec::new();
        for pipe in pipe_iter {
            pipes.push(pipe?);
        }

        Ok(pipes)
    }

    /// 根据钢管ID获取钢管信息
    pub fn get_pipe_by_id(&self, pipe_id: &str) -> Result<Option<SteelPipe>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, pipe_id, diameter, thickness, length, material, 
                    quantity, location, supplier, entry_date, last_update, status 
             FROM pipes WHERE pipe_id = ?",
        )?;

        let mut pipe_iter = stmt.query_map(params![pipe_id], |row| {
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
        })?;

        if let Some(pipe) = pipe_iter.next() {
            return Ok(Some(pipe?));
        }

        Ok(None)
    }

    /// 更新钢管数量，使用事务确保数据一致性
    pub fn update_pipe_quantity(&self, pipe_id: &str, quantity_change: i32) -> Result<()> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let conn = self.conn.lock().unwrap();

        let tx = conn.unchecked_transaction()?;

        // 检查钢管是否存在并获取当前数量
        let current_quantity: i32 = tx.query_row(
            "SELECT quantity FROM pipes WHERE pipe_id = ?",
            params![pipe_id],
            |row| row.get(0),
        )?;

        // 检查数量是否足够
        if current_quantity + quantity_change < 0 {
            return Err(DbError::InsufficientStock {
                current: current_quantity,
                requested: quantity_change,
            });
        }

        tx.execute(
            "UPDATE pipes SET quantity = quantity + ?, last_update = ? WHERE pipe_id = ?",
            params![quantity_change, now, pipe_id],
        )?;

        tx.commit()?;
        Ok(())
    }

    /// 添加出入库记录
    pub fn add_inventory_record(&self, record: &InventoryRecord) -> Result<()> {
        // 验证数据
        record.validate()?;

        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO inventory_records (pipe_id, operation_type, quantity, operation_date, operator, remarks) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                record.pipe_id,
                record.operation_type,
                record.quantity,
                now,
                record.operator,
                record.remarks
            ],
        )?;

        Ok(())
    }

    /// 获取出入库记录，支持筛选
    pub fn get_inventory_records(
        &self,
        pipe_id: Option<&str>,
        operation_type: Option<&str>,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<Vec<InventoryRecord>> {
        let mut query =
            "SELECT id, pipe_id, operation_type, quantity, operation_date, operator, remarks 
                        FROM inventory_records WHERE 1=1"
                .to_string();
        let mut params = Vec::new();

        if let Some(id) = pipe_id {
            query.push_str(" AND pipe_id = ?");
            params.push(id.to_string());
        }

        if let Some(op_type) = operation_type {
            if op_type != "全部" {
                query.push_str(" AND operation_type = ?");
                params.push(op_type.to_string());
            }
        }

        if let Some(start) = start_date {
            query.push_str(" AND operation_date >= ?");
            params.push(start.to_string());
        }

        if let Some(end) = end_date {
            query.push_str(" AND operation_date <= ?");
            params.push(end.to_string());
        }

        query.push_str(" ORDER BY operation_date DESC");

        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&query)?;

        let param_refs: Vec<&dyn rusqlite::ToSql> =
            params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
        let record_iter = stmt.query_map(param_refs.as_slice(), |row| {
            Ok(InventoryRecord {
                id: Some(row.get(0)?),
                pipe_id: row.get(1)?,
                operation_type: row.get(2)?,
                quantity: row.get(3)?,
                operation_date: row.get(4)?,
                operator: row.get(5)?,
                remarks: row.get(6)?,
            })
        })?;

        let mut records = Vec::new();
        for record in record_iter {
            records.push(record?);
        }

        Ok(records)
    }

    /// 获取统计数据
    pub fn get_statistics(&self) -> Result<Statistics> {
        let mut stats = Statistics::default();
        let conn = self.conn.lock().unwrap();

        // 总钢管种类数
        stats.total_types = conn.query_row("SELECT COUNT(*) FROM pipes", [], |row| row.get(0))?;

        // 总钢管数量
        stats.total_quantity = conn.query_row("SELECT SUM(quantity) FROM pipes", [], |row| {
            let val: Option<i64> = row.get(0)?;
            Ok(val.unwrap_or(0) as i32)
        })?;

        // 入库总数
        stats.total_in = conn.query_row(
            "SELECT SUM(quantity) FROM inventory_records WHERE operation_type = '入库'",
            [],
            |row| {
                let val: Option<i64> = row.get(0)?;
                Ok(val.unwrap_or(0) as i32)
            },
        )?;

        // 出库总数
        stats.total_out = conn.query_row(
            "SELECT SUM(quantity) FROM inventory_records WHERE operation_type = '出库'",
            [],
            |row| {
                let val: Option<i64> = row.get(0)?;
                Ok(val.unwrap_or(0) as i32)
            },
        )?;

        Ok(stats)
    }

    /// 导出库存数据为CSV格式
    pub fn export_inventory_to_csv(&self) -> Result<String> {
        let pipes = self.get_pipes()?;
        let mut csv = String::from(
            "钢管编号,直径(毫米),壁厚(毫米),长度(米),材质,数量,存放位置,供应商,入库日期,状态
",
        );

        for pipe in pipes {
            csv.push_str(&format!(
                "{},{:.2},{:.2},{:.2},{},{},{},{},{},{}
",
                pipe.pipe_id,
                pipe.diameter,
                pipe.thickness,
                pipe.length,
                pipe.material,
                pipe.quantity,
                pipe.location.as_ref().unwrap_or(&String::new()),
                pipe.supplier.as_ref().unwrap_or(&String::new()),
                pipe.entry_date,
                pipe.status
            ));
        }

        Ok(csv)
    }

    /// 导出出入库记录为CSV格式
    pub fn export_records_to_csv(
        &self,
        pipe_id: Option<&str>,
        operation_type: Option<&str>,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<String> {
        let records = self.get_inventory_records(pipe_id, operation_type, start_date, end_date)?;
        let mut csv = String::from(
            "钢管编号,操作类型,数量,操作日期,操作员,备注
",
        );

        for record in records {
            csv.push_str(&format!(
                "{},{},{},{},{},{}
",
                record.pipe_id,
                record.operation_type,
                record.quantity,
                record.operation_date,
                record.operator,
                record.remarks.as_ref().unwrap_or(&String::new())
            ));
        }

        Ok(csv)
    }
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

impl Database {
    pub fn delete_pipe(&self, pipe_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let tx = conn.unchecked_transaction()?;

        tx.execute("DELETE FROM inventory_records WHERE pipe_id = ?", params![pipe_id])?;
        tx.execute("DELETE FROM pipes WHERE pipe_id = ?", params![pipe_id])?;

        tx.commit()?;
        Ok(())
    }

    pub fn update_pipe(&self, pipe: &SteelPipe) -> Result<()> {
        pipe.validate()?;

        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "UPDATE pipes SET 
                diameter = ?, thickness = ?, length = ?, material = ?, 
                quantity = ?, location = ?, supplier = ?, 
                last_update = ?, status = ? 
             WHERE pipe_id = ?",
            params![
                pipe.diameter, pipe.thickness, pipe.length, pipe.material,
                pipe.quantity, pipe.location, pipe.supplier,
                now, pipe.status, pipe.pipe_id
            ],
        )?;

        Ok(())
    }

    pub fn get_low_stock_pipes(&self, threshold: i32) -> Result<Vec<SteelPipe>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, pipe_id, diameter, thickness, length, material, 
                    quantity, location, supplier, entry_date, last_update, status 
             FROM pipes WHERE quantity <= ? ORDER BY quantity ASC",
        )?;

        let pipe_iter = stmt.query_map(params![threshold], |row| {
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
        })?;

        let mut pipes = Vec::new();
        for pipe in pipe_iter {
            pipes.push(pipe?);
        }

        Ok(pipes)
    }

    pub fn get_statistics_by_material(&self) -> Result<Vec<MaterialStats>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT material, COUNT(*) as type_count, SUM(quantity) as total_quantity 
             FROM pipes GROUP BY material ORDER BY total_quantity DESC",
        )?;

        let stats_iter = stmt.query_map([], |row| {
            Ok(MaterialStats {
                material: row.get(0)?,
                type_count: row.get(1)?,
                total_quantity: row.get(2)?,
            })
        })?;

        let mut stats = Vec::new();
        for s in stats_iter {
            stats.push(s?);
        }

        Ok(stats)
    }

    pub fn get_recent_records(&self, limit: usize) -> Result<Vec<InventoryRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, pipe_id, operation_type, quantity, operation_date, operator, remarks 
             FROM inventory_records ORDER BY operation_date DESC LIMIT ?",
        )?;

        let record_iter = stmt.query_map(params![limit as i64], |row| {
            Ok(InventoryRecord {
                id: Some(row.get(0)?),
                pipe_id: row.get(1)?,
                operation_type: row.get(2)?,
                quantity: row.get(3)?,
                operation_date: row.get(4)?,
                operator: row.get(5)?,
                remarks: row.get(6)?,
            })
        })?;

        let mut records = Vec::new();
        for record in record_iter {
            records.push(record?);
        }

        Ok(records)
    }

    pub fn get_pipes_paginated(&self, offset: i64, limit: i64) -> Result<Vec<SteelPipe>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, pipe_id, diameter, thickness, length, material, 
                    quantity, location, supplier, entry_date, last_update, status 
             FROM pipes ORDER BY entry_date DESC LIMIT ? OFFSET ?",
        )?;

        let pipe_iter = stmt.query_map(params![limit, offset], |row| {
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
        })?;

        let mut pipes = Vec::new();
        for pipe in pipe_iter {
            pipes.push(pipe?);
        }

        Ok(pipes)
    }

    pub fn get_pipes_count(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count = conn.query_row("SELECT COUNT(*) FROM pipes", [], |row| row.get(0))?;
        Ok(count)
    }

    pub fn export_inventory_to_file(&self, path: &str) -> Result<()> {
        let csv = self.export_inventory_to_csv()?;
        std::fs::write(path, csv)?;
        Ok(())
    }

    pub fn export_records_to_file(&self, path: &str, pipe_id: Option<&str>, operation_type: Option<&str>, start_date: Option<&str>, end_date: Option<&str>) -> Result<()> {
        let csv = self.export_records_to_csv(pipe_id, operation_type, start_date, end_date)?;
        std::fs::write(path, csv)?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn log_operation(&self, operation_type: &str, target_type: &str, target_id: &str, snapshot_before: &str, snapshot_after: &str, operator: &str, remarks: &str) -> Result<()> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO operation_logs (operation_type, target_type, target_id, snapshot_before, snapshot_after, operator, timestamp, remarks)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![operation_type, target_type, target_id, snapshot_before, snapshot_after, operator, now, remarks],
        )?;
        Ok(())
    }

    pub fn get_operation_logs(&self, limit: usize) -> Result<Vec<OperationLog>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, operation_type, target_type, target_id, snapshot_before, snapshot_after, operator, timestamp, remarks
             FROM operation_logs ORDER BY timestamp DESC LIMIT ?",
        )?;
        let log_iter = stmt.query_map(params![limit as i64], |row| {
            Ok(OperationLog {
                id: row.get(0)?,
                operation_type: row.get(1)?,
                target_type: row.get(2)?,
                target_id: row.get(3)?,
                snapshot_before: row.get(4)?,
                snapshot_after: row.get(5)?,
                operator: row.get(6)?,
                timestamp: row.get(7)?,
                remarks: row.get(8)?,
            })
        })?;
        let mut logs = Vec::new();
        for log in log_iter {
            logs.push(log?);
        }
        Ok(logs)
    }

    pub fn undo_operation(&self, log_id: i64) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        let tx = conn.unchecked_transaction()?;

        let (operation_type, target_type, target_id, snapshot_before, snapshot_after) = tx.query_row(
            "SELECT operation_type, target_type, target_id, snapshot_before, snapshot_after FROM operation_logs WHERE id = ?",
            params![log_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, String>(4)?)),
        )?;

        match (operation_type.as_str(), target_type.as_str()) {
            ("add_pipe", "pipe") | ("入库", "pipe") => {
                tx.execute("DELETE FROM pipes WHERE pipe_id = ?", params![target_id])?;
                tx.execute("DELETE FROM inventory_records WHERE pipe_id = ? AND operation_type = '入库' AND operation_date = (SELECT MAX(operation_date) FROM inventory_records WHERE pipe_id = ?)", params![target_id, target_id])?;
            }
            ("delete_pipe", "pipe") => {
                if !snapshot_before.is_empty() {
                    let pipe: SteelPipe = serde_json::from_str(&snapshot_before).map_err(|e| DbError::Validation(format!("恢复数据解析失败: {}", e)))?;
                    tx.execute(
                        "INSERT INTO pipes (pipe_id, diameter, thickness, length, material, quantity, location, supplier, entry_date, last_update, status)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                        params![pipe.pipe_id, pipe.diameter, pipe.thickness, pipe.length, pipe.material, pipe.quantity, pipe.location, pipe.supplier, pipe.entry_date, pipe.last_update, pipe.status],
                    )?;
                }
            }
            ("update_pipe", "pipe") => {
                if !snapshot_before.is_empty() {
                    let pipe: SteelPipe = serde_json::from_str(&snapshot_before).map_err(|e| DbError::Validation(format!("恢复数据解析失败: {}", e)))?;
                    tx.execute(
                        "UPDATE pipes SET diameter = ?, thickness = ?, length = ?, material = ?, quantity = ?, location = ?, supplier = ?, last_update = ?, status = ? WHERE pipe_id = ?",
                        params![pipe.diameter, pipe.thickness, pipe.length, pipe.material, pipe.quantity, pipe.location, pipe.supplier, pipe.last_update, pipe.status, pipe.pipe_id],
                    )?;
                }
            }
            ("exit_pipe", "pipe") | ("出库", "pipe") => {
                if !snapshot_before.is_empty() {
                    let before_qty: i32 = serde_json::from_str(&snapshot_before).unwrap_or(0);
                    let after_qty: i32 = serde_json::from_str(&snapshot_after).unwrap_or(0);
                    let diff = after_qty - before_qty;
                    tx.execute(
                        "UPDATE pipes SET quantity = quantity + ? WHERE pipe_id = ?",
                        params![diff, target_id],
                    )?;
                    tx.execute("DELETE FROM inventory_records WHERE pipe_id = ? AND operation_type = '出库' AND operation_date = (SELECT MAX(operation_date) FROM inventory_records WHERE pipe_id = ? AND operation_type = '出库')", params![target_id, target_id])?;
                }
            }
            _ => {
                return Err(DbError::Validation(format!("不支持撤回的操作: {} - {}", operation_type, target_type)));
            }
        }

        tx.execute("DELETE FROM operation_logs WHERE id = ?", params![log_id])?;
        tx.commit()?;
        Ok(format!("已撤回操作: {} ({})", operation_type, target_id))
    }

    pub fn undo_last_operation(&self) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        let log_id: Option<i64> = conn.query_row(
            "SELECT id FROM operation_logs ORDER BY timestamp DESC LIMIT 1",
            [],
            |row| row.get(0),
        ).ok();

        if let Some(id) = log_id {
            drop(conn);
            self.undo_operation(id)
        } else {
            Err(DbError::Validation("没有可撤回的操作".to_string()))
        }
    }

    pub fn import_pipes_from_csv(&self, csv_content: &str, operator: &str) -> Result<(usize, usize)> {
        let mut success_count = 0;
        let mut fail_count = 0;
        let mut error_msgs = Vec::new();

        for (i, line) in csv_content.lines().enumerate() {
            if i == 0 {
                continue;
            }
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let fields: Vec<&str> = line.split(',').collect();
            if fields.len() < 6 {
                fail_count += 1;
                error_msgs.push(format!("第{}行: 字段不足", i + 1));
                continue;
            }

            let pipe_id = fields[0].trim().to_string();
            let diameter: f64 = match fields[1].trim().parse() {
                Ok(v) => v,
                Err(_) => { fail_count += 1; error_msgs.push(format!("第{}行: 直径无效", i + 1)); continue; }
            };
            let thickness: f64 = match fields[2].trim().parse() {
                Ok(v) => v,
                Err(_) => { fail_count += 1; error_msgs.push(format!("第{}行: 壁厚无效", i + 1)); continue; }
            };
            let length: f64 = match fields[3].trim().parse() {
                Ok(v) => v,
                Err(_) => { fail_count += 1; error_msgs.push(format!("第{}行: 长度无效", i + 1)); continue; }
            };
            let material = fields[4].trim().to_string();
            let quantity: i32 = match fields[5].trim().parse() {
                Ok(v) => v,
                Err(_) => { fail_count += 1; error_msgs.push(format!("第{}行: 数量无效", i + 1)); continue; }
            };

            let location = if fields.len() > 6 && !fields[6].trim().is_empty() { Some(fields[6].trim().to_string()) } else { None };
            let supplier = if fields.len() > 7 && !fields[7].trim().is_empty() { Some(fields[7].trim().to_string()) } else { None };

            let pipe = SteelPipe {
                id: None,
                pipe_id: pipe_id.clone(),
                diameter, thickness, length, material, quantity,
                location, supplier,
                entry_date: String::new(),
                last_update: None,
                status: "在库".to_string(),
            };

            match self.add_pipe(&pipe) {
                Ok(()) => {
                    let record = InventoryRecord {
                        id: None,
                        pipe_id: pipe_id.clone(),
                        operation_type: "入库".to_string(),
                        quantity,
                        operation_date: String::new(),
                        operator: operator.to_string(),
                        remarks: Some("批量导入".to_string()),
                    };
                    let _ = self.add_inventory_record(&record);
                    success_count += 1;
                }
                Err(e) => {
                    fail_count += 1;
                    error_msgs.push(format!("第{}行: {}", i + 1, e));
                }
            }
        }

        if !error_msgs.is_empty() {
            let msg = error_msgs.iter().take(10).cloned().collect::<Vec<_>>().join("; ");
            Err(DbError::Validation(format!("导入完成，成功{}条，失败{}条。部分错误: {}", success_count, fail_count, msg)))
        } else {
            Ok((success_count, fail_count))
        }
    }

    pub fn import_pipes_from_excel(&self, file_path: &str, operator: &str) -> Result<(usize, usize)> {
        let mut workbook = calamine::open_workbook_auto(file_path)
            .map_err(|e| DbError::Validation(format!("无法打开Excel文件: {}", e)))?;

        let range = workbook.worksheet_range_at(0)
            .ok_or_else(|| DbError::Validation("Excel文件中没有工作表".to_string()))?
            .map_err(|e| DbError::Validation(format!("读取工作表失败: {}", e)))?;

        let mut success_count = 0;
        let mut fail_count = 0;
        let mut error_msgs = Vec::new();

        for (row_idx, row) in range.rows().enumerate() {
            if row_idx == 0 {
                continue;
            }
            if row.len() < 6 {
                fail_count += 1;
                error_msgs.push(format!("第{}行: 字段不足", row_idx + 1));
                continue;
            }

            let pipe_id = match row[0].as_string() {
                Some(s) if !s.trim().is_empty() => s.trim().to_string(),
                _ => { fail_count += 1; error_msgs.push(format!("第{}行: 钢管编号无效", row_idx + 1)); continue; }
            };
            let diameter: f64 = match row[1].as_f64() {
                Some(v) if v > 0.0 => v,
                _ => { fail_count += 1; error_msgs.push(format!("第{}行: 直径无效", row_idx + 1)); continue; }
            };
            let thickness: f64 = match row[2].as_f64() {
                Some(v) if v > 0.0 => v,
                _ => { fail_count += 1; error_msgs.push(format!("第{}行: 壁厚无效", row_idx + 1)); continue; }
            };
            let length: f64 = match row[3].as_f64() {
                Some(v) if v > 0.0 => v,
                _ => { fail_count += 1; error_msgs.push(format!("第{}行: 长度无效", row_idx + 1)); continue; }
            };
            let material = match row[4].as_string() {
                Some(s) if !s.trim().is_empty() => s.trim().to_string(),
                _ => { fail_count += 1; error_msgs.push(format!("第{}行: 材质无效", row_idx + 1)); continue; }
            };
            let quantity: i32 = match row[5].as_i64() {
                Some(v) if v > 0 => v as i32,
                _ => { fail_count += 1; error_msgs.push(format!("第{}行: 数量无效", row_idx + 1)); continue; }
            };

            let location = if row.len() > 6 {
                row[6].as_string().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
            } else { None };
            let supplier = if row.len() > 7 {
                row[7].as_string().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
            } else { None };

            let pipe = SteelPipe {
                id: None,
                pipe_id: pipe_id.clone(),
                diameter, thickness, length, material, quantity,
                location, supplier,
                entry_date: String::new(),
                last_update: None,
                status: "在库".to_string(),
            };

            match self.add_pipe(&pipe) {
                Ok(()) => {
                    let record = InventoryRecord {
                        id: None,
                        pipe_id: pipe_id.clone(),
                        operation_type: "入库".to_string(),
                        quantity,
                        operation_date: String::new(),
                        operator: operator.to_string(),
                        remarks: Some("Excel批量导入".to_string()),
                    };
                    let _ = self.add_inventory_record(&record);
                    success_count += 1;
                }
                Err(e) => {
                    fail_count += 1;
                    error_msgs.push(format!("第{}行: {}", row_idx + 1, e));
                }
            }
        }

        if !error_msgs.is_empty() {
            let msg = error_msgs.iter().take(10).cloned().collect::<Vec<_>>().join("; ");
            Err(DbError::Validation(format!("导入完成，成功{}条，失败{}条。部分错误: {}", success_count, fail_count, msg)))
        } else {
            Ok((success_count, fail_count))
        }
    }

    pub fn export_inventory_to_excel(&self, path: &str) -> Result<()> {
        let pipes = self.get_pipes()?;
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        let header_fmt = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0x4472C4))
            .set_font_color(Color::White)
            .set_border(FormatBorder::Thin);
        let cell_fmt = Format::new().set_border(FormatBorder::Thin);
        let num_fmt = Format::new().set_border(FormatBorder::Thin).set_num_format("0.00");
        let int_fmt = Format::new().set_border(FormatBorder::Thin).set_num_format("0");

        let headers = ["钢管编号", "直径(mm)", "壁厚(mm)", "长度(m)", "材质", "数量", "存放位置", "供应商", "入库日期", "状态"];
        for (col, &header) in headers.iter().enumerate() {
            worksheet.write_string_with_format(0, col as u16, header, &header_fmt)
                .map_err(|e: XlsxError| DbError::Validation(e.to_string()))?;
        }

        for (row_idx, pipe) in pipes.iter().enumerate() {
            let row = (row_idx + 1) as u32;
            worksheet.write_string_with_format(row, 0, &pipe.pipe_id, &cell_fmt)
                .map_err(|e: XlsxError| DbError::Validation(e.to_string()))?;
            worksheet.write_number_with_format(row, 1, pipe.diameter, &num_fmt)
                .map_err(|e: XlsxError| DbError::Validation(e.to_string()))?;
            worksheet.write_number_with_format(row, 2, pipe.thickness, &num_fmt)
                .map_err(|e: XlsxError| DbError::Validation(e.to_string()))?;
            worksheet.write_number_with_format(row, 3, pipe.length, &num_fmt)
                .map_err(|e: XlsxError| DbError::Validation(e.to_string()))?;
            worksheet.write_string_with_format(row, 4, &pipe.material, &cell_fmt)
                .map_err(|e: XlsxError| DbError::Validation(e.to_string()))?;
            worksheet.write_number_with_format(row, 5, pipe.quantity as f64, &int_fmt)
                .map_err(|e: XlsxError| DbError::Validation(e.to_string()))?;
            worksheet.write_string_with_format(row, 6, pipe.location.as_deref().unwrap_or(""), &cell_fmt)
                .map_err(|e: XlsxError| DbError::Validation(e.to_string()))?;
            worksheet.write_string_with_format(row, 7, pipe.supplier.as_deref().unwrap_or(""), &cell_fmt)
                .map_err(|e: XlsxError| DbError::Validation(e.to_string()))?;
            worksheet.write_string_with_format(row, 8, &pipe.entry_date, &cell_fmt)
                .map_err(|e: XlsxError| DbError::Validation(e.to_string()))?;
            worksheet.write_string_with_format(row, 9, &pipe.status, &cell_fmt)
                .map_err(|e: XlsxError| DbError::Validation(e.to_string()))?;
        }

        worksheet.set_column_width(0, 16.0).ok();
        worksheet.set_column_width(1, 12.0).ok();
        worksheet.set_column_width(2, 12.0).ok();
        worksheet.set_column_width(3, 12.0).ok();
        worksheet.set_column_width(4, 14.0).ok();
        worksheet.set_column_width(5, 10.0).ok();
        worksheet.set_column_width(6, 16.0).ok();
        worksheet.set_column_width(7, 16.0).ok();
        worksheet.set_column_width(8, 22.0).ok();
        worksheet.set_column_width(9, 10.0).ok();

        workbook.save(path).map_err(|e: XlsxError| DbError::Validation(e.to_string()))?;
        Ok(())
    }

    pub fn export_records_to_excel(&self, path: &str, pipe_id: Option<&str>, operation_type: Option<&str>, start_date: Option<&str>, end_date: Option<&str>) -> Result<()> {
        let records = self.get_inventory_records(pipe_id, operation_type, start_date, end_date)?;
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        let header_fmt = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0x548235))
            .set_font_color(Color::White)
            .set_border(FormatBorder::Thin);
        let cell_fmt = Format::new().set_border(FormatBorder::Thin);
        let int_fmt = Format::new().set_border(FormatBorder::Thin).set_num_format("0");

        let headers = ["钢管编号", "操作类型", "数量", "操作日期", "操作员", "备注"];
        for (col, &header) in headers.iter().enumerate() {
            worksheet.write_string_with_format(0, col as u16, header, &header_fmt)
                .map_err(|e: XlsxError| DbError::Validation(e.to_string()))?;
        }

        for (row_idx, record) in records.iter().enumerate() {
            let row = (row_idx + 1) as u32;
            worksheet.write_string_with_format(row, 0, &record.pipe_id, &cell_fmt)
                .map_err(|e: XlsxError| DbError::Validation(e.to_string()))?;
            worksheet.write_string_with_format(row, 1, &record.operation_type, &cell_fmt)
                .map_err(|e: XlsxError| DbError::Validation(e.to_string()))?;
            worksheet.write_number_with_format(row, 2, record.quantity as f64, &int_fmt)
                .map_err(|e: XlsxError| DbError::Validation(e.to_string()))?;
            worksheet.write_string_with_format(row, 3, &record.operation_date, &cell_fmt)
                .map_err(|e: XlsxError| DbError::Validation(e.to_string()))?;
            worksheet.write_string_with_format(row, 4, &record.operator, &cell_fmt)
                .map_err(|e: XlsxError| DbError::Validation(e.to_string()))?;
            worksheet.write_string_with_format(row, 5, record.remarks.as_deref().unwrap_or(""), &cell_fmt)
                .map_err(|e: XlsxError| DbError::Validation(e.to_string()))?;
        }

        worksheet.set_column_width(0, 16.0).ok();
        worksheet.set_column_width(1, 12.0).ok();
        worksheet.set_column_width(2, 10.0).ok();
        worksheet.set_column_width(3, 22.0).ok();
        worksheet.set_column_width(4, 12.0).ok();
        worksheet.set_column_width(5, 20.0).ok();

        workbook.save(path).map_err(|e: XlsxError| DbError::Validation(e.to_string()))?;
        Ok(())
    }
}
