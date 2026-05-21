use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PipeType {
    Casing,
    Tubing,
}

impl PipeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Casing => "casing",
            Self::Tubing => "tubing",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "casing" => Some(Self::Casing),
            "tubing" => Some(Self::Tubing),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PipeStatus {
    InStock,
    Outbound,
    Scrapped,
}

impl PipeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InStock => "in_stock",
            Self::Outbound => "outbound",
            Self::Scrapped => "scrapped",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "in_stock" => Some(Self::InStock),
            "outbound" => Some(Self::Outbound),
            "scrapped" => Some(Self::Scrapped),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EndType {
    SC,
    LC,
    BC,
    X,
}

impl EndType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SC => "SC",
            Self::LC => "LC",
            Self::BC => "BC",
            Self::X => "X",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "SC" => Some(Self::SC),
            "LC" => Some(Self::LC),
            "BC" => Some(Self::BC),
            "X" => Some(Self::X),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ScreenType {
    WireWrapped,
    Slotted,
    Punched,
    MetalFelt,
}

impl ScreenType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WireWrapped => "wire_wrapped",
            Self::Slotted => "slotted",
            Self::Punched => "punched",
            Self::MetalFelt => "metal_felt",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "wire_wrapped" => Some(Self::WireWrapped),
            "slotted" => Some(Self::Slotted),
            "punched" => Some(Self::Punched),
            "metal_felt" => Some(Self::MetalFelt),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InboundType {
    Purchase,
    Production,
    Return,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OutboundType {
    Sales,
    Transfer,
    Scrapped,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Inbound,
    Outbound,
    Transfer,
    CheckAdjust,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    AutoApproved,
    Pending,
    Approved,
    Rejected,
}

pub const API_5CT_GRADES: &[&str] = &[
    "H40", "J55", "K55", "N80", "L80", "C90", "T95", "P110", "Q125",
];
