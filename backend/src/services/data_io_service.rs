use std::collections::HashMap;
use std::io::Cursor;

use calamine::{open_workbook_from_rs, Data, Reader, Xlsx};
use csv::Trim;
use rust_xlsxwriter::*;
use sqlx::SqlitePool;

use crate::dto::common::PaginationParams;
use crate::dto::data_io_dto::*;
use crate::error::AppError;
use crate::repositories::data_io_repo::DataIORepo;
use crate::repositories::operation_log_repo::{CreateOperationLog, OperationLog, OperationLogFilter, OperationLogRepo};

pub struct DataIOService;

impl DataIOService {
    fn get_columns(entity_type: &str) -> Vec<(&'static str, &'static str)> {
        match entity_type {
            ENTITY_SEAMLESS_PIPES => vec![
                ("pipe_number", "Pipe Number"),
                ("batch_number", "Batch Number"),
                ("pipe_type", "Pipe Type"),
                ("grade", "Grade"),
                ("od", "OD (mm)"),
                ("wt", "WT (mm)"),
                ("length", "Length (m)"),
                ("weight_per_unit", "Weight/Unit (kg)"),
                ("end_type", "End Type"),
                ("coupling_type", "Coupling Type"),
                ("coupling_od", "Coupling OD (mm)"),
                ("coupling_length", "Coupling Length (mm)"),
                ("heat_number", "Heat Number"),
                ("serial_number", "Serial Number"),
                ("manufacturer", "Manufacturer"),
                ("production_date", "Production Date"),
                ("cert_number", "Cert Number"),
                ("status", "Status"),
                ("notes", "Notes"),
            ],
            ENTITY_SCREEN_PIPES => vec![
                ("pipe_number", "Pipe Number"),
                ("batch_number", "Batch Number"),
                ("screen_type", "Screen Type"),
                ("slot_size", "Slot Size (mm)"),
                ("filtration_grade", "Filtration Grade"),
                ("base_od", "Base OD (mm)"),
                ("base_wt", "Base WT (mm)"),
                ("base_grade", "Base Grade"),
                ("base_end_type", "Base End Type"),
                ("length", "Length (m)"),
                ("weight_per_unit", "Weight/Unit (kg)"),
                ("heat_number", "Heat Number"),
                ("serial_number", "Serial Number"),
                ("manufacturer", "Manufacturer"),
                ("production_date", "Production Date"),
                ("cert_number", "Cert Number"),
                ("status", "Status"),
                ("notes", "Notes"),
            ],
            ENTITY_INVENTORY => vec![
                ("pipe_id", "Pipe ID"),
                ("pipe_number", "Pipe Number"),
                ("pipe_type", "Pipe Type"),
                ("grade", "Grade"),
                ("od", "OD (mm)"),
                ("wt", "WT (mm)"),
                ("status", "Status"),
                ("location_id", "Location ID"),
                ("heat_number", "Heat Number"),
            ],
            ENTITY_PURCHASE_ORDERS => vec![
                ("order_no", "Order No"),
                ("supplier_id", "Supplier ID"),
                ("order_date", "Order Date"),
                ("status", "Status"),
                ("total_amount", "Total Amount"),
                ("items_count", "Items Count"),
                ("notes", "Notes"),
            ],
            ENTITY_SALES_ORDERS => vec![
                ("order_no", "Order No"),
                ("customer_id", "Customer ID"),
                ("order_date", "Order Date"),
                ("status", "Status"),
                ("total_amount", "Total Amount"),
                ("items_count", "Items Count"),
                ("notes", "Notes"),
            ],
            ENTITY_QUALITY_CERTS => vec![
                ("cert_number", "Cert Number"),
                ("pipe_type", "Pipe Type"),
                ("pipe_id", "Pipe ID"),
                ("cert_date", "Cert Date"),
                ("result", "Result"),
                ("inspector", "Inspector"),
                ("inspection_body", "Inspection Body"),
                ("notes", "Notes"),
            ],
            _ => vec![],
        }
    }

    fn get_field_names(entity_type: &str) -> Vec<String> {
        Self::get_columns(entity_type)
            .into_iter()
            .map(|(f, _)| f.to_string())
            .collect()
    }

    fn get_header_labels(entity_type: &str) -> Vec<String> {
        Self::get_columns(entity_type)
            .into_iter()
            .map(|(_, l)| l.to_string())
            .collect()
    }

    async fn fetch_export_data(
        pool: &SqlitePool,
        entity_type: &str,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        match entity_type {
            ENTITY_SEAMLESS_PIPES => DataIORepo::export_seamless_pipes(pool).await,
            ENTITY_SCREEN_PIPES => DataIORepo::export_screen_pipes(pool).await,
            ENTITY_INVENTORY => DataIORepo::export_inventory(pool).await,
            ENTITY_PURCHASE_ORDERS => DataIORepo::export_purchase_orders(pool).await,
            ENTITY_SALES_ORDERS => DataIORepo::export_sales_orders(pool).await,
            ENTITY_QUALITY_CERTS => DataIORepo::export_quality_certs(pool).await,
            _ => Err(AppError::Validation(format!(
                "Unknown entity type: {}",
                entity_type
            ))),
        }
    }

    fn rows_to_xlsx(
        field_names: &[String],
        headers: &[String],
        rows: &[serde_json::Value],
    ) -> Result<Vec<u8>, AppError> {
        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet();

        let bold = Format::new().set_bold();

        for (col, label) in headers.iter().enumerate() {
            sheet
                .write_string_with_format(0, col as u16, label, &bold)
                .map_err(|e| AppError::ExportError(format!("Write header: {}", e)))?;
        }

        for (row_idx, row) in rows.iter().enumerate() {
            for (col, field) in field_names.iter().enumerate() {
                let value = row.get(field);
                match value {
                    Some(serde_json::Value::Null) | None => {}
                    Some(serde_json::Value::String(s)) => {
                        sheet
                            .write_string(row_idx as u32 + 1, col as u16, s)
                            .map_err(|e| AppError::ExportError(format!("Write string: {}", e)))?;
                    }
                    Some(serde_json::Value::Number(n)) => {
                        if let Some(f) = n.as_f64() {
                            sheet
                                .write_number(row_idx as u32 + 1, col as u16, f)
                                .map_err(|e| AppError::ExportError(format!("Write number: {}", e)))?;
                        } else if let Some(i) = n.as_i64() {
                            sheet
                                .write_number(row_idx as u32 + 1, col as u16, i as f64)
                                .map_err(|e| AppError::ExportError(format!("Write int: {}", e)))?;
                        }
                    }
                    Some(serde_json::Value::Bool(b)) => {
                        sheet
                            .write_boolean(row_idx as u32 + 1, col as u16, *b)
                            .map_err(|e| AppError::ExportError(format!("Write bool: {}", e)))?;
                    }
                    _ => {}
                }
            }
        }

        workbook
            .save_to_buffer()
            .map_err(|e| AppError::ExportError(format!("Save xlsx: {}", e)))
    }

    fn rows_to_csv(
        field_names: &[String],
        rows: &[serde_json::Value],
    ) -> Result<Vec<u8>, AppError> {
        let mut writer = csv::Writer::from_writer(Vec::new());

        writer
            .write_record(field_names)
            .map_err(|e| AppError::ExportError(format!("Write CSV header: {}", e)))?;

        for row in rows {
            let mut record = Vec::new();
            for field in field_names {
                let value = match row.get(field) {
                    Some(serde_json::Value::Null) | None => String::new(),
                    Some(serde_json::Value::String(s)) => s.clone(),
                    Some(serde_json::Value::Number(n)) => n.to_string(),
                    Some(serde_json::Value::Bool(b)) => b.to_string(),
                    _ => String::new(),
                };
                record.push(value);
            }
            writer
                .write_record(&record)
                .map_err(|e| AppError::ExportError(format!("Write CSV row: {}", e)))?;
        }

        writer
            .into_inner()
            .map_err(|e| AppError::ExportError(format!("Finalize CSV: {}", e)))
    }

    fn parse_xlsx_rows(
        data: &[u8],
        field_names: &[String],
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let cursor = Cursor::new(data);
        let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)
            .map_err(|e| AppError::ImportError(format!("Open xlsx: {}", e)))?;

        let sheet_name = workbook
            .sheet_names()
            .first()
            .cloned()
            .ok_or_else(|| AppError::ImportError("No sheets found in xlsx".into()))?;

        let range = workbook
            .worksheet_range(&sheet_name)
            .map_err(|e| AppError::ImportError(format!("Read sheet: {}", e)))?;

        let mut rows = Vec::new();
        let mut iter = range.rows();

        let header_row = match iter.next() {
            Some(r) => r,
            None => return Ok(rows),
        };

        let col_map: HashMap<usize, usize> = field_names
            .iter()
            .enumerate()
            .filter_map(|(idx, name)| {
                header_row.iter().position(|h| {
                    let h_str = h.to_string().trim().to_lowercase();
                    h_str == name.to_lowercase() || h_str == name.replace("_", " ").to_lowercase()
                }).map(|pos| (pos, idx))
            })
            .collect();

        for row in iter {
            let mut obj = serde_json::Map::new();
            for (col_idx, cell) in row.iter().enumerate() {
                if let Some(&field_idx) = col_map.get(&col_idx) {
                    let field_name = &field_names[field_idx];
                    let value: serde_json::Value = match cell {
                        Data::String(s) => serde_json::Value::String(s.clone()),
                        Data::Float(f) => {
                            serde_json::Number::from_f64(*f)
                                .map(serde_json::Value::Number)
                                .unwrap_or(serde_json::Value::Null)
                        }
                        Data::Int(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
                        Data::Bool(b) => serde_json::Value::Bool(*b),
                        Data::DateTime(_) => serde_json::Value::String(cell.to_string()),
                        _ => serde_json::Value::Null,
                    };
                    obj.insert(field_name.clone(), value);
                }
            }
            rows.push(serde_json::Value::Object(obj));
        }

        Ok(rows)
    }

    fn parse_csv_rows(
        data: &[u8],
        field_names: &[String],
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let mut reader = csv::ReaderBuilder::new()
            .trim(Trim::All)
            .flexible(true)
            .from_reader(data);

        let headers: Vec<String> = reader
            .headers()
            .map_err(|e| AppError::ImportError(format!("Read CSV headers: {}", e)))?
            .iter()
            .map(|h| h.trim().to_lowercase())
            .collect();

        let col_map: HashMap<usize, usize> = field_names
            .iter()
            .enumerate()
            .filter_map(|(idx, name)| {
                headers
                    .iter()
                    .position(|h| *h == name.to_lowercase() || *h == name.replace("_", " ").to_lowercase())
                    .map(|pos| (pos, idx))
            })
            .collect();

        let mut rows = Vec::new();
        for result in reader.records() {
            let record = result.map_err(|e| AppError::ImportError(format!("Read CSV row: {}", e)))?;
            let mut obj = serde_json::Map::new();
            for (col_idx, field) in record.iter().enumerate() {
                if let Some(&field_idx) = col_map.get(&col_idx) {
                    let field_name = &field_names[field_idx];
                    let trimmed = field.trim();
                    if !trimmed.is_empty() {
                        if let Ok(n) = trimmed.parse::<f64>() {
                            if let Some(num) = serde_json::Number::from_f64(n) {
                                obj.insert(field_name.clone(), serde_json::Value::Number(num));
                            } else {
                                obj.insert(field_name.clone(), serde_json::Value::String(trimmed.to_string()));
                            }
                        } else {
                            obj.insert(
                                field_name.clone(),
                                serde_json::Value::String(trimmed.to_string()),
                            );
                        }
                    }
                }
            }
            rows.push(serde_json::Value::Object(obj));
        }

        Ok(rows)
    }

    fn validate_entity(entity_type: &str) -> Result<(), AppError> {
        if VALID_ENTITY_TYPES.contains(&entity_type) {
            Ok(())
        } else {
            Err(AppError::Validation(format!(
                "Invalid entity type: {}. Valid types: {:?}",
                entity_type, VALID_ENTITY_TYPES
            )))
        }
    }

    pub async fn export_entity(
        pool: &SqlitePool,
        entity_type: &str,
        format: &str,
    ) -> Result<Vec<u8>, AppError> {
        Self::validate_entity(entity_type)?;

        let rows = Self::fetch_export_data(pool, entity_type).await?;
        let field_names = Self::get_field_names(entity_type);
        let headers = Self::get_header_labels(entity_type);

        match format {
            "csv" => Self::rows_to_csv(&field_names, &rows),
            _ => Self::rows_to_xlsx(&field_names, &headers, &rows),
        }
    }

    pub async fn download_template(
        entity_type: &str,
        format: &str,
    ) -> Result<Vec<u8>, AppError> {
        Self::validate_entity(entity_type)?;

        let field_names = Self::get_field_names(entity_type);
        let headers = Self::get_header_labels(entity_type);
        let rows: Vec<serde_json::Value> = Vec::new();

        match format {
            "csv" => Self::rows_to_csv(&field_names, &rows),
            _ => Self::rows_to_xlsx(&field_names, &headers, &rows),
        }
    }

    pub async fn import_entity(
        pool: &SqlitePool,
        entity_type: &str,
        data: &[u8],
        file_name: &str,
    ) -> Result<ImportResult, AppError> {
        Self::validate_entity(entity_type)?;

        let field_names = Self::get_field_names(entity_type);

        let rows = if file_name.ends_with(".csv") {
            Self::parse_csv_rows(data, &field_names)?
        } else {
            Self::parse_xlsx_rows(data, &field_names)?
        };

        if rows.is_empty() {
            return Err(AppError::ImportError("No data rows found in file".into()));
        }

        let (imported_count, errors) = match entity_type {
            ENTITY_SEAMLESS_PIPES => {
                DataIORepo::import_seamless_pipes(pool, &rows).await?
            }
            ENTITY_SCREEN_PIPES => {
                DataIORepo::import_screen_pipes(pool, &rows).await?
            }
            _ => {
                return Err(AppError::ImportError(format!(
                    "Import not supported for entity type: {}",
                    entity_type
                )))
            }
        };

        Ok(ImportResult {
            entity_type: entity_type.to_string(),
            imported_count,
            failed_count: errors.len() as u64,
            errors,
        })
    }

    pub async fn list_operation_logs(
        pool: &SqlitePool,
        query: &OperationLogQuery,
    ) -> Result<(Vec<OperationLog>, u64), AppError> {
        let params = PaginationParams {
            page: query.page,
            page_size: query.page_size,
            sort_by: None,
            sort_order: None,
        };
        let filter = OperationLogFilter {
            user_id: query.user_id,
            username: None,
            action: query.action.clone(),
            entity_type: query.entity_type.clone(),
            entity_id: None,
        };

        let (logs, total) = OperationLogRepo::list(pool, &params, &filter)
            .await
            .map_err(AppError::from)?;

        Ok((logs, total))
    }

    pub async fn log_operation(
        pool: &SqlitePool,
        user_id: Option<i64>,
        username: Option<String>,
        action: &str,
        entity_type: &str,
        entity_id: Option<i64>,
        details: Option<String>,
        ip_address: Option<String>,
    ) -> Result<(), AppError> {
        let log = CreateOperationLog {
            user_id,
            username,
            action: action.to_string(),
            entity_type: entity_type.to_string(),
            entity_id,
            details,
            ip_address,
        };
        OperationLogRepo::create(pool, &log)
            .await
            .map_err(AppError::from)?;
        Ok(())
    }

    pub fn content_type(format: &str) -> &'static str {
        match format {
            "csv" => "text/csv; charset=utf-8",
            _ => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        }
    }

    pub fn file_extension(format: &str) -> &'static str {
        match format {
            "csv" => "csv",
            _ => "xlsx",
        }
    }
}
