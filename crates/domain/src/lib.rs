//! 领域层：实体、角色、不变量。
//!
//! 约束：不得依赖 axum / sqlx / tokio —— 本 crate 只描述业务，
//! 由 CI 的 `scripts/ci/layering.sh` 断言。

#![forbid(unsafe_code)]

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

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

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("非法角色：{0}（只允许 admin / operator / viewer）")]
pub struct UnknownRole(pub String);

impl FromStr for Role {
    type Err = UnknownRole;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "admin" => Ok(Role::Admin),
            "operator" => Ok(Role::Operator),
            "viewer" => Ok(Role::Viewer),
            other => Err(UnknownRole(other.to_owned())),
        }
    }
}

impl TryFrom<&str> for Role {
    type Error = UnknownRole;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_only_the_three_roles() {
        for role in Role::ALL {
            assert_eq!(role.as_str().parse::<Role>().unwrap(), role);
        }
        assert_eq!("root".parse::<Role>(), Err(UnknownRole("root".into())));
        assert_eq!("Admin".parse::<Role>(), Err(UnknownRole("Admin".into())));
        assert_eq!("".parse::<Role>(), Err(UnknownRole(String::new())));
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
}
