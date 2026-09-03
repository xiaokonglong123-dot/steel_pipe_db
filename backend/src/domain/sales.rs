//! 销售订单状态机 —— 唯一权威迁移表（ADR-R1）
//!
//! 生命周期：draft → submitted → (approved | rejected)
//!            submitted|approved → cancelled（submitted 释放预留）
//!            approved → shipped（一次性发货；v3 沿用 v2 语义，不引入 awaiting_shipment）
//!
//! service 层禁止使用裸字符串比较，一律经本模块校验。

use crate::error::{AppError, ErrorCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SalesOrderStatus {
    Draft,
    Submitted,
    Approved,
    Rejected,
    Cancelled,
    Shipped,
}

impl SalesOrderStatus {
    pub fn parse(s: &str) -> Result<Self, AppError> {
        Ok(match s {
            "draft" => Self::Draft,
            "submitted" => Self::Submitted,
            "approved" => Self::Approved,
            "rejected" => Self::Rejected,
            "cancelled" => Self::Cancelled,
            "shipped" => Self::Shipped,
            other => {
                return Err(AppError::new(
                    ErrorCode::InvalidTransition,
                    format!("未知销售订单状态: {other}"),
                ))
            }
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Submitted => "submitted",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Cancelled => "cancelled",
            Self::Shipped => "shipped",
        }
    }

    /// 当前状态下可执行的动作（唯一权威源）。
    ///
    /// 注：draft 不可 cancel（v2 语义：草稿只能删除，取消是有预留/已审批单据的操作）。
    pub fn allowed_actions(self) -> &'static [&'static str] {
        match self {
            Self::Draft => &["edit", "delete", "submit"],
            Self::Submitted => &["approve", "reject", "cancel"],
            Self::Approved => &["ship", "cancel"],
            Self::Rejected | Self::Cancelled | Self::Shipped => &[],
        }
    }

    pub fn can(self, action: &str) -> bool {
        self.allowed_actions().contains(&action)
    }

    /// 校验动作合法性；非法时返回与 v2 一致的错误（OrderCannotModify 14001）。
    pub fn ensure_can(self, action: &str, desc: &str) -> Result<(), AppError> {
        if self.can(action) {
            return Ok(());
        }
        Err(AppError::new(
            ErrorCode::OrderCannotModify,
            format!("销售订单当前状态为 {}，不可{desc}", self.as_str()),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_all_states() {
        for s in [
            "draft",
            "submitted",
            "approved",
            "rejected",
            "cancelled",
            "shipped",
        ] {
            let st = SalesOrderStatus::parse(s).unwrap();
            assert_eq!(st.as_str(), s);
        }
        assert!(SalesOrderStatus::parse("bogus").is_err());
    }

    #[test]
    fn draft_can_submit_edit_delete_but_not_cancel() {
        let a = SalesOrderStatus::Draft.allowed_actions();
        assert!(a.contains(&"submit") && a.contains(&"edit") && a.contains(&"delete"));
        assert!(!a.contains(&"cancel") && !a.contains(&"approve"));
    }

    #[test]
    fn submitted_can_approve_reject_cancel() {
        let a = SalesOrderStatus::Submitted.allowed_actions();
        assert!(a.contains(&"approve") && a.contains(&"reject") && a.contains(&"cancel"));
    }

    #[test]
    fn approved_can_ship_and_cancel_or_terminal_empty() {
        assert!(SalesOrderStatus::Approved.can("ship"));
        assert!(SalesOrderStatus::Approved.can("cancel"));
        assert!(SalesOrderStatus::Shipped.allowed_actions().is_empty());
        assert!(SalesOrderStatus::Rejected.allowed_actions().is_empty());
        assert!(SalesOrderStatus::Cancelled.allowed_actions().is_empty());
    }

    #[test]
    fn ensure_can_keeps_v2_error_shape() {
        let err = SalesOrderStatus::Approved.ensure_can("submit", "提交").unwrap_err();
        assert_eq!(err.code, ErrorCode::OrderCannotModify);
        assert!(err.message.contains("approved") && err.message.contains("不可提交"));
    }
}
