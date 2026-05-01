use serde::{Deserialize, Serialize};
use crate::error::{AppError, Result as AppResult};

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
    pub furnace_number: Option<String>,
    pub heat_treatment_batch: Option<String>,
    pub sample_number: Option<String>,
    pub production_count: Option<i32>,
    pub material_rack: Option<String>,
    pub remarks: Option<String>,
}

impl SteelPipe {
    pub fn validate(&self) -> AppResult<()> {
        if self.pipe_id.trim().is_empty() {
            return Err(AppError::Validation("钢管编号不能为空".to_string()));
        }
        if self.diameter <= 0.0 || self.diameter > 10000.0 {
            return Err(AppError::Validation("直径必须在0-10000mm之间".to_string()));
        }
        if self.thickness <= 0.0 || self.thickness > 500.0 {
            return Err(AppError::Validation("壁厚必须在0-500mm之间".to_string()));
        }
        if self.length <= 0.0 || self.length > 1000.0 {
            return Err(AppError::Validation("长度必须在0-1000m之间".to_string()));
        }
        if self.material.trim().is_empty() {
            return Err(AppError::Validation("材质不能为空".to_string()));
        }
        if self.quantity <= 0 {
            return Err(AppError::Validation("数量必须大于0".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_pipe() -> SteelPipe {
        SteelPipe {
            id: None,
            pipe_id: "P001".to_string(),
            diameter: 20.0,
            thickness: 2.0,
            length: 6.0,
            material: "Stainless".to_string(),
            quantity: 10,
            location: None,
            supplier: None,
            entry_date: "2024-01-01".to_string(),
            last_update: None,
            status: "在库".to_string(),
            furnace_number: None,
            heat_treatment_batch: None,
            sample_number: None,
            production_count: None,
            material_rack: None,
            remarks: None,
        }
    }

    #[test]
    fn test_validate_valid() {
        let pipe = mock_pipe();
        assert!(pipe.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_id() {
        let mut pipe = mock_pipe();
        pipe.pipe_id = "  ".to_string();
        let res = pipe.validate();
        assert!(matches!(res, Err(AppError::Validation(m)) if m == "钢管编号不能为空"));
    }

    #[test]
    fn test_validate_invalid_diameter() {
        let mut pipe = mock_pipe();
        pipe.diameter = -1.0;
        let res = pipe.validate();
        assert!(matches!(res, Err(AppError::Validation(m)) if m == "直径必须在0-10000mm之间"));
    }

    #[test]
    fn test_validate_invalid_thickness() {
        let mut pipe = mock_pipe();
        pipe.thickness = 0.0;
        let res = pipe.validate();
        assert!(matches!(res, Err(AppError::Validation(m)) if m == "壁厚必须在0-500mm之间"));
    }

    #[test]
    fn test_validate_invalid_length() {
        let mut pipe = mock_pipe();
        pipe.length = 2000.0;
        let res = pipe.validate();
        assert!(matches!(res, Err(AppError::Validation(m)) if m == "长度必须在0-1000m之间"));
    }

    #[test]
    fn test_validate_empty_material() {
        let mut pipe = mock_pipe();
        pipe.material = "".to_string();
        let res = pipe.validate();
        assert!(matches!(res, Err(AppError::Validation(m)) if m == "材质不能为空"));
    }

    #[test]
    fn test_validate_invalid_quantity() {
        let mut pipe = mock_pipe();
        pipe.quantity = 0;
        let res = pipe.validate();
        assert!(matches!(res, Err(AppError::Validation(m)) if m == "数量必须大于0"));
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Production {
    pub id: Option<i64>,
    pub furnace_number: String,
    pub heat_treatment_batch: Option<String>,
    pub material_batch: Option<String>,
    pub production_count: i32,
    pub sample: Option<String>,
    pub supplier: Option<String>,
    pub operator: String,
    pub production_date: String,
    pub remarks: Option<String>,
}

#[derive(Deserialize)]
pub struct PipeQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub material: Option<String>,
    pub status: Option<String>,
    pub min_diameter: Option<f64>,
    pub max_diameter: Option<f64>,
    pub min_length: Option<f64>,
    pub max_length: Option<f64>,
}

#[derive(Deserialize)]
pub struct RecordQuery {
    pub pipe_id: Option<String>,
    pub operation_type: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Deserialize)]
pub struct BatchDeleteRequest {
    pub pipe_ids: Vec<String>,
    pub operator: String,
}

#[derive(Deserialize)]
pub struct EntryRequest {
    pub pipe: SteelPipe,
    pub operator: String,
    pub remarks: Option<String>,
}

#[derive(Deserialize)]
pub struct ExitRequest {
    pub pipe_id: String,
    pub quantity: i32,
    pub operator: String,
    pub remarks: Option<String>,
}

#[derive(Deserialize)]
pub struct ImportRequest {
    pub csv_content: String,
    pub operator: String,
}

#[derive(Deserialize)]
pub struct ExcelImportRequest {
    pub excel_base64: String,
    pub operator: String,
}

#[derive(Deserialize)]
pub struct SaveRequest {
    pub path: Option<String>,
}

#[derive(Deserialize)]
pub struct RestoreRequest {
    pub backup_path: String,
}

#[derive(Deserialize)]
pub struct ReportRequest {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub report_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionRequest {
    pub furnace_number: String,
    pub heat_treatment_batch: Option<String>,
    pub material_batch: Option<String>,
    pub production_count: i32,
    pub sample: Option<String>,
    pub supplier: Option<String>,
    pub operator: String,
    pub remarks: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryTrend {
    pub date: String,
    pub entry_count: i32,
    pub exit_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataDictionaries {
    pub materials: Vec<String>,
    pub locations: Vec<String>,
    pub statuses: Vec<String>,
}
