use std::str::FromStr;
use crate::dto::pipe_dto::PipeSearchResult;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Sqlite, QueryBuilder};

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
    /// Welded pipe variants (API 5L welded pipes)
    Welded,
}

impl PipeType {
    /// Classify a raw pipe_type string into `Seamless`, `Screen`, or `Welded`.
    ///
    /// - `"seamless"`, `"casing"`, `"tubing"`, `"line_pipe"` → `Seamless`
    /// - `"screen"`, `"screened"` → `Screen`
    /// - `"welded"` → `Welded`
    pub fn from_pipe_type_str(s: &str) -> Option<Self> {
        match s {
            "seamless" | "casing" | "tubing" | "line_pipe" => Some(Self::Seamless),
            "screen" | "screened" => Some(Self::Screen),
            "welded" => Some(Self::Welded),
            _ => None,
        }
    }

    /// Returns the database table name for this pipe type.
    pub fn table_name(&self) -> &'static str {
        match self {
            Self::Seamless => "seamless_pipes",
            Self::Screen => "screen_pipes",
            Self::Welded => "welded_pipes",
        }
    }

    /// Returns the pipe number prefix for this pipe type.
    pub fn pipe_number_prefix(&self) -> &'static str {
        match self {
            Self::Seamless => "SP",
            Self::Screen => "SCP",
            Self::Welded => "WP",
        }
    }
}

impl FromStr for PipeType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_pipe_type_str(s).ok_or_else(|| format!("Unknown pipe_type: {}", s))
    }
}

/// Pipe status enum — replaces magic strings with type-safe transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PipeStatus {
    New,
    InStock,
    Outbound,
    Scrapped,
    InTransit,
    Reserved,
}

impl PipeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::InStock => "in_stock",
            Self::Outbound => "outbound",
            Self::Scrapped => "scrapped",
            Self::InTransit => "in_transit",
            Self::Reserved => "reserved",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "new" => Some(Self::New),
            "in_stock" => Some(Self::InStock),
            "outbound" => Some(Self::Outbound),
            "scrapped" => Some(Self::Scrapped),
            "in_transit" => Some(Self::InTransit),
            "reserved" => Some(Self::Reserved),
            _ => None,
        }
    }

    pub fn valid_transitions(&self) -> &'static [PipeStatus] {
        match self {
            Self::New => &[Self::InStock],
            Self::InStock => &[Self::Outbound, Self::Scrapped, Self::InTransit, Self::Reserved],
            Self::Outbound => &[Self::InStock],
            Self::InTransit => &[Self::InStock, Self::Outbound],
            Self::Reserved => &[Self::InStock, Self::Outbound],
            Self::Scrapped => &[],
        }
    }

    pub fn can_transition_to(&self, target: PipeStatus) -> bool {
        self.valid_transitions().contains(&target)
    }
}

/// Trait that all pipe models must implement for generic repository/service/handler support.
///
/// This enables writing CRUD logic once and having it work for SeamlessPipe, ScreenPipe,
/// and any future pipe types (WeldedPipe, CoatedPipe, etc.).
pub trait PipeModel: Send + Sync + Serialize + for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Clone + Unpin + 'static {
    /// The DTO for creating a new pipe of this type.
    type CreateDto: Send + Sync + Clone + 'static;
    /// The DTO for updating a pipe of this type.
    type UpdateDto: Send + Sync + Clone + 'static;
    /// The filter params for listing pipes of this type.
    type FilterParams: Send + Sync + Clone + 'static;

    /// Database table name.
    const TABLE_NAME: &'static str;

    /// Pipe type enum value.
    const PIPE_TYPE: PipeType;

    /// Column name for the grade field (grade for seamless, base_grade for screen).
    const GRADE_COLUMN: &'static str;

    /// Column name for the OD field (od for seamless, base_od for screen).
    const OD_COLUMN: &'static str;

    /// Column name for the WT field (wt for seamless, base_wt for screen).
    const WT_COLUMN: &'static str;

    /// Column name for the pipe type field (pipe_type for seamless, screen_type for screen).
    const PIPE_TYPE_COLUMN: &'static str;

    /// Get the pipe number.
    fn pipe_number(&self) -> &str;

    /// Get the current status.
    fn status(&self) -> &str;

    /// Set the status (for mutations).
    fn set_status(&mut self, status: &str);

    /// Get the pipe ID.
    fn id(&self) -> i64;

    /// Get the location ID.
    fn location_id(&self) -> Option<i64>;

    /// Set the location ID.
    fn set_location_id(&mut self, location_id: Option<i64>);

    /// Get the grade value (for search/filter).
    fn grade(&self) -> &str;

    /// Get the OD value (for search/filter).
    fn od(&self) -> f64;

    /// Get the WT value (for search/filter).
    fn wt(&self) -> f64;

    /// Get the pipe type string (for search result).
    fn pipe_type_str(&self) -> &str;

    /// Get the batch number.
    fn batch_number(&self) -> Option<&str>;

    /// Get the heat number.
    fn heat_number(&self) -> Option<&str>;

    /// Get the serial number.
    fn serial_number(&self) -> Option<&str>;

    /// Get the manufacturer.
    fn manufacturer(&self) -> Option<&str>;

    /// Get the deleted_at timestamp (for soft delete check).
    fn deleted_at(&self) -> Option<&str>;

    /// Check if this pipe has been soft-deleted.
    fn is_deleted(&self) -> bool {
        self.deleted_at().is_some()
    }

    /// Validate a sort column name for this pipe type.
    /// Returns Some(column_name) if valid, None if invalid (prevents SQL injection).
    fn valid_sort_column(col: &str) -> Option<&'static str>;

    /// Generate a pipe number for this type.
    fn generate_pipe_number(grade: &str, od: f64, wt: f64) -> String {
        let prefix = Self::PIPE_TYPE.pipe_number_prefix();
        let serial = uuid::Uuid::new_v4().to_string();
        let short_serial = &serial[..8];
        format!("{}-{}-{}x{}-{}", prefix, grade, od, wt, short_serial)
    }

    /// Convert this pipe to a search result.
    fn to_search_result(&self) -> PipeSearchResult {
        PipeSearchResult {
            id: self.id(),
            pipe_type: self.pipe_type_str().to_string(),
            pipe_number: self.pipe_number().to_string(),
            grade: self.grade().to_string(),
            od: self.od(),
            wt: self.wt(),
            status: self.status().to_string(),
            location_id: self.location_id(),
        }
    }

    /// Validate status transition.
    fn validate_status_transition(current: &str, target: &str) -> Result<(), AppError> {
        let current_status = PipeStatus::from_str(current)
            .ok_or_else(|| AppError::Validation(format!("Invalid current status: {}", current)))?;
        let target_status = PipeStatus::from_str(target)
            .ok_or_else(|| AppError::Validation(format!("Invalid target status: {}", target)))?;

        if !current_status.can_transition_to(target_status) {
            return Err(AppError::Validation(format!(
                "Invalid status transition from '{}' to '{}'",
                current, target
            )));
        }
        Ok(())
    }

    /// Build INSERT query using QueryBuilder.
    /// Implementors append column names and values to the builder.
    fn build_create_query<'a>(builder: &mut QueryBuilder<'a, Sqlite>, dto: &'a Self::CreateDto);

    /// Build UPDATE query using QueryBuilder.
    /// Implementors append SET clauses to the builder.
    fn build_update_query<'a>(builder: &mut QueryBuilder<'a, Sqlite>, dto: &'a Self::UpdateDto);
}