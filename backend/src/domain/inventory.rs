//! Domain types for inventory operations.
//!
//! Type-safe enums for approval status, inbound/outbound types,
//! and stock status that replace raw string fields across the codebase.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Approval status for inbound and outbound records.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    AutoApproved,
    Pending,
    Approved,
    Rejected,
}

impl FromStr for ApprovalStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto_approved" => Ok(Self::AutoApproved),
            "pending" => Ok(Self::Pending),
            "approved" => Ok(Self::Approved),
            "rejected" => Ok(Self::Rejected),
            _ => Err(()),
        }
    }
}

impl ApprovalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AutoApproved => "auto_approved",
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
        }
    }

    pub fn deserialize_from_string<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(|()| serde::de::Error::custom(format!("Invalid ApprovalStatus: {}", s)))
    }
}

impl std::fmt::Display for ApprovalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Inbound type: purchase receipt, production return, or transfer in.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InboundType {
    Purchase,
    Production,
    Return,
}

impl FromStr for InboundType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "purchase" => Ok(Self::Purchase),
            "production" => Ok(Self::Production),
            "return" => Ok(Self::Return),
            _ => Err(()),
        }
    }
}

impl InboundType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Purchase => "purchase",
            Self::Production => "production",
            Self::Return => "return",
        }
    }
}

impl std::fmt::Display for InboundType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Outbound type: sales, scrapped, or transfer out.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OutboundType {
    Sales,
    Scrapped,
    Transfer,
}

impl FromStr for OutboundType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sales" => Ok(Self::Sales),
            "scrapped" => Ok(Self::Scrapped),
            "transfer" => Ok(Self::Transfer),
            _ => Err(()),
        }
    }
}

impl OutboundType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sales => "sales",
            Self::Scrapped => "scrapped",
            Self::Transfer => "transfer",
        }
    }
}

impl std::fmt::Display for OutboundType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
