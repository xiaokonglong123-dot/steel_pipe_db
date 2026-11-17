//! 管子领域规则：状态机、入库校验、pipe_id 规范。
//!
//! 对应旧实现散落在 `tube_service.py`/`tubes.py` 里的守卫（D 域），
//! 在这里变成可单测的纯函数。

use crate::{QualityMark, TubeStatus, TubeType};

/// 管子编号最大长度（旧实现为 TEXT，无长度约束；这里收紧到 64）。
pub const MAX_PIPE_ID_LEN: usize = 64;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TubeError {
    #[error("管子编号不能为空")]
    EmptyPipeId,
    #[error("管子编号长度 {0} > {MAX_PIPE_ID_LEN}")]
    PipeIdTooLong(usize),
    #[error("管子编号只能包含字母、数字与 `-_.:/`")]
    PipeIdCharset,
    #[error("钢级不能为空")]
    EmptyGrade,
    #[error("外径必须为正")]
    NonPositiveOuterDiameter,
    #[error("壁厚必须为正")]
    NonPositiveWallThickness,
    #[error("长度必须为正")]
    NonPositiveLength,
    #[error("库位为空")]
    EmptyLocation,
    #[error("管子当前状态为 {0}，不允许出库")]
    NotInStock(TubeStatus),
    #[error("管子当前状态为 {0}，不允许调拨")]
    NotTransferable(TubeStatus),
    #[error("管子在调拨中，不能直接出库")]
    InTransfer,
    #[error("问题管/报废管需要管理员确认")]
    ProblematicRequiresAdmin,
    #[error("报废管不能出库或调拨")]
    Scrapped,
    #[error("状态 {from} 无法变更为 {to}")]
    IllegalTransition { from: TubeStatus, to: TubeStatus },
}

/// 校验并规范化管子编号（trim + 字符集检查）。
pub fn normalize_pipe_id(raw: &str) -> Result<String, TubeError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(TubeError::EmptyPipeId);
    }
    let length = trimmed.chars().count();
    if length > MAX_PIPE_ID_LEN {
        return Err(TubeError::PipeIdTooLong(length));
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ':' | '/'))
    {
        return Err(TubeError::PipeIdCharset);
    }
    Ok(trimmed.to_owned())
}

/// 入库前的几何/标识校验（合规校验另由 `steelpipe-compliance` 负责）。
#[derive(Debug, Clone)]
pub struct TubeDraft {
    pub pipe_id: String,
    pub pipe_type: TubeType,
    pub outer_diameter_mm: f64,
    pub wall_thickness_mm: f64,
    pub length_m: f64,
    pub steel_grade: String,
    pub thread_type: Option<String>,
    pub manufacturer: String,
    pub batch_no: Option<String>,
    pub heat_no: Option<String>,
    pub heat_treat_method: Option<String>,
    pub location: String,
    pub api_5ct_cert: Option<String>,
    pub quality_mark: QualityMark,
    pub quality_note: Option<String>,
}

impl TubeDraft {
    /// 校验后返回规范化编号。
    pub fn validate(&self) -> Result<String, TubeError> {
        let pipe_id = normalize_pipe_id(&self.pipe_id)?;
        if self.steel_grade.trim().is_empty() {
            return Err(TubeError::EmptyGrade);
        }
        if !self.outer_diameter_mm.is_finite() || self.outer_diameter_mm <= 0.0 {
            return Err(TubeError::NonPositiveOuterDiameter);
        }
        if !self.wall_thickness_mm.is_finite() || self.wall_thickness_mm <= 0.0 {
            return Err(TubeError::NonPositiveWallThickness);
        }
        if !self.length_m.is_finite() || self.length_m <= 0.0 {
            return Err(TubeError::NonPositiveLength);
        }
        if self.location.trim().is_empty() {
            return Err(TubeError::EmptyLocation);
        }
        Ok(pipe_id)
    }
}

/// 出库前置检查（原 `stock_out` 的 TOCTOU 守卫在 SQL 层，这里先做状态判定）。
pub fn ensure_stock_out_ready(
    status: TubeStatus,
    quality: QualityMark,
    actor_is_admin: bool,
) -> Result<(), TubeError> {
    match status {
        TubeStatus::InStock => {}
        TubeStatus::InTransfer => return Err(TubeError::InTransfer),
        other => return Err(TubeError::NotInStock(other)),
    }
    match quality {
        QualityMark::Normal => Ok(()),
        QualityMark::Problematic if actor_is_admin => Ok(()),
        QualityMark::Problematic => Err(TubeError::ProblematicRequiresAdmin),
        QualityMark::Scrapped => Err(TubeError::Scrapped),
    }
}

/// 调拨前置检查。
pub fn ensure_transfer_ready(
    status: TubeStatus,
    quality: QualityMark,
    actor_is_admin: bool,
) -> Result<(), TubeError> {
    match status {
        TubeStatus::InStock => {}
        other => return Err(TubeError::NotTransferable(other)),
    }
    match quality {
        QualityMark::Scrapped => Err(TubeError::Scrapped),
        QualityMark::Problematic if !actor_is_admin => Err(TubeError::ProblematicRequiresAdmin),
        _ => Ok(()),
    }
}

/// 状态机：只有明确允许的迁移才成立。
pub fn transition(from: TubeStatus, to: TubeStatus) -> Result<TubeStatus, TubeError> {
    use TubeStatus::*;
    let allowed = matches!(
        (from, to),
        (InStock, InTransfer)
            | (InTransfer, InStock)
            | (InTransfer, OutStock)
            | (InStock, OutStock)
    );
    if allowed {
        Ok(to)
    } else {
        Err(TubeError::IllegalTransition { from, to })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft() -> TubeDraft {
        TubeDraft {
            pipe_id: "P-0001".into(),
            pipe_type: TubeType::SeamlessPipe,
            outer_diameter_mm: 88.9,
            wall_thickness_mm: 7.0,
            length_m: 9.5,
            steel_grade: "L80".into(),
            thread_type: Some("EUE".into()),
            manufacturer: "宝钢".into(),
            batch_no: None,
            heat_no: None,
            heat_treat_method: None,
            location: "A1".into(),
            api_5ct_cert: None,
            quality_mark: QualityMark::Normal,
            quality_note: None,
        }
    }

    #[test]
    fn pipe_id_is_normalized_and_validated() {
        assert_eq!(normalize_pipe_id("  P-0001  ").unwrap(), "P-0001");
        assert_eq!(normalize_pipe_id("A/B:C-1_2.3").unwrap(), "A/B:C-1_2.3");
        assert_eq!(normalize_pipe_id("   "), Err(TubeError::EmptyPipeId));
        assert_eq!(normalize_pipe_id("P 0001"), Err(TubeError::PipeIdCharset));
        assert_eq!(normalize_pipe_id("管-1"), Err(TubeError::PipeIdCharset));
        let too_long = "P".repeat(MAX_PIPE_ID_LEN + 1);
        assert_eq!(
            normalize_pipe_id(&too_long),
            Err(TubeError::PipeIdTooLong(MAX_PIPE_ID_LEN + 1))
        );
    }

    #[test]
    fn draft_rejects_non_positive_dimensions() {
        let mut bad = draft();
        bad.outer_diameter_mm = 0.0;
        assert_eq!(bad.validate(), Err(TubeError::NonPositiveOuterDiameter));
        bad = draft();
        bad.wall_thickness_mm = -1.0;
        assert_eq!(bad.validate(), Err(TubeError::NonPositiveWallThickness));
        bad = draft();
        bad.length_m = 0.0;
        assert_eq!(bad.validate(), Err(TubeError::NonPositiveLength));
        bad = draft();
        bad.steel_grade = "  ".into();
        assert_eq!(bad.validate(), Err(TubeError::EmptyGrade));
        bad = draft();
        bad.location = "".into();
        assert_eq!(bad.validate(), Err(TubeError::EmptyLocation));
        assert_eq!(draft().validate().unwrap(), "P-0001");
    }

    #[test]
    fn stock_out_rules_follow_quality_and_role() {
        assert!(ensure_stock_out_ready(TubeStatus::InStock, QualityMark::Normal, false).is_ok());
        assert_eq!(
            ensure_stock_out_ready(TubeStatus::InStock, QualityMark::Problematic, false),
            Err(TubeError::ProblematicRequiresAdmin)
        );
        assert!(
            ensure_stock_out_ready(TubeStatus::InStock, QualityMark::Problematic, true).is_ok()
        );
        assert_eq!(
            ensure_stock_out_ready(TubeStatus::InStock, QualityMark::Scrapped, true),
            Err(TubeError::Scrapped)
        );
        assert_eq!(
            ensure_stock_out_ready(TubeStatus::InTransfer, QualityMark::Normal, true),
            Err(TubeError::InTransfer)
        );
        assert_eq!(
            ensure_stock_out_ready(TubeStatus::OutStock, QualityMark::Normal, true),
            Err(TubeError::NotInStock(TubeStatus::OutStock))
        );
    }

    #[test]
    fn transfer_rules_allow_problematic_only_for_admin() {
        assert!(ensure_transfer_ready(TubeStatus::InStock, QualityMark::Normal, false).is_ok());
        assert_eq!(
            ensure_transfer_ready(TubeStatus::InStock, QualityMark::Problematic, false),
            Err(TubeError::ProblematicRequiresAdmin)
        );
        assert!(ensure_transfer_ready(TubeStatus::InStock, QualityMark::Problematic, true).is_ok());
        assert_eq!(
            ensure_transfer_ready(TubeStatus::InStock, QualityMark::Scrapped, true),
            Err(TubeError::Scrapped)
        );
        assert_eq!(
            ensure_transfer_ready(TubeStatus::OutStock, QualityMark::Normal, true),
            Err(TubeError::NotTransferable(TubeStatus::OutStock))
        );
    }

    #[test]
    fn state_machine_only_allows_known_transitions() {
        use TubeStatus::*;
        for (from, to) in [
            (InStock, InTransfer),
            (InTransfer, InStock),
            (InTransfer, OutStock),
            (InStock, OutStock),
        ] {
            assert_eq!(transition(from, to).unwrap(), to, "{from} → {to} 应允许");
        }
        for (from, to) in [
            (OutStock, InStock),
            (OutStock, InTransfer),
            (InStock, InStock),
            (InTransfer, InTransfer),
        ] {
            assert!(
                matches!(
                    transition(from, to),
                    Err(TubeError::IllegalTransition { .. })
                ),
                "{from} → {to} 不该允许"
            );
        }
    }
}
