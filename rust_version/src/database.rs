
use rusqlite::{Connection, Result as SqliteResult, params, Transaction};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local, NaiveDateTime};
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
            return Err(DbError::Validation(format!("无效的材质类型: {}", self.material)));
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
            return Err(DbError::Validation("操作类型必须是'入库'或'出库'".to_string()));
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
            conn: Arc::new(Mutex::new(conn))
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

        Ok(())
    }

    /// 添加钢管到库存，使用事务确保数据一致性
    pub fn add_pipe(&self, pipe: &SteelPipe) -> Result<()> {
        // 验证数据
        pipe.validate()?;

        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let tx = self.conn.unchecked_transaction()
            .context("开始事务失败")?;

        // 检查钢管ID是否已存在
        let existing = tx.query_row(
            "SELECT id, quantity FROM pipes WHERE pipe_id = ?",
            params![pipe.pipe_id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i32>(1)?)),
        );

        match existing {
            Ok((id, current_qty)) => {
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
                ).context("更新钢管信息失败")?;
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
                ).context("插入钢管信息失败")?;
            }
            Err(e) => return Err(e).context("查询钢管信息失败"),
        }

        tx.commit().context("提交事务失败")?;
        Ok(())
    }

    /// 获取所有钢管信息
    pub fn get_pipes(&self) -> Result<Vec<SteelPipe>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, pipe_id, diameter, thickness, length, material, 
                    quantity, location, supplier, entry_date, last_update, status 
             FROM pipes ORDER BY entry_date DESC"
        ).context("准备查询语句失败")?;

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
        }).context("执行查询失败")?;

        let mut pipes = Vec::new();
        for pipe in pipe_iter {
            pipes.push(pipe.context("解析钢管数据失败")?);
        }

        Ok(pipes)
    }

    /// 根据钢管ID获取钢管信息
    pub fn get_pipe_by_id(&self, pipe_id: &str) -> Result<Option<SteelPipe>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, pipe_id, diameter, thickness, length, material, 
                    quantity, location, supplier, entry_date, last_update, status 
             FROM pipes WHERE pipe_id = ?"
        ).context("准备查询语句失败")?;

        let pipe_iter = stmt.query_map(params![pipe_id], |row| {
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
        }).context("执行查询失败")?;

        for pipe in pipe_iter {
            return Ok(Some(pipe.context("解析钢管数据失败")?));
        }

        Ok(None)
    }

    /// 更新钢管数量，使用事务确保数据一致性
    pub fn update_pipe_quantity(&self, pipe_id: &str, quantity_change: i32) -> Result<()> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let tx = self.conn.unchecked_transaction()
            .context("开始事务失败")?;

        // 检查钢管是否存在并获取当前数量
        let current_quantity: i32 = tx.query_row(
            "SELECT quantity FROM pipes WHERE pipe_id = ?",
            params![pipe_id],
            |row| row.get(0),
        ).context("查询钢管数量失败")?;

        // 检查数量是否足够
        if current_quantity + quantity_change < 0 {
            bail!("库存不足，当前库存: {}, 请求变更: {}", current_quantity, quantity_change);
        }

        tx.execute(
            "UPDATE pipes SET quantity = quantity + ?, last_update = ? WHERE pipe_id = ?",
            params![quantity_change, now, pipe_id],
        ).context("更新钢管数量失败")?;

        tx.commit().context("提交事务失败")?;
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
        let mut query = "SELECT id, pipe_id, operation_type, quantity, operation_date, operator, remarks 
                        FROM inventory_records WHERE 1=1".to_string();
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

        let mut stmt = self.conn.prepare(&query)
            .context("准备查询语句失败")?;

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
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
        }).context("执行查询失败")?;

        let mut records = Vec::new();
        for record in record_iter {
            records.push(record.context("解析出入库记录失败")?);
        }

        Ok(records)
    }

    /// 获取统计数据
    pub fn get_statistics(&self) -> Result<Statistics> {
        let mut stats = Statistics::default();

        // 总钢管种类数
        stats.total_types = self.conn.query_row(
            "SELECT COUNT(*) FROM pipes", 
            [], 
            |row| row.get(0)
        ).context("查询总种类数失败")?;

        // 总钢管数量
        stats.total_quantity = self.conn.query_row(
            "SELECT SUM(quantity) FROM pipes", 
            [], 
            |row| {
                let val: Option<i64> = row.get(0)?;
                Ok(val.unwrap_or(0) as i32)
            }
        ).context("查询总数量失败")?;

        // 入库总数
        stats.total_in = self.conn.query_row(
            "SELECT SUM(quantity) FROM inventory_records WHERE operation_type = '入库'", 
            [], 
            |row| {
                let val: Option<i64> = row.get(0)?;
                Ok(val.unwrap_or(0) as i32)
            }
        ).context("查询入库总数失败")?;

        // 出库总数
        stats.total_out = self.conn.query_row(
            "SELECT SUM(quantity) FROM inventory_records WHERE operation_type = '出库'", 
            [], 
            |row| {
                let val: Option<i64> = row.get(0)?;
                Ok(val.unwrap_or(0) as i32)
            }
        ).context("查询出库总数失败")?;

        Ok(stats)
    }

    /// 导出库存数据为CSV格式
    pub fn export_inventory_to_csv(&self) -> Result<String> {
        let pipes = self.get_pipes()?;
        let mut csv = String::from("钢管编号,直径(毫米),壁厚(毫米),长度(米),材质,数量,存放位置,供应商,入库日期,状态
");

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
        let mut csv = String::from("钢管编号,操作类型,数量,操作日期,操作员,备注
");

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

