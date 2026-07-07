use crate::domain::pipe::{PipeModel, PipeType, PipeStatus};
use crate::dto::pipe_dto::{CreateSeamlessPipeRequest, UpdateSeamlessPipeRequest, PipeFilterParams, PipeSearchResult};
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, QueryBuilder, Sqlite};

/// Seamless pipe DB row. API 5CT standard master data.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SeamlessPipe {
    pub id: i64,
    pub pipe_number: String,
    pub batch_number: Option<String>,
    pub pipe_type: String,
    pub grade: String,
    pub od: f64,
    pub wt: f64,
    pub length: Option<f64>,
    pub weight_per_unit: Option<f64>,
    pub end_type: Option<String>,
    pub coupling_type: Option<String>,
    pub coupling_od: Option<f64>,
    pub coupling_length: Option<f64>,
    pub heat_number: Option<String>,
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
    pub production_date: Option<String>,
    pub cert_number: Option<String>,
    pub location_id: Option<i64>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

impl PipeModel for SeamlessPipe {
    type CreateDto = CreateSeamlessPipeRequest;
    type UpdateDto = UpdateSeamlessPipeRequest;
    type FilterParams = PipeFilterParams;

    const TABLE_NAME: &'static str = "seamless_pipes";
    const PIPE_TYPE: PipeType = PipeType::Seamless;
    const GRADE_COLUMN: &'static str = "grade";
    const OD_COLUMN: &'static str = "od";
    const WT_COLUMN: &'static str = "wt";
    const PIPE_TYPE_COLUMN: &'static str = "pipe_type";

    fn pipe_number(&self) -> &str { &self.pipe_number }
    fn status(&self) -> &str { &self.status }
    fn set_status(&mut self, status: &str) { self.status = status.to_string(); }
    fn id(&self) -> i64 { self.id }
    fn location_id(&self) -> Option<i64> { self.location_id }
    fn set_location_id(&mut self, location_id: Option<i64>) { self.location_id = location_id; }
    fn grade(&self) -> &str { &self.grade }
    fn od(&self) -> f64 { self.od }
    fn wt(&self) -> f64 { self.wt }
    fn pipe_type_str(&self) -> &str { "seamless" }
    fn batch_number(&self) -> Option<&str> { self.batch_number.as_deref() }
    fn heat_number(&self) -> Option<&str> { self.heat_number.as_deref() }
    fn serial_number(&self) -> Option<&str> { self.serial_number.as_deref() }
    fn manufacturer(&self) -> Option<&str> { self.manufacturer.as_deref() }
    fn deleted_at(&self) -> Option<&str> { self.deleted_at.as_deref() }
    fn valid_sort_column(col: &str) -> Option<&'static str> {
        match col {
            "pipe_number" => Some("pipe_number"),
            "grade" => Some("grade"),
            "od" => Some("od"),
            "wt" => Some("wt"),
            "status" => Some("status"),
            "manufacturer" => Some("manufacturer"),
            "production_date" => Some("production_date"),
            _ => None,
        }
    }

    fn generate_pipe_number(grade: &str, od: f64, wt: f64) -> String {
        let prefix = Self::PIPE_TYPE.pipe_number_prefix();
        let serial = uuid::Uuid::new_v4().to_string();
        let short_serial = &serial[..8];
        format!("{}-{}-{}x{}-{}", prefix, grade, od, wt, short_serial)
    }

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

    fn build_create_query<'a>(builder: &mut QueryBuilder<'a, Sqlite>, dto: &'a Self::CreateDto) {
        builder
            .push(", pipe_type, grade, od, wt, length, weight_per_unit, end_type, \
                  coupling_type, coupling_od, coupling_length, heat_number, serial_number, \
                  manufacturer, production_date, cert_number, location_id, notes, \
                  status) VALUES (");
        let mut sep = builder.separated(", ");
        sep.push_bind(dto.pipe_number.as_deref())
            .push_bind(dto.batch_number.as_deref())
            .push_bind(dto.pipe_type.as_deref())
            .push_bind(&dto.grade)
            .push_bind(dto.od)
            .push_bind(dto.wt)
            .push_bind(dto.length)
            .push_bind(dto.weight_per_unit)
            .push_bind(dto.end_type.as_deref())
            .push_bind(dto.coupling_type.as_deref())
            .push_bind(dto.coupling_od)
            .push_bind(dto.coupling_length)
            .push_bind(dto.heat_number.as_deref())
            .push_bind(dto.serial_number.as_deref())
            .push_bind(dto.manufacturer.as_deref())
            .push_bind(dto.production_date.as_deref())
            .push_bind(dto.cert_number.as_deref())
            .push_bind(None::<i64>) // location_id
            .push_bind(dto.notes.as_deref())
            .push_bind("new"); // status
    }

    fn build_update_query<'a>(builder: &mut QueryBuilder<'a, Sqlite>, dto: &'a Self::UpdateDto) {
        let mut first = true;
        if let Some(ref v) = dto.batch_number {
            if !first { builder.push(", "); }
            builder.push("batch_number = ").push_bind(v);
            first = false;
        }
        if let Some(ref v) = dto.pipe_type {
            if !first { builder.push(", "); }
            builder.push("pipe_type = ").push_bind(v);
            first = false;
        }
        if let Some(ref v) = dto.grade {
            if !first { builder.push(", "); }
            builder.push("grade = ").push_bind(v);
            first = false;
        }
        if let Some(v) = dto.od {
            if !first { builder.push(", "); }
            builder.push("od = ").push_bind(v);
            first = false;
        }
        if let Some(v) = dto.wt {
            if !first { builder.push(", "); }
            builder.push("wt = ").push_bind(v);
            first = false;
        }
        if let Some(v) = dto.length {
            if !first { builder.push(", "); }
            builder.push("length = ").push_bind(v);
            first = false;
        }
        if let Some(v) = dto.weight_per_unit {
            if !first { builder.push(", "); }
            builder.push("weight_per_unit = ").push_bind(v);
            first = false;
        }
        if let Some(ref v) = dto.end_type {
            if !first { builder.push(", "); }
            builder.push("end_type = ").push_bind(v);
            first = false;
        }
        if let Some(ref v) = dto.coupling_type {
            if !first { builder.push(", "); }
            builder.push("coupling_type = ").push_bind(v);
            first = false;
        }
        if let Some(v) = dto.coupling_od {
            if !first { builder.push(", "); }
            builder.push("coupling_od = ").push_bind(v);
            first = false;
        }
        if let Some(v) = dto.coupling_length {
            if !first { builder.push(", "); }
            builder.push("coupling_length = ").push_bind(v);
            first = false;
        }
        if let Some(ref v) = dto.heat_number {
            if !first { builder.push(", "); }
            builder.push("heat_number = ").push_bind(v);
            first = false;
        }
        if let Some(ref v) = dto.serial_number {
            if !first { builder.push(", "); }
            builder.push("serial_number = ").push_bind(v);
            first = false;
        }
        if let Some(ref v) = dto.manufacturer {
            if !first { builder.push(", "); }
            builder.push("manufacturer = ").push_bind(v);
            first = false;
        }
        if let Some(ref v) = dto.production_date {
            if !first { builder.push(", "); }
            builder.push("production_date = ").push_bind(v);
            first = false;
        }
        if let Some(ref v) = dto.cert_number {
            if !first { builder.push(", "); }
            builder.push("cert_number = ").push_bind(v);
            first = false;
        }
        if let Some(ref v) = dto.notes {
            if !first { builder.push(", "); }
            builder.push("notes = ").push_bind(v);
        }
    }
}