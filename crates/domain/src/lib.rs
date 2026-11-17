//! 领域层：枚举、不变量、状态机。无 IO 依赖。
//!
//! 约束：不得依赖 axum / sqlx / tokio —— 由 CI 的 `scripts/ci/layering.sh` 断言。

#![forbid(unsafe_code)]

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

pub mod tube;

/// 字符串枚举的统一解析错误。
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("{kind} 不接受取值 {value:?}")]
pub struct UnknownValue {
    pub kind: &'static str,
    pub value: String,
}

/// 生成「字面量 ↔ 枚举」双向映射，serde 走 snake_case。
macro_rules! str_enum {
    ($(#[$meta:meta])* $name:ident { $($variant:ident => $text:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            pub const ALL: &'static [$name] = &[$($name::$variant),+];

            pub fn as_str(self) -> &'static str {
                match self {
                    $($name::$variant => $text),+
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl FromStr for $name {
            type Err = UnknownValue;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value {
                    $($text => Ok($name::$variant),)+
                    other => Err(UnknownValue { kind: stringify!($name), value: other.to_owned() }),
                }
            }
        }

        impl TryFrom<&str> for $name {
            type Error = UnknownValue;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                value.parse()
            }
        }
    };
}

/// 系统只有三种角色（原 PROJECT_RULES #78）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Operator,
    Viewer,
}

impl Role {
    pub const ALL: [Role; 3] = [Role::Admin, Role::Operator, Role::Viewer];

    pub fn as_str(self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::Operator => "operator",
            Role::Viewer => "viewer",
        }
    }

    /// 是否具备写权限（原 `require_operator`）。
    pub fn can_write(self) -> bool {
        matches!(self, Role::Admin | Role::Operator)
    }

    /// 是否具备管理权限（原 `require_admin`）。
    pub fn is_admin(self) -> bool {
        matches!(self, Role::Admin)
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Role {
    type Err = UnknownValue;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "admin" => Ok(Role::Admin),
            "operator" => Ok(Role::Operator),
            "viewer" => Ok(Role::Viewer),
            other => Err(UnknownValue {
                kind: "Role",
                value: other.to_owned(),
            }),
        }
    }
}

str_enum!(
    /// 管子类型（原 `tubes.type`）。
    TubeType {
        SeamlessPipe => "seamless_pipe",
        ScreenPipe => "screen_pipe",
    }
);

str_enum!(
    /// 管子状态（原 `tubes.status`）：在库 / 调拨中 / 已出库。
    TubeStatus {
        InStock => "in_stock",
        InTransfer => "in_transfer",
        OutStock => "out_stock",
    }
);

str_enum!(
    /// 质量标记（原 `tubes.quality_mark`）。
    QualityMark {
        Normal => "normal",
        Problematic => "problematic",
        Scrapped => "scrapped",
    }
);

str_enum!(
    /// 单据类型（设计 §4.4）。
    TransactionType {
        In => "in",
        Out => "out",
        Move => "move",
        Check => "check",
    }
);

str_enum!(
    /// 盘点单状态。
    CheckStatus {
        Open => "open",
        Confirmed => "confirmed",
    }
);

str_enum!(
    /// 盘点明细结论。
    CheckResult {
        Ok => "ok",
        Missing => "missing",
        Moved => "moved",
        Surplus => "surplus",
    }
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_only_the_three_roles() {
        for role in Role::ALL {
            assert_eq!(role.as_str().parse::<Role>().unwrap(), role);
        }
        assert_eq!(
            "root".parse::<Role>(),
            Err(UnknownValue {
                kind: "Role",
                value: "root".into()
            })
        );
        assert!("Admin".parse::<Role>().is_err(), "角色必须小写");
        assert!("".parse::<Role>().is_err());
    }

    #[test]
    fn write_and_admin_capabilities_are_distinct() {
        assert!(Role::Admin.can_write() && Role::Admin.is_admin());
        assert!(Role::Operator.can_write() && !Role::Operator.is_admin());
        assert!(!Role::Viewer.can_write() && !Role::Viewer.is_admin());
    }

    #[test]
    fn serde_uses_lowercase() {
        assert_eq!(
            serde_json::to_string(&Role::Operator).unwrap(),
            "\"operator\""
        );
        assert_eq!(
            serde_json::from_str::<Role>("\"viewer\"").unwrap(),
            Role::Viewer
        );
        assert!(serde_json::from_str::<Role>("\"ROOT\"").is_err());
    }

    #[test]
    fn every_str_enum_roundtrips_and_rejects_unknown() {
        macro_rules! check {
            ($ty:ty) => {
                for value in <$ty>::ALL {
                    let text = value.as_str();
                    assert_eq!(text.parse::<$ty>().unwrap(), *value);
                    let json = serde_json::to_string(value).unwrap();
                    assert_eq!(json, format!("\"{text}\""));
                    assert_eq!(serde_json::from_str::<$ty>(&json).unwrap(), *value);
                }
                assert!(
                    format!("{}_nope", <$ty>::ALL[0].as_str())
                        .parse::<$ty>()
                        .is_err()
                );
            };
        }
        check!(TubeType);
        check!(TubeStatus);
        check!(QualityMark);
        check!(TransactionType);
        check!(CheckStatus);
        check!(CheckResult);
    }

    #[test]
    fn documented_literals_are_exact() {
        assert_eq!(TubeType::SeamlessPipe.as_str(), "seamless_pipe");
        assert_eq!(TubeType::ScreenPipe.as_str(), "screen_pipe");
        assert_eq!(TubeStatus::InStock.as_str(), "in_stock");
        assert_eq!(TubeStatus::InTransfer.as_str(), "in_transfer");
        assert_eq!(TubeStatus::OutStock.as_str(), "out_stock");
        assert_eq!(QualityMark::Normal.as_str(), "normal");
        assert_eq!(QualityMark::Problematic.as_str(), "problematic");
        assert_eq!(QualityMark::Scrapped.as_str(), "scrapped");
        assert_eq!(TransactionType::Move.as_str(), "move");
        assert_eq!(CheckResult::Ok.as_str(), "ok");
    }
}
