//! CSV 批量导入：流式逐行、行级部分成功、行数上限 10 000。
//!
//! 与旧实现对齐（设计 §0.4 D7 / §17）：
//! - 每行独立事务：失败行不影响已成功行（非整体回滚）
//! - 合规门禁与单根入库完全一致（含类型扩展表）
//! - 超过行数上限整体拒绝并提示分批（返回 400，不写任何行）

use std::collections::BTreeMap;

use steelpipe_domain::{QualityMark, TubeType};
use steelpipe_infra::DbPool;

use crate::auth::Identity;
use crate::inventory::{
    InventoryError, InventoryService, ScreenPipeInput, SeamlessPipeInput, StockInInput,
};

/// 单次导入行数上限（旧实现 `MAX_IMPORT_ROWS`）。
pub const MAX_IMPORT_ROWS: usize = 10_000;

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("CSV 缺少表头列：{0}")]
    MissingColumns(String),
    #[error("行数 {0} 超过单次上限 {MAX_IMPORT_ROWS}，请分批导入")]
    TooManyRows(usize),
    #[error("CSV 解析失败：{0}")]
    Parse(String),
    #[error(transparent)]
    Inventory(#[from] InventoryError),
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RowFailure {
    pub row: usize,
    pub pipe_id: String,
    pub error: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ImportReport {
    pub total: usize,
    pub inserted: usize,
    pub failed: Vec<RowFailure>,
}

impl ImportReport {
    pub fn failed_count(&self) -> usize {
        self.failed.len()
    }
}

const REQUIRED: [&str; 8] = [
    "pipe_id",
    "pipe_type",
    "outer_diameter_mm",
    "wall_thickness_mm",
    "length_m",
    "steel_grade",
    "manufacturer",
    "location",
];

pub struct ImportService {
    inventory: InventoryService,
}

impl ImportService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            inventory: InventoryService::new(pool),
        }
    }

    /// 解析并导入。返回逐行结果；`failed` 空表示全部成功。
    pub async fn import_csv(
        &self,
        actor: &Identity,
        csv: &str,
    ) -> Result<ImportReport, ImportError> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .trim(csv::Trim::All)
            .from_reader(strip_bom(csv).as_bytes());

        let headers = reader
            .headers()
            .map_err(|error| ImportError::Parse(error.to_string()))?
            .clone();
        let index: BTreeMap<String, usize> = headers
            .iter()
            .enumerate()
            .map(|(position, name)| (name.trim().to_ascii_lowercase(), position))
            .collect();

        let missing: Vec<&str> = REQUIRED
            .iter()
            .copied()
            .filter(|column| !index.contains_key(*column))
            .collect();
        if !missing.is_empty() {
            return Err(ImportError::MissingColumns(missing.join(", ")));
        }

        // 先读进内存以检查行数上限（10k 行 × 数百字节是可接受的峰值）
        let records: Vec<csv::StringRecord> = reader
            .records()
            .collect::<Result<_, _>>()
            .map_err(|error| ImportError::Parse(error.to_string()))?;
        if records.len() > MAX_IMPORT_ROWS {
            return Err(ImportError::TooManyRows(records.len()));
        }

        let total = records.len();
        let mut inserted = 0usize;
        let mut failed = Vec::new();

        for (offset, record) in records.iter().enumerate() {
            let row = offset + 2; // 1-based + 表头
            let pipe_id = field(&index, record, "pipe_id").unwrap_or_default();
            match build_input(&index, record) {
                Ok(input) => match self.inventory.stock_in(actor, input).await {
                    Ok(_) => inserted += 1,
                    Err(error) => failed.push(RowFailure {
                        row,
                        pipe_id,
                        error: error.to_string(),
                    }),
                },
                Err(error) => failed.push(RowFailure {
                    row,
                    pipe_id,
                    error,
                }),
            }
        }

        Ok(ImportReport {
            total,
            inserted,
            failed,
        })
    }
}

fn strip_bom(source: &str) -> &str {
    source.strip_prefix('\u{feff}').unwrap_or(source)
}

fn field(
    index: &BTreeMap<String, usize>,
    record: &csv::StringRecord,
    name: &str,
) -> Option<String> {
    let position = *index.get(name)?;
    record
        .get(position)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn number(index: &BTreeMap<String, usize>, record: &csv::StringRecord, name: &str) -> Option<f64> {
    field(index, record, name).and_then(|raw| raw.parse::<f64>().ok())
}

fn build_input(
    index: &BTreeMap<String, usize>,
    record: &csv::StringRecord,
) -> Result<StockInInput, String> {
    let pipe_id = field(index, record, "pipe_id").ok_or_else(|| "pipe_id 为空".to_owned())?;
    let pipe_type_raw =
        field(index, record, "pipe_type").ok_or_else(|| "pipe_type 为空".to_owned())?;
    let pipe_type: TubeType = pipe_type_raw
        .parse()
        .map_err(|_| format!("pipe_type 非法：{pipe_type_raw}"))?;

    let outer_diameter_mm = number(index, record, "outer_diameter_mm")
        .ok_or_else(|| "outer_diameter_mm 非法".to_owned())?;
    let wall_thickness_mm = number(index, record, "wall_thickness_mm")
        .ok_or_else(|| "wall_thickness_mm 非法".to_owned())?;
    let length_m = number(index, record, "length_m").ok_or_else(|| "length_m 非法".to_owned())?;

    let steel_grade =
        field(index, record, "steel_grade").ok_or_else(|| "steel_grade 为空".to_owned())?;
    let manufacturer =
        field(index, record, "manufacturer").ok_or_else(|| "manufacturer 为空".to_owned())?;
    let location = field(index, record, "location").ok_or_else(|| "location 为空".to_owned())?;

    let quality_mark = match field(index, record, "quality_mark") {
        Some(raw) => raw
            .parse::<QualityMark>()
            .map_err(|_| format!("quality_mark 非法：{raw}"))?,
        None => QualityMark::Normal,
    };

    let screen = if pipe_type == TubeType::ScreenPipe {
        let slot_width_mm = number(index, record, "slot_width_mm");
        let filtration_rating_um = number(index, record, "filtration_rating_um");
        match (slot_width_mm, filtration_rating_um) {
            (Some(slot_width_mm), Some(filtration_rating_um)) => Some(ScreenPipeInput {
                screen_type: field(index, record, "screen_type"),
                slot_width_mm,
                slot_density: number(index, record, "slot_density").unwrap_or(0.0),
                filtration_rating_um,
                slot_pattern: field(index, record, "slot_pattern"),
                base_pipe_grade: field(index, record, "base_pipe_grade"),
            }),
            _ => None,
        }
    } else {
        None
    };

    let seamless = if pipe_type == TubeType::SeamlessPipe
        && (index.contains_key("surface_treatment") || index.contains_key("end_protector"))
    {
        Some(SeamlessPipeInput {
            surface_treatment: field(index, record, "surface_treatment"),
            end_protector: field(index, record, "end_protector"),
        })
    } else {
        None
    };

    Ok(StockInInput {
        pipe_id,
        pipe_type,
        outer_diameter_mm,
        wall_thickness_mm,
        length_m,
        steel_grade,
        thread_type: field(index, record, "thread_type"),
        api_5ct_cert: field(index, record, "api_5ct_cert"),
        manufacturer,
        batch_no: field(index, record, "batch_no"),
        heat_no: field(index, record, "heat_no"),
        heat_treat_method: field(index, record, "heat_treat_method"),
        location,
        quality_mark,
        quality_note: field(index, record, "quality_note"),
        screen,
        seamless,
        force: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use steelpipe_auth::hash_password;
    use steelpipe_domain::Role;
    use steelpipe_infra::repo::{audit as audit_repo, locations, standards, tubes, users};
    use steelpipe_infra::{time, uow::Uow};

    struct Fixture {
        pool: DbPool,
        path: std::path::PathBuf,
        service: ImportService,
        actor: Identity,
    }

    async fn fixture(tag: &str) -> Fixture {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("steelpipe-import-{tag}-{nanos}.db"));
        let pool = steelpipe_infra::connect(&format!("sqlite://{}", path.display()), 8)
            .await
            .unwrap();
        steelpipe_infra::migrate(&pool).await.unwrap();
        let mut uow = Uow::begin(&pool).await.unwrap();
        locations::insert(uow.conn(), "A1", None, None)
            .await
            .unwrap();
        standards::insert(
            uow.conn(),
            "L80",
            Some(88.9),
            Some(6.45),
            Some(7.34),
            None,
            None,
        )
        .await
        .unwrap();
        let hash = hash_password("pw-123456").unwrap();
        let id = users::insert(
            uow.conn(),
            "root",
            &hash,
            "Root",
            Role::Admin,
            &time::now_rfc3339(),
        )
        .await
        .unwrap();
        uow.commit().await.unwrap();

        Fixture {
            service: ImportService::new(pool.clone()),
            actor: Identity {
                user_id: id,
                username: "root".into(),
                display_name: "Root".into(),
                role: Role::Admin,
                token_version: 0,
            },
            pool,
            path,
        }
    }

    const HEADER: &str = "pipe_id,pipe_type,outer_diameter_mm,wall_thickness_mm,length_m,steel_grade,manufacturer,location\n";

    #[tokio::test]
    async fn imports_good_rows_and_reports_bad_ones() {
        let fx = fixture("mixed").await;
        let csv = format!(
            "{HEADER}P-1,seamless_pipe,88.9,7.0,9.5,L80,宝钢,A1\n\
             P-2,seamless_pipe,88.9,3.0,9.5,L80,宝钢,A1\n\
             P-3,seamless_pipe,88.9,7.0,9.5,X99,宝钢,A1\n\
             ,seamless_pipe,88.9,7.0,9.5,L80,宝钢,A1\n"
        );
        let report = fx
            .service
            .import_csv(&fx.actor, &csv)
            .await
            .expect("导入失败");
        assert_eq!(report.total, 4);
        assert_eq!(report.inserted, 1);
        assert_eq!(report.failed.len(), 3);
        assert!(report.failed.iter().any(|row| row.error.contains("合规")));
        assert!(
            report
                .failed
                .iter()
                .any(|row| row.error.contains("标准规格"))
        );
        assert!(
            report
                .failed
                .iter()
                .any(|row| row.error.contains("pipe_id"))
        );

        assert!(tubes::find(&fx.pool, "P-1").await.unwrap().is_some());
        assert!(
            tubes::find(&fx.pool, "P-2").await.unwrap().is_none(),
            "失败行不得入库"
        );
        // 成功行各有 1 条审计，失败行 0 条
        assert_eq!(audit_repo::count(&fx.pool).await.unwrap(), 1);

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn rejects_missing_columns_and_oversized_files() {
        let fx = fixture("shape").await;
        let err = fx
            .service
            .import_csv(&fx.actor, "pipe_id,pipe_type\nP-1,seamless_pipe\n")
            .await
            .unwrap_err();
        assert!(
            matches!(err, ImportError::MissingColumns(_)),
            "实际：{err:?}"
        );

        let mut csv = String::from(HEADER);
        for i in 0..MAX_IMPORT_ROWS + 1 {
            csv.push_str(&format!("P-{i},seamless_pipe,88.9,7.0,9.5,L80,宝钢,A1\n"));
        }
        let err = fx.service.import_csv(&fx.actor, &csv).await.unwrap_err();
        assert!(matches!(err, ImportError::TooManyRows(_)), "实际：{err:?}");
        // 超过上限整体拒绝：一行都没写
        assert_eq!(audit_repo::count(&fx.pool).await.unwrap(), 0);

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn handles_bom_and_optional_columns() {
        let fx = fixture("bom").await;
        let csv = "\u{feff}pipe_id,pipe_type,outer_diameter_mm,wall_thickness_mm,length_m,steel_grade,manufacturer,location,thread_type,batch_no,surface_treatment\n\
             P-7,seamless_pipe,88.9,7.0,9.5,L80,宝钢,A1,EUE,B1,磷化\n";
        let report = fx
            .service
            .import_csv(&fx.actor, csv)
            .await
            .expect("导入失败");
        assert_eq!(report.inserted, 1, "{:?}", report.failed);

        let tube = tubes::find(&fx.pool, "P-7").await.unwrap().unwrap();
        assert_eq!(tube.thread_type.as_deref(), Some("EUE"));
        assert_eq!(tube.batch_no.as_deref(), Some("B1"));
        let spec = tubes::seamless_spec(&fx.pool, "P-7")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(spec.surface_treatment.as_deref(), Some("磷化"));

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }
}
