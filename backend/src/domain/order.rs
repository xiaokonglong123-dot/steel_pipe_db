use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Lifecycle state for purchase and sales orders.
///
/// Transitions:
/// `Draft → Pending → Approved → Completed`
/// `Draft → Cancelled`
/// `Pending → Rejected → Draft`
/// `Approved → Cancelled`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Draft,
    Pending,
    Approved,
    Rejected,
    Completed,
    Cancelled,
}

impl FromStr for OrderStatus {
    type Err = ();

    /// Parse a string into an order status. Returns `Err(())` if the string's garbage.
    /// Valid values: `"draft" | "pending" | "approved" | "rejected" | "completed" | "cancelled"`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(Self::Draft),
            "pending" => Ok(Self::Pending),
            "approved" => Ok(Self::Approved),
            "rejected" => Ok(Self::Rejected),
            "completed" => Ok(Self::Completed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(()),
        }
    }
}

impl OrderStatus {
    /// Convert to the snake_case string stored in the database.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }

    /// Custom serde deserializer that accepts both the enum form and a plain string.
    /// This allows seamless reading from JSON (`"approved"`) and from sqlx string columns.
    pub fn deserialize_from_string<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(|()| serde::de::Error::custom(format!("Invalid OrderStatus: {}", s)))
    }
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl OrderStatus {

    /// Check whether transitioning from the current status to the target is valid.
    /// Valid transition matrix:
    /// - Draft → Pending | Cancelled
    /// - Pending → Approved | Rejected
    /// - Rejected → Draft
    /// - Approved → Completed | Cancelled
    pub fn valid_transition(&self, target: &Self) -> bool {
        matches!(
            (self, target),
            (Self::Draft, Self::Pending)
                | (Self::Pending, Self::Approved)
                | (Self::Pending, Self::Rejected)
                | (Self::Rejected, Self::Draft)
                | (Self::Approved, Self::Completed)
                | (Self::Approved, Self::Cancelled)
                | (Self::Draft, Self::Cancelled)
        )
    }
}
