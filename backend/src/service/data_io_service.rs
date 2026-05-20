use std::collections::HashMap;
use std::io::Cursor;

use calamine::{open_workbook_from_rs, Data as CellData, Reader, Xlsx};
use chrono::Utc;
use csv::Writer as CsvWriter;
use encoding_rs::GBK;
use rust_xlsxwriter::{Format, Workbook, FormatBorder, Color};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportResult {
    pub success_count: i32,
    pub fail_count: i32,
    pub errors: Vec<RowError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RowError {
    pub row: i32,
    pub reason: String,
    pub raw_data: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct ImportConfig {
    #[serde(default)]
    pub on_duplicate: DuplicateStrategy,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum DuplicateStrategy {
    #[default]
    Skip,
    Overwrite,
}


#[derive(Debug, Deserialize, Default)]
pub struct ExportFilter {
    pub format: Option<String>,
    pub grade: Option<String>,
    pub status: Option<String>,
    pub location: Option<String>,
    pub pipe_type: Option<String>,
    pub search: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub supplier_id: Option<String>,
    pub customer_id: Option<String>,
    pub inbound_type: Option<String>,
    pub outbound_type: Option<String>,
}

impl ExportFilter {
    pub fn fmt(&self) -> &str {
        self.format.as_deref().unwrap_or("xlsx")
    }
}

// ── Field definition for header mapping ─────────────────────────────────────

struct FieldDef {
    name: &'static str,
    aliases: &'static [&'static str],
}

impl FieldDef {
    fn matches(&self, header: &str) -> bool {
        let h = header.trim();
        h.eq_ignore_ascii_case(self.name)
            || self.aliases.iter().any(|a| h.eq_ignore_ascii_case(a))
            || self.aliases.iter().any(|a| h.contains(a))
            || h.contains(self.name)
    }
}

const SEAMLESS_FIELDS: &[FieldDef] = &[
    FieldDef { name: "pipe_number", aliases: &["管号", "编号", "pipe_no", "pipe num"] },
    FieldDef { name: "grade", aliases: &["钢级"] },
    FieldDef { name: "od", aliases: &["外径", "outer diameter"] },
    FieldDef { name: "wt", aliases: &["壁厚", "wall thickness", "weight per foot"] },
    FieldDef { name: "length", aliases: &["长度"] },
    FieldDef { name: "weight", aliases: &["重量", "单重"] },
    FieldDef { name: "connection_type", aliases: &["接箍类型", "连接类型", "connection"] },
    FieldDef { name: "heat_number", aliases: &["炉号", "heat no", "heat"] },
    FieldDef { name: "production_date", aliases: &["生产日期", "生产时间", "prod date"] },
    FieldDef { name: "status", aliases: &["状态", "pipe status"] },
    FieldDef { name: "location", aliases: &["位置", "库位", "仓库位置"] },
    FieldDef { name: "notes", aliases: &["备注", "说明", "remark"] },
];

const SCREEN_FIELDS: &[FieldDef] = &[
    FieldDef { name: "pipe_number", aliases: &["管号", "编号", "pipe_no", "pipe num"] },
    FieldDef { name: "grade", aliases: &["钢级"] },
    FieldDef { name: "od", aliases: &["外径", "outer diameter"] },
    FieldDef { name: "wt", aliases: &["壁厚", "wall thickness", "weight per foot"] },
    FieldDef { name: "length", aliases: &["长度"] },
    FieldDef { name: "weight", aliases: &["重量", "单重"] },
    FieldDef { name: "screen_type", aliases: &["筛管类型", "滤管类型", "screen"] },
    FieldDef { name: "slot_width", aliases: &["缝宽", "slot", "缝隙"] },
    FieldDef { name: "open_area", aliases: &["开孔率", "open area", "open area %"] },
    FieldDef { name: "connection_type", aliases: &["接箍类型", "连接类型", "connection"] },
    FieldDef { name: "heat_number", aliases: &["炉号", "heat no", "heat"] },
    FieldDef { name: "production_date", aliases: &["生产日期", "生产时间", "prod date"] },
    FieldDef { name: "status", aliases: &["状态", "pipe status"] },
    FieldDef { name: "location", aliases: &["位置", "库位", "仓库位置"] },
    FieldDef { name: "notes", aliases: &["备注", "说明", "remark"] },
];

// ── Service ──────────────────────────────────────────────────────────────────

pub struct DataIoService {
    pool: SqlitePool,
}

impl DataIoService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Templates
    // ═══════════════════════════════════════════════════════════════════════

    pub fn generate_seamless_template(&self) -> AppResult<Vec<u8>> {
        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet();

        let hdr_fmt = Format::new()
            .set_bold()
            .set_border(FormatBorder::Thin)
            .set_background_color(Color::RGB(0x4472C4))
            .set_font_color(Color::White);

        let headers: &[(&str, &str)] = &[
            ("pipe_number", "管号"),
            ("grade", "钢级"),
            ("od", "外径(in)"),
            ("wt", "壁厚(lb/ft)"),
            ("length", "长度(ft)"),
            ("weight", "重量(lb)"),
            ("connection_type", "接箍类型"),
            ("heat_number", "炉号"),
            ("production_date", "生产日期"),
            ("location", "位置"),
            ("notes", "备注"),
        ];

        for (col, (en, cn)) in headers.iter().enumerate() {
            sheet
                .write_string_with_format(0, col as u16, format!("{} ({})", en, cn), &hdr_fmt)
                .map_err(|e| AppError::Internal(format!("Excel write: {}", e)))?;
        }

        let example: &[&str] = &[
            "", "J55", "4.500", "11.60", "40.0", "520.0",
            "BTC", "H2405", "2024-05-01", "A1-R01", "",
        ];
        let ex_fmt = Format::new().set_font_color(Color::RGB(0x808080));
        for (col, val) in example.iter().enumerate() {
            sheet
                .write_string_with_format(1, col as u16, *val, &ex_fmt)
                .map_err(|e| AppError::Internal(format!("Excel write: {}", e)))?;
        }

        for col in 0..headers.len() as u16 {
            sheet
                .set_column_width(col, 20)
                .map_err(|e| AppError::Internal(format!("Excel width: {}", e)))?;
        }

        workbook
            .save_to_buffer()
            .map_err(|e| AppError::Internal(format!("Excel save: {}", e)))
    }

    pub fn generate_screen_template(&self) -> AppResult<Vec<u8>> {
        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet();

        let hdr_fmt = Format::new()
            .set_bold()
            .set_border(FormatBorder::Thin)
            .set_background_color(Color::RGB(0x4472C4))
            .set_font_color(Color::White);

        let headers: &[(&str, &str)] = &[
            ("pipe_number", "管号"),
            ("grade", "钢级"),
            ("od", "外径(in)"),
            ("wt", "壁厚(lb/ft)"),
            ("length", "长度(ft)"),
            ("weight", "重量(lb)"),
            ("screen_type", "筛管类型"),
            ("slot_width", "缝宽(mm)"),
            ("open_area", "开孔率(%)"),
            ("connection_type", "接箍类型"),
            ("heat_number", "炉号"),
            ("production_date", "生产日期"),
            ("location", "位置"),
            ("notes", "备注"),
        ];

        for (col, (en, cn)) in headers.iter().enumerate() {
            sheet
                .write_string_with_format(0, col as u16, format!("{} ({})", en, cn), &hdr_fmt)
                .map_err(|e| AppError::Internal(format!("Excel write: {}", e)))?;
        }

        let example: &[&str] = &[
            "", "J55", "4.500", "11.60", "40.0", "520.0",
            "wire_wrapped", "0.30", "4.5", "BTC", "H2405",
            "2024-05-01", "A1-R01", "",
        ];
        let ex_fmt = Format::new().set_font_color(Color::RGB(0x808080));
        for (col, val) in example.iter().enumerate() {
            sheet
                .write_string_with_format(1, col as u16, *val, &ex_fmt)
                .map_err(|e| AppError::Internal(format!("Excel write: {}", e)))?;
        }

        for col in 0..headers.len() as u16 {
            sheet
                .set_column_width(col, 20)
                .map_err(|e| AppError::Internal(format!("Excel width: {}", e)))?;
        }

        workbook
            .save_to_buffer()
            .map_err(|e| AppError::Internal(format!("Excel save: {}", e)))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Import — seamless pipes
    // ═══════════════════════════════════════════════════════════════════════

    pub async fn import_seamless_pipes(
        &self,
        data: Vec<u8>,
        content_type: &str,
        config: &ImportConfig,
    ) -> AppResult<ImportResult> {
        let rows = self.parse_file(data, content_type, SEAMLESS_FIELDS)?;
        self.batch_insert_seamless(rows, config).await
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Import — screen pipes
    // ═══════════════════════════════════════════════════════════════════════

    pub async fn import_screen_pipes(
        &self,
        data: Vec<u8>,
        content_type: &str,
        config: &ImportConfig,
    ) -> AppResult<ImportResult> {
        let rows = self.parse_file(data, content_type, SCREEN_FIELDS)?;
        self.batch_insert_screen(rows, config).await
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Export — inventory (stock_summary view)
    // ═══════════════════════════════════════════════════════════════════════

    pub async fn export_inventory(&self, filter: &ExportFilter) -> AppResult<Vec<u8>> {
        let rows = self.query_stock(filter).await?;

        let headers = &[
            "Pipe ID", "Type", "Pipe Number", "Grade", "Status", "Location",
            "OD (in)", "WT (lb/ft)", "Length (ft)", "Weight (lb)", "Notes",
        ];
        let data: Vec<Vec<String>> = rows
            .into_iter()
            .map(|r| {
                vec![
                    r.0, r.1, r.2, r.3, r.4, r.5,
                    r.6.to_string(), r.7.to_string(), r.8.to_string(), r.9.to_string(),
                    r.10.unwrap_or_default(),
                ]
            })
            .collect();

        self.write_file(headers, &data, filter.fmt())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Export — inbound records
    // ═══════════════════════════════════════════════════════════════════════

    pub async fn export_inbound(&self, filter: &ExportFilter) -> AppResult<Vec<u8>> {
        let rows = self.query_inbound(filter).await?;
        let headers = &[
            "Inbound No", "Type", "Supplier ID", "Order ID", "Operator ID",
            "Total Items", "Notes", "Created At",
        ];
        let data: Vec<Vec<String>> = rows
            .into_iter()
            .map(|r| {
                vec![
                    r.0, r.1, r.2.unwrap_or_default(), r.3.unwrap_or_default(),
                    r.4, r.5.to_string(), r.6.unwrap_or_default(), r.7,
                ]
            })
            .collect();
        self.write_file(headers, &data, filter.fmt())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Export — outbound records
    // ═══════════════════════════════════════════════════════════════════════

    pub async fn export_outbound(&self, filter: &ExportFilter) -> AppResult<Vec<u8>> {
        let rows = self.query_outbound(filter).await?;
        let headers = &[
            "Outbound No", "Type", "Customer ID", "Order ID", "Operator ID",
            "Total Items", "Notes", "Created At",
        ];
        let data: Vec<Vec<String>> = rows
            .into_iter()
            .map(|r| {
                vec![
                    r.0, r.1, r.2.unwrap_or_default(), r.3.unwrap_or_default(),
                    r.4, r.5.to_string(), r.6.unwrap_or_default(), r.7,
                ]
            })
            .collect();
        self.write_file(headers, &data, filter.fmt())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Export — pipes (seamless + screen merged)
    // ═══════════════════════════════════════════════════════════════════════

    pub async fn export_pipes(&self, filter: &ExportFilter) -> AppResult<Vec<u8>> {
        let headers = &[
            "Pipe Number", "Type", "Grade", "OD (in)", "WT (lb/ft)",
            "Length (ft)", "Weight (lb)", "Screen Type", "Slot Width (mm)",
            "Open Area (%)", "Connection Type", "Heat Number",
            "Production Date", "Status", "Location", "Notes",
        ];

        let mut data: Vec<Vec<String>> = Vec::new();

        let pt = filter.pipe_type.as_deref().unwrap_or("all");
        if pt == "all" || pt == "seamless" {
            let rows = self.query_seamless(filter).await?;
            for r in rows {
                data.push(vec![
                    r.0, "seamless".into(), r.1, r.2.to_string(), r.3.to_string(),
                    r.4.to_string(), r.5.to_string(), String::new(), String::new(),
                    String::new(), r.6.unwrap_or_default(), r.7.unwrap_or_default(),
                    r.8.unwrap_or_default(), r.9, r.10.unwrap_or_default(),
                    r.11.unwrap_or_default(),
                ]);
            }
        }
        if pt == "all" || pt == "screen" {
            let rows = self.query_screen(filter).await?;
            for r in rows {
                data.push(vec![
                    r.0, "screen".into(), r.1, r.2.to_string(), r.3.to_string(),
                    r.4.to_string(), r.5.to_string(), r.6,
                    r.7.map(|v| v.to_string()).unwrap_or_default(),
                    r.8.map(|v| v.to_string()).unwrap_or_default(),
                    r.9.unwrap_or_default(), r.10.unwrap_or_default(),
                    r.11.unwrap_or_default(), r.12, r.13.unwrap_or_default(),
                    r.14.unwrap_or_default(),
                ]);
            }
        }

        self.write_file(headers, &data, filter.fmt())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Private — file parsing
    // ═══════════════════════════════════════════════════════════════════════

    fn parse_file(
        &self,
        data: Vec<u8>,
        content_type: &str,
        fields: &[FieldDef],
    ) -> AppResult<Vec<HashMap<String, String>>> {
        let ct = content_type.to_lowercase();
        let is_excel = ct.contains("spreadsheetml")
            || ct.contains("excel")
            || ct.ends_with("xlsx")
            || ct.ends_with("xls")
            || (data.len() > 4 && data[0] == 0x50 && data[1] == 0x4b);

        if is_excel {
            self.parse_excel(data, fields)
        } else {
            self.parse_csv(data, fields)
        }
    }

    fn parse_excel(
        &self,
        data: Vec<u8>,
        fields: &[FieldDef],
    ) -> AppResult<Vec<HashMap<String, String>>> {
        let cursor = Cursor::new(data);
        let mut workbook: Xlsx<Cursor<Vec<u8>>> = open_workbook_from_rs(cursor)
            .map_err(|e| AppError::BadRequest(format!("Cannot open Excel: {}", e)))?;

        let sheet = workbook
            .worksheet_range_at(0)
            .ok_or_else(|| AppError::BadRequest("Excel has no sheets".into()))?
            .map_err(|e| AppError::BadRequest(format!("Read sheet error: {}", e)))?;

        let mut rows_iter = sheet.rows();
        let header_row = rows_iter
            .next()
            .ok_or_else(|| AppError::BadRequest("Excel has no header row".into()))?;

        let col_map = self.map_excel_headers(header_row, fields)?;

        let mut results = Vec::new();
        for (idx, row) in rows_iter.enumerate() {
            let mut rec = HashMap::new();
            for (ci, cell) in row.iter().enumerate() {
                if let Some(fname) = col_map.get(&ci) {
                    let val = match cell {
                        CellData::String(s) => s.clone(),
                        CellData::Float(f) => f.to_string(),
                        CellData::Int(i) => i.to_string(),
                        CellData::Empty => String::new(),
                        CellData::Bool(b) => b.to_string(),
                        _ => String::new(),
                    };
                    rec.insert(fname.clone(), val);
                }
            }
            rec.insert("_row".into(), (idx + 2).to_string());
            results.push(rec);
        }

        Ok(results)
    }

    fn parse_csv(
        &self,
        data: Vec<u8>,
        fields: &[FieldDef],
    ) -> AppResult<Vec<HashMap<String, String>>> {
        let text = match std::str::from_utf8(&data) {
            Ok(s) => s.to_owned(),
            Err(_) => {
                let (decoded, _, _) = GBK.decode(&data);
                decoded.into_owned()
            }
        };

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_reader(Cursor::new(text));

        let raw_headers: Vec<String> = reader
            .headers()
            .map_err(|e| AppError::BadRequest(format!("CSV headers: {}", e)))?
            .iter()
            .map(|h| h.trim().to_string())
            .collect();

        let mut col_map: HashMap<usize, String> = HashMap::new();
        for (ci, h) in raw_headers.iter().enumerate() {
            for f in fields {
                if f.matches(h) {
                    col_map.insert(ci, f.name.to_string());
                    break;
                }
            }
        }

        if col_map.is_empty() {
            return Err(AppError::BadRequest(
                "Cannot map CSV headers to known fields. Expected: 管号/钢级/外径/... or pipe_number/grade/od/..."
                    .into(),
            ));
        }

        let mut results = Vec::new();
        for (idx, row_res) in reader.records().enumerate() {
            let row = row_res
                .map_err(|e| AppError::BadRequest(format!("CSV row {}: {}", idx + 2, e)))?;
            let mut rec = HashMap::new();
            for (ci, val) in row.iter().enumerate() {
                if let Some(fname) = col_map.get(&ci) {
                    rec.insert(fname.clone(), val.trim().to_string());
                }
            }
            rec.insert("_row".into(), (idx + 2).to_string());
            results.push(rec);
        }

        Ok(results)
    }

    fn map_excel_headers(
        &self,
        header_row: &[CellData],
        fields: &[FieldDef],
    ) -> AppResult<HashMap<usize, String>> {
        let mut mapping = HashMap::new();
        for (ci, cell) in header_row.iter().enumerate() {
            let text = match cell {
                CellData::String(s) => s.trim().to_string(),
                _ => continue,
            };
            for f in fields {
                if f.matches(&text) {
                    mapping.insert(ci, f.name.to_string());
                    break;
                }
            }
        }
        if mapping.is_empty() {
            return Err(AppError::BadRequest(
                "Could not map any Excel headers to known fields".into(),
            ));
        }
        Ok(mapping)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Private — batch inserts
    // ═══════════════════════════════════════════════════════════════════════

    async fn batch_insert_seamless(
        &self,
        rows: Vec<HashMap<String, String>>,
        config: &ImportConfig,
    ) -> AppResult<ImportResult> {
        let mut success = 0i32;
        let mut fail = 0i32;
        let mut errors = Vec::new();

        for rec in rows {
            let row_num = rec.get("_row").cloned().unwrap_or_default();
            let raw = format!("{:?}", rec);

            let pipe_number = match rec.get("pipe_number").filter(|v| !v.trim().is_empty()) {
                Some(n) => n.trim().to_string(),
                None => {
                    fail += 1;
                    errors.push(RowError {
                        row: row_num.parse().unwrap_or(0),
                        reason: "Missing pipe_number".into(),
                        raw_data: raw,
                    });
                    continue;
                }
            };
            let grade = match rec.get("grade").filter(|v| !v.trim().is_empty()) {
                Some(v) => v.trim().to_string(),
                None => {
                    fail += 1;
                    errors.push(RowError {
                        row: row_num.parse().unwrap_or(0),
                        reason: "Missing grade".into(),
                        raw_data: raw,
                    });
                    continue;
                }
            };

            let od = rec.get("od").and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
            let wt = rec.get("wt").and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
            let len = rec.get("length").and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
            let wgt = rec.get("weight").and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);

            let conn = rec
                .get("connection_type")
                .and_then(|v| if v.is_empty() { None } else { Some(v.clone()) });
            let heat = rec
                .get("heat_number")
                .and_then(|v| if v.is_empty() { None } else { Some(v.clone()) });
            let prod_date = rec
                .get("production_date")
                .and_then(|v| if v.is_empty() { None } else { Some(v.clone()) });
            let status = rec
                .get("status")
                .filter(|v| !v.is_empty())
                .cloned()
                .unwrap_or_else(|| "in_stock".to_string());
            let loc = rec
                .get("location")
                .and_then(|v| if v.is_empty() { None } else { Some(v.clone()) });
            let notes = rec
                .get("notes")
                .and_then(|v| if v.is_empty() { None } else { Some(v.clone()) });

            // Check duplicate
            let existing: Option<(String,)> =
                sqlx::query_as("SELECT id FROM seamless_pipes WHERE pipe_number = ?1 AND deleted_at IS NULL")
                    .bind(&pipe_number)
                    .fetch_optional(&self.pool)
                    .await
                    .map_err(AppError::from)?;

            let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

            if let Some((existing_id,)) = existing {
                if config.on_duplicate == DuplicateStrategy::Skip {
                    fail += 1;
                    errors.push(RowError {
                        row: row_num.parse().unwrap_or(0),
                        reason: format!("Duplicate pipe_number: {}", pipe_number),
                        raw_data: raw,
                    });
                    continue;
                }
                // Overwrite
                let _ = sqlx::query(
                    "UPDATE seamless_pipes SET grade=?1,od=?2,wt=?3,length=?4,weight=?5,\
                     connection_type=?6,heat_number=?7,production_date=?8,\
                     status=?9,location=?10,notes=?11,updated_at=?12 \
                     WHERE id=?13",
                )
                .bind(&grade)
                .bind(od)
                .bind(wt)
                .bind(len)
                .bind(wgt)
                .bind(&conn)
                .bind(&heat)
                .bind(&prod_date)
                .bind(&status)
                .bind(&loc)
                .bind(&notes)
                .bind(&now)
                .bind(&existing_id)
                .execute(&self.pool)
                .await
                .map_err(AppError::from)?;
                success += 1;
                continue;
            }

            // Insert new
            let id = Uuid::new_v4().to_string();
            let result = sqlx::query(
                "INSERT INTO seamless_pipes \
                 (id,pipe_number,grade,od,wt,length,weight,connection_type,\
                  heat_number,production_date,status,location,notes,created_at,updated_at) \
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",
            )
            .bind(&id)
            .bind(&pipe_number)
            .bind(&grade)
            .bind(od)
            .bind(wt)
            .bind(len)
            .bind(wgt)
            .bind(&conn)
            .bind(&heat)
            .bind(&prod_date)
            .bind(&status)
            .bind(&loc)
            .bind(&notes)
            .bind(&now)
            .bind(&now)
            .execute(&self.pool)
            .await;

            match result {
                Ok(_) => success += 1,
                Err(e) => {
                    fail += 1;
                    errors.push(RowError {
                        row: row_num.parse().unwrap_or(0),
                        reason: format!("DB error: {}", e),
                        raw_data: raw,
                    });
                }
            }
        }

        Ok(ImportResult {
            success_count: success,
            fail_count: fail,
            errors,
        })
    }

    async fn batch_insert_screen(
        &self,
        rows: Vec<HashMap<String, String>>,
        config: &ImportConfig,
    ) -> AppResult<ImportResult> {
        let mut success = 0i32;
        let mut fail = 0i32;
        let mut errors = Vec::new();

        for rec in rows {
            let row_num = rec.get("_row").cloned().unwrap_or_default();
            let raw = format!("{:?}", rec);

            let pipe_number = match rec.get("pipe_number").filter(|v| !v.trim().is_empty()) {
                Some(n) => n.trim().to_string(),
                None => {
                    fail += 1;
                    errors.push(RowError {
                        row: row_num.parse().unwrap_or(0),
                        reason: "Missing pipe_number".into(),
                        raw_data: raw,
                    });
                    continue;
                }
            };
            let grade = match rec.get("grade").filter(|v| !v.trim().is_empty()) {
                Some(v) => v.trim().to_string(),
                None => {
                    fail += 1;
                    errors.push(RowError {
                        row: row_num.parse().unwrap_or(0),
                        reason: "Missing grade".into(),
                        raw_data: raw,
                    });
                    continue;
                }
            };

            let od = rec.get("od").and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
            let wt = rec.get("wt").and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
            let len = rec.get("length").and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
            let wgt = rec.get("weight").and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);

            let screen_type = rec.get("screen_type").cloned().unwrap_or_default();
            let slot_w = rec.get("slot_width").and_then(|v| v.parse::<f64>().ok());
            let open_a = rec.get("open_area").and_then(|v| v.parse::<f64>().ok());

            let conn = rec
                .get("connection_type")
                .and_then(|v| if v.is_empty() { None } else { Some(v.clone()) });
            let heat = rec
                .get("heat_number")
                .and_then(|v| if v.is_empty() { None } else { Some(v.clone()) });
            let prod_date = rec
                .get("production_date")
                .and_then(|v| if v.is_empty() { None } else { Some(v.clone()) });
            let status = rec
                .get("status")
                .filter(|v| !v.is_empty())
                .cloned()
                .unwrap_or_else(|| "in_stock".to_string());
            let loc = rec
                .get("location")
                .and_then(|v| if v.is_empty() { None } else { Some(v.clone()) });
            let notes = rec
                .get("notes")
                .and_then(|v| if v.is_empty() { None } else { Some(v.clone()) });

            // Check duplicate
            let existing: Option<(String,)> =
                sqlx::query_as("SELECT id FROM screen_pipes WHERE pipe_number = ?1 AND deleted_at IS NULL")
                    .bind(&pipe_number)
                    .fetch_optional(&self.pool)
                    .await
                    .map_err(AppError::from)?;

            let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

            if let Some((existing_id,)) = existing {
                if config.on_duplicate == DuplicateStrategy::Skip {
                    fail += 1;
                    errors.push(RowError {
                        row: row_num.parse().unwrap_or(0),
                        reason: format!("Duplicate pipe_number: {}", pipe_number),
                        raw_data: raw,
                    });
                    continue;
                }
                // Overwrite
                let _ = sqlx::query(
                    "UPDATE screen_pipes SET grade=?1,od=?2,wt=?3,length=?4,weight=?5,\
                     screen_type=?6,slot_width=?7,open_area=?8,\
                     connection_type=?9,heat_number=?10,production_date=?11,\
                     status=?12,location=?13,notes=?14,updated_at=?15 \
                     WHERE id=?16",
                )
                .bind(&grade)
                .bind(od)
                .bind(wt)
                .bind(len)
                .bind(wgt)
                .bind(&screen_type)
                .bind(slot_w)
                .bind(open_a)
                .bind(&conn)
                .bind(&heat)
                .bind(&prod_date)
                .bind(&status)
                .bind(&loc)
                .bind(&notes)
                .bind(&now)
                .bind(&existing_id)
                .execute(&self.pool)
                .await
                .map_err(AppError::from)?;
                success += 1;
                continue;
            }

            // Insert new
            let id = Uuid::new_v4().to_string();
            let result = sqlx::query(
                "INSERT INTO screen_pipes \
                 (id,pipe_number,grade,od,wt,length,weight,screen_type,\
                  slot_width,open_area,connection_type,heat_number,\
                  production_date,status,location,notes,created_at,updated_at) \
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)",
            )
            .bind(&id)
            .bind(&pipe_number)
            .bind(&grade)
            .bind(od)
            .bind(wt)
            .bind(len)
            .bind(wgt)
            .bind(&screen_type)
            .bind(slot_w)
            .bind(open_a)
            .bind(&conn)
            .bind(&heat)
            .bind(&prod_date)
            .bind(&status)
            .bind(&loc)
            .bind(&notes)
            .bind(&now)
            .bind(&now)
            .execute(&self.pool)
            .await;

            match result {
                Ok(_) => success += 1,
                Err(e) => {
                    fail += 1;
                    errors.push(RowError {
                        row: row_num.parse().unwrap_or(0),
                        reason: format!("DB error: {}", e),
                        raw_data: raw,
                    });
                }
            }
        }

        Ok(ImportResult {
            success_count: success,
            fail_count: fail,
            errors,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Private — query helpers (tuple-based for direct SQL mapping)
    // ═══════════════════════════════════════════════════════════════════════

    async fn query_stock(
        &self,
        filter: &ExportFilter,
    ) -> AppResult<
        Vec<(
            String, String, String, String, String, String, f64, f64, f64, f64, Option<String>,
        )>,
    > {
        let grade = filter.grade.as_deref().unwrap_or("");
        let status = filter.status.as_deref().unwrap_or("");
        let location = filter.location.as_deref().unwrap_or("");
        let pipe_type = filter.pipe_type.as_deref().unwrap_or("");

        let rows = sqlx::query_as::<_, (
            String, String, String, String, String, String, f64, f64, f64, f64, Option<String>,
        )>(
            "SELECT pipeline_id, pipe_type, pipe_number, grade, status, location, \
                    od, wt, length, weight, notes \
             FROM stock_summary \
             WHERE (?1 = '' OR grade = ?1) \
               AND (?2 = '' OR status = ?2) \
               AND (?3 = '' OR location = ?3) \
               AND (?4 = '' OR ?4 = 'all' OR pipe_type = ?4)",
        )
        .bind(grade)
        .bind(status)
        .bind(location)
        .bind(pipe_type)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows)
    }

    async fn query_inbound(
        &self,
        filter: &ExportFilter,
    ) -> AppResult<
        Vec<(
            String, String, Option<String>, Option<String>, String, i32, Option<String>, String,
        )>,
    > {
        let it = filter.inbound_type.as_deref().unwrap_or("");
        let sd = filter.start_date.as_deref().unwrap_or("");
        let ed = filter.end_date.as_deref().unwrap_or("");

        let rows = sqlx::query_as::<_, (
            String, String, Option<String>, Option<String>, String, i32, Option<String>, String,
        )>(
            "SELECT inbound_no, inbound_type, supplier_id, order_id, operator_id, \
                    total_items, notes, created_at \
             FROM inbound_records \
             WHERE (?1 = '' OR inbound_type = ?1) \
               AND (?2 = '' OR created_at >= ?2) \
               AND (?3 = '' OR created_at <= ?3) \
             ORDER BY created_at DESC",
        )
        .bind(it)
        .bind(sd)
        .bind(ed)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows)
    }

    async fn query_outbound(
        &self,
        filter: &ExportFilter,
    ) -> AppResult<
        Vec<(
            String, String, Option<String>, Option<String>, String, i32, Option<String>, String,
        )>,
    > {
        let ot = filter.outbound_type.as_deref().unwrap_or("");
        let sd = filter.start_date.as_deref().unwrap_or("");
        let ed = filter.end_date.as_deref().unwrap_or("");

        let rows = sqlx::query_as::<_, (
            String, String, Option<String>, Option<String>, String, i32, Option<String>, String,
        )>(
            "SELECT outbound_no, outbound_type, customer_id, order_id, operator_id, \
                    total_items, notes, created_at \
             FROM outbound_records \
             WHERE (?1 = '' OR outbound_type = ?1) \
               AND (?2 = '' OR created_at >= ?2) \
               AND (?3 = '' OR created_at <= ?3) \
             ORDER BY created_at DESC",
        )
        .bind(ot)
        .bind(sd)
        .bind(ed)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows)
    }

    async fn query_seamless(
        &self,
        filter: &ExportFilter,
    ) -> AppResult<
        Vec<(
            String, String, f64, f64, f64, f64, Option<String>, Option<String>, Option<String>,
            String, Option<String>, Option<String>,
        )>,
    > {
        let grade = filter.grade.as_deref().unwrap_or("");
        let status = filter.status.as_deref().unwrap_or("");
        let search = filter.search.as_deref().unwrap_or("");

        let rows = sqlx::query_as::<_, (
            String, String, f64, f64, f64, f64, Option<String>, Option<String>, Option<String>,
            String, Option<String>, Option<String>,
        )>(
            "SELECT pipe_number, grade, od, wt, length, weight, \
                    connection_type, heat_number, production_date, \
                    status, location, notes \
             FROM seamless_pipes \
             WHERE deleted_at IS NULL \
               AND (?1 = '' OR grade = ?1) \
               AND (?2 = '' OR status = ?2) \
               AND (?3 = '' OR (pipe_number LIKE '%' || ?3 || '%' OR heat_number LIKE '%' || ?3 || '%')) \
             ORDER BY created_at DESC",
        )
        .bind(grade)
        .bind(status)
        .bind(search)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows)
    }

    async fn query_screen(
        &self,
        filter: &ExportFilter,
    ) -> AppResult<
        Vec<(
            String, String, f64, f64, f64, f64, String, Option<f64>, Option<f64>,
            Option<String>, Option<String>, Option<String>, String, Option<String>, Option<String>,
        )>,
    > {
        let grade = filter.grade.as_deref().unwrap_or("");
        let status = filter.status.as_deref().unwrap_or("");
        let search = filter.search.as_deref().unwrap_or("");

        let rows = sqlx::query_as::<_, (
            String, String, f64, f64, f64, f64, String, Option<f64>, Option<f64>,
            Option<String>, Option<String>, Option<String>, String, Option<String>, Option<String>,
        )>(
            "SELECT pipe_number, grade, od, wt, length, weight, screen_type, \
                    slot_width, open_area, connection_type, heat_number, \
                    production_date, status, location, notes \
             FROM screen_pipes \
             WHERE deleted_at IS NULL \
               AND (?1 = '' OR grade = ?1) \
               AND (?2 = '' OR status = ?2) \
               AND (?3 = '' OR (pipe_number LIKE '%' || ?3 || '%' OR heat_number LIKE '%' || ?3 || '%')) \
             ORDER BY created_at DESC",
        )
        .bind(grade)
        .bind(status)
        .bind(search)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Private — file writers
    // ═══════════════════════════════════════════════════════════════════════

    fn write_file(&self, headers: &[&str], data: &[Vec<String>], fmt: &str) -> AppResult<Vec<u8>> {
        match fmt {
            "csv" => self.write_csv(headers, data),
            _ => self.write_xlsx(headers, data),
        }
    }

    fn write_xlsx(&self, headers: &[&str], data: &[Vec<String>]) -> AppResult<Vec<u8>> {
        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet();

        let hdr_fmt = Format::new()
            .set_bold()
            .set_border(FormatBorder::Thin)
            .set_background_color(Color::RGB(0x4472C4))
            .set_font_color(Color::White);

        for (col, h) in headers.iter().enumerate() {
            sheet
                .write_string_with_format(0, col as u16, *h, &hdr_fmt)
                .map_err(|e| AppError::Internal(format!("Excel write: {}", e)))?;
        }

        for (ri, row) in data.iter().enumerate() {
            for (ci, val) in row.iter().enumerate() {
                sheet
                    .write_string_with_format((ri + 1) as u32, ci as u16, val.as_str(), &Format::new())
                    .map_err(|e| AppError::Internal(format!("Excel write: {}", e)))?;
            }
        }

        for col in 0..headers.len() as u16 {
            sheet
                .set_column_width(col, 18)
                .map_err(|e| AppError::Internal(format!("Excel width: {}", e)))?;
        }

        workbook
            .save_to_buffer()
            .map_err(|e| AppError::Internal(format!("Excel save: {}", e)))
    }

    fn write_csv(&self, headers: &[&str], data: &[Vec<String>]) -> AppResult<Vec<u8>> {
        let mut buf = Vec::new();
        {
            let mut writer = CsvWriter::from_writer(Cursor::new(&mut buf));
            writer
                .write_record(headers)
                .map_err(|e| AppError::Internal(format!("CSV write: {}", e)))?;
            for row in data {
                writer
                    .write_record(row.iter().map(|s| s.as_str()))
                    .map_err(|e| AppError::Internal(format!("CSV write: {}", e)))?;
            }
            writer
                .flush()
                .map_err(|e| AppError::Internal(format!("CSV flush: {}", e)))?;
        }

        // UTF-8 BOM for Excel compatibility with Chinese characters
        let mut result = vec![0xEFu8, 0xBB, 0xBF];
        result.append(&mut buf);
        Ok(result)
    }
}
