use serde::{Deserialize, Serialize};

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

impl OrderStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "draft" => Some(Self::Draft),
            "pending" => Some(Self::Pending),
            "approved" => Some(Self::Approved),
            "rejected" => Some(Self::Rejected),
            "completed" => Some(Self::Completed),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }

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
