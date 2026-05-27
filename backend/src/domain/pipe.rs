use std::str::FromStr;

/// Domain enum for pipe type classification.
///
/// Replaces the scattered `match pipe_type.as_str() { "seamless" | "casing" | "tubing" => ... }`
/// pattern across services and repositories with a single source of truth.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipeType {
    /// Seamless pipe variants (casing, tubing, line pipe, etc.)
    Seamless,
    /// Screen pipe variants (screened / perforated)
    Screen,
}

impl PipeType {
    /// Classify a raw pipe_type string into `Seamless` or `Screen`.
    ///
    /// - `"seamless"`, `"casing"`, `"tubing"`, `"line_pipe"` → `Seamless`
    /// - `"screen"`, `"screened"` → `Screen`
    pub fn from_pipe_type_str(s: &str) -> Option<Self> {
        match s {
            "seamless" | "casing" | "tubing" | "line_pipe" => Some(Self::Seamless),
            "screen" | "screened" => Some(Self::Screen),
            _ => None,
        }
    }

    /// Returns the database table name for this pipe type.
    pub fn table_name(&self) -> &'static str {
        match self {
            Self::Seamless => "seamless_pipes",
            Self::Screen => "screen_pipes",
        }
    }
}

impl FromStr for PipeType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_pipe_type_str(s)
            .ok_or_else(|| format!("Unknown pipe_type: {}", s))
    }
}
