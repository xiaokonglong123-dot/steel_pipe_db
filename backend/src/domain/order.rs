//! 订单状态机（采购/销售共享状态集合 + 迁移规则，对齐 detailed-design §4.5/4.6）

use crate::error::ErrorCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Draft,
    Submitted,
    Approved,
    Rejected,
    Cancelled,
    // 采购
    Ordered,
    PartiallyReceived,
    Received,
    // 销售
    AwaitingShipment,
    PartiallyShipped,
    Shipped,
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Submitted => "submitted",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Cancelled => "cancelled",
            Self::Ordered => "ordered",
            Self::PartiallyReceived => "partially_received",
            Self::Received => "received",
            Self::AwaitingShipment => "awaiting_shipment",
            Self::PartiallyShipped => "partially_shipped",
            Self::Shipped => "shipped",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, crate::AppError> {
        Ok(match s {
            "draft" => Self::Draft,
            "submitted" => Self::Submitted,
            "approved" => Self::Approved,
            "rejected" => Self::Rejected,
            "cancelled" => Self::Cancelled,
            "ordered" => Self::Ordered,
            "partially_received" => Self::PartiallyReceived,
            "received" => Self::Received,
            "awaiting_shipment" => Self::AwaitingShipment,
            "partially_shipped" => Self::PartiallyShipped,
            "shipped" => Self::Shipped,
            other => {
                return Err(crate::AppError::new(
                    ErrorCode::InvalidTransition,
                    format!("未知订单状态: {other}"),
                ))
            }
        })
    }
}

/// doc_status 全局语义（对齐 detailed-design §4.5 审批联动规则）：
///   0=草稿/未提交审批，1=已提交或审批完成，2=已取消
pub const DOC_DRAFT: i64 = 0;
pub const DOC_SUBMITTED: i64 = 1;
pub const DOC_CANCELLED: i64 = 2;
