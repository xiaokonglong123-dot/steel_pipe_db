//! 库存单据状态机 —— 入库单 / 出库单 / 盘点单（ADR-R1）
//!
//! 三者都很小，但与订单一样：service 层不得裸比较字符串，
//! 动作校验与未来 allowed_actions 下发（ADR-R4）统一以本模块为唯一权威源。

use crate::error::{AppError, ErrorCode};

macro_rules! simple_status {
    ($name:ident, $doc:expr, { $($variant:ident => $raw:literal : [$($action:literal),*],)* }) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name {
            $( $variant ),*
        }

        impl $name {
            pub fn parse(s: &str) -> Result<Self, AppError> {
                Ok(match s {
                    $( $raw => Self::$variant, )*
                    other => {
                        return Err(AppError::new(
                            ErrorCode::InvalidTransition,
                            format!("未知{}状态: {other}", $doc),
                        ))
                    }
                })
            }

            pub fn as_str(self) -> &'static str {
                match self {
                    $( Self::$variant => $raw, )*
                }
            }

            pub fn allowed_actions(self) -> &'static [&'static str] {
                match self {
                    $( Self::$variant => &[ $( $action ),* ], )*
                }
            }

            pub fn can(self, action: &str) -> bool {
                self.allowed_actions().contains(&action)
            }

            /// 校验动作合法性；`code` 由调用点决定（入/出库 14001，盘点 13004）。
            pub fn ensure_can(self, code: ErrorCode, action: &str, desc: &str) -> Result<(), AppError> {
                if self.can(action) {
                    return Ok(());
                }
                Err(AppError::new(
                    code,
                    format!("{}当前状态为 {}，不可{desc}", $doc, self.as_str()),
                ))
            }
        }
    };
}

simple_status!(
    InboundStatus,
    "入库单",
    {
        Draft => "draft": ["post"],
        Posted => "posted": [],
        Cancelled => "cancelled": [],
    }
);

simple_status!(
    OutboundStatus,
    "出库单",
    {
        Draft => "draft": ["post"],
        Posted => "posted": [],
        Cancelled => "cancelled": [],
    }
);

simple_status!(
    CheckSessionStatus,
    "盘点单",
    {
        // record：录入实盘数量（draft|counted 均可反复录入）
        Draft => "draft": ["record"],
        Counted => "counted": ["record", "post"],
        Posted => "posted": [],
        Cancelled => "cancelled": [],
    }
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_all() {
        for s in ["draft", "posted", "cancelled"] {
            assert_eq!(InboundStatus::parse(s).unwrap().as_str(), s);
            assert_eq!(OutboundStatus::parse(s).unwrap().as_str(), s);
        }
        for s in ["draft", "counted", "posted", "cancelled"] {
            assert_eq!(CheckSessionStatus::parse(s).unwrap().as_str(), s);
        }
        assert!(InboundStatus::parse("bogus").is_err());
    }

    #[test]
    fn inbound_draft_can_post_only() {
        assert!(InboundStatus::Draft.can("post"));
        assert!(!InboundStatus::Draft.can("cancel")); // v2 未提供取消入库单入口
        assert!(InboundStatus::Posted.allowed_actions().is_empty());
    }

    #[test]
    fn check_session_transitions() {
        assert!(CheckSessionStatus::Draft.can("record"));
        assert!(!CheckSessionStatus::Draft.can("post")); // 未录入不可过账
        assert!(CheckSessionStatus::Counted.can("record"));
        assert!(CheckSessionStatus::Counted.can("post"));
        assert!(CheckSessionStatus::Posted.allowed_actions().is_empty());
    }

    #[test]
    fn ensure_can_uses_supplied_error_code() {
        let err = CheckSessionStatus::Posted
            .ensure_can(ErrorCode::CheckNotDraft, "post", "过账")
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::CheckNotDraft);
        assert_eq!(err.code.code(), 13004);
        assert!(err.message.contains("盘点单") && err.message.contains("不可过账"));
        let err = InboundStatus::Posted
            .ensure_can(ErrorCode::OrderCannotModify, "post", "过账")
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::OrderCannotModify);
    }
}
