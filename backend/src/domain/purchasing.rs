//! 采购订单状态机 —— 唯一权威迁移表（ADR-R1）
//!
//! 生命周期：draft → submitted → (approved | rejected)
//!            draft|submitted → cancelled
//!            approved|partially_received → partially_received|received（多次收货）
//!
//! service 层禁止再出现裸字符串比较（`order.status != "draft"`），一律经本模块校验；
//! `allowed_actions` 同时用于后端校验与前端按钮渲染（ADR-R4）。

use crate::error::{AppError, ErrorCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PurchaseOrderStatus {
    Draft,
    Submitted,
    Approved,
    Rejected,
    Cancelled,
    PartiallyReceived,
    Received,
}

impl PurchaseOrderStatus {
    pub fn parse(s: &str) -> Result<Self, AppError> {
        Ok(match s {
            "draft" => Self::Draft,
            "submitted" => Self::Submitted,
            "approved" => Self::Approved,
            "rejected" => Self::Rejected,
            "cancelled" => Self::Cancelled,
            "partially_received" => Self::PartiallyReceived,
            "received" => Self::Received,
            other => {
                return Err(AppError::new(
                    ErrorCode::InvalidTransition,
                    format!("未知采购订单状态: {other}"),
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
            Self::PartiallyReceived => "partially_received",
            Self::Received => "received",
        }
    }

    /// 当前状态下可执行的动作（唯一权威源）。
    pub fn allowed_actions(self) -> &'static [&'static str] {
        match self {
            Self::Draft => &["edit", "delete", "submit", "cancel"],
            Self::Submitted => &["approve", "reject", "cancel"],
            Self::Approved => &["receive"],
            // 部分收货后允许继续收货（修复 v2 "partially_received 死锁态"）
            Self::PartiallyReceived => &["receive"],
            Self::Rejected | Self::Cancelled | Self::Received => &[],
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
            format!("采购订单当前状态为 {}，不可{desc}", self.as_str()),
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
            "partially_received",
            "received",
        ] {
            let st = PurchaseOrderStatus::parse(s).unwrap();
            assert_eq!(st.as_str(), s);
        }
        assert!(PurchaseOrderStatus::parse("bogus").is_err());
    }

    #[test]
    fn draft_can_submit_cancel_edit_delete_only() {
        let a = PurchaseOrderStatus::Draft.allowed_actions();
        assert!(a.contains(&"submit") && a.contains(&"cancel"));
        assert!(a.contains(&"edit") && a.contains(&"delete"));
        assert!(!a.contains(&"approve") && !a.contains(&"receive"));
    }

    #[test]
    fn submitted_can_approve_reject_cancel() {
        let a = PurchaseOrderStatus::Submitted.allowed_actions();
        assert!(a.contains(&"approve") && a.contains(&"reject") && a.contains(&"cancel"));
        assert!(!a.contains(&"submit") && !a.contains(&"edit"));
    }

    #[test]
    fn approved_can_receive_and_terminal_states_empty() {
        assert!(PurchaseOrderStatus::Approved.can("receive"));
        assert!(PurchaseOrderStatus::PartiallyReceived.can("receive"));
        assert!(PurchaseOrderStatus::Received.allowed_actions().is_empty());
        assert!(PurchaseOrderStatus::Rejected.allowed_actions().is_empty());
        assert!(PurchaseOrderStatus::Cancelled.allowed_actions().is_empty());
    }

    #[test]
    fn ensure_can_keeps_v2_error_shape() {
        let err = PurchaseOrderStatus::Approved
            .ensure_can("submit", "提交")
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::OrderCannotModify);
        assert_eq!(err.code.code(), 14001);
        assert!(err.message.contains("approved") && err.message.contains("不可提交"));
    }
}
