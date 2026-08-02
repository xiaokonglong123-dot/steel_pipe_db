use chrono::{DateTime, Utc};
use crate::domain::pipe::{PipeModel, PipeType};
use crate::dto::pipe_dto::{CreateScreenPipeRequest, UpdateScreenPipeRequest, PipeFilterParams, PipeSearchResult};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, QueryBuilder, Postgres};

/// Screen pipe DB row. Sand-control screens.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScreenPipe {
    pub id: i64,
    pub pipe_number: String,
    pub batch_number: Option<String>,
    pub screen_type: String,
    pub slot_size: Option<f64>,
    pub filtration_grade: Option<String>,
    pub base_od: f64,
    pub base_wt: f64,
    pub base_grade: String,
    pub base_end_type: Option<String>,
    pub length: Option<f64>,
    pub weight_per_unit: Option<f64>,
    pub heat_number: Option<String>,
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
    pub production_date: Option<String>,
    pub cert_number: Option<String>,
    pub location_id: Option<i64>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<String>,
}

impl PipeModel for ScreenPipe {
    type CreateDto = CreateScreenPipeRequest;
    type UpdateDto = UpdateScreenPipeRequest;
    type FilterParams = PipeFilterParams;

    const TABLE_NAME: &'static str = "screen_pipes";
    const PIPE_TYPE: PipeType = PipeType::Screen;
    const GRADE_COLUMN: &'static str = "base_grade";
    const OD_COLUMN: &'static str = "base_od";
    const WT_COLUMN: &'static str = "base_wt";
    const PIPE_TYPE_COLUMN: &'static str = "screen_type";

    fn pipe_number(&self) -> &str { &self.pipe_number }
    fn status(&self) -> &str { &self.status }
    fn set_status(&mut self, status: &str) { self.status = status.to_string(); }
    fn id(&self) -> i64 { self.id }
    fn location_id(&self) -> Option<i64> { self.location_id }
    fn set_location_id(&mut self, location_id: Option<i64>) { self.location_id = location_id; }
    fn grade(&self) -> &str { &self.base_grade }
    fn od(&self) -> f64 { self.base_od }
    fn wt(&self) -> f64 { self.base_wt }
    fn pipe_type_str(&self) -> &str { "screen" }
    fn batch_number(&self) -> Option<&str> { self.batch_number.as_deref() }
    fn heat_number(&self) -> Option<&str> { self.heat_number.as_deref() }
    fn serial_number(&self) -> Option<&str> { self.serial_number.as_deref() }
    fn manufacturer(&self) -> Option<&str> { self.manufacturer.as_deref() }
    fn deleted_at(&self) -> Option<&str> { self.deleted_at.as_deref() }

    fn valid_sort_column(col: &str) -> Option<&'static str> {
        match col {
            "pipe_number" => Some("pipe_number"),
            "base_grade" => Some("base_grade"),
            "base_od" => Some("base_od"),
            "base_wt" => Some("base_wt"),
            "status" => Some("status"),
            "manufacturer" => Some("manufacturer"),
            "production_date" => Some("production_date"),
            _ => None,
        }
    }

    fn generate_pipe_number(grade: &str, od: f64, wt: f64) -> String {
        let prefix = "SCP";
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

    fn build_create_query<'a>(builder: &mut QueryBuilder<'a, Postgres>, dto: &'a Self::CreateDto) {
        builder
            .push(", screen_type, slot_size, filtration_grade, base_od, base_wt, \
                  base_grade, base_end_type, length, weight_per_unit, heat_number, \
                  serial_number, manufacturer, production_date, cert_number, \
                  location_id, notes, status) VALUES (");
        let mut sep = builder.separated(", ");
        sep.push_bind(dto.pipe_number.as_deref())
            .push_bind(dto.batch_number.as_deref())
            .push_bind(dto.screen_type.as_deref())
            .push_bind(dto.slot_size)
            .push_bind(dto.filtration_grade.as_deref())
            .push_bind(dto.base_od)
            .push_bind(dto.base_wt)
            .push_bind(dto.base_grade.as_str())
            .push_bind(dto.base_end_type.as_deref())
            .push_bind(dto.length)
            .push_bind(dto.weight_per_unit)
            .push_bind(dto.heat_number.as_deref())
            .push_bind(dto.serial_number.as_deref())
            .push_bind(dto.manufacturer.as_deref())
            .push_bind(dto.production_date.as_deref())
            .push_bind(dto.cert_number.as_deref())
            .push_bind(None::<i64>) // location_id
            .push_bind(dto.notes.as_deref())
            .push_bind("new"); // status
    }

    fn build_update_query<'a>(builder: &mut QueryBuilder<'a, Postgres>, dto: &'a Self::UpdateDto) {
        let mut first = true;
        if let Some(ref v) = dto.batch_number {
            if !first { builder.push(", "); }
            builder.push("batch_number = ").push_bind(v);
            first = false;
        }
        if let Some(ref v) = dto.screen_type {
            if !first { builder.push(", "); }
            builder.push("screen_type = ").push_bind(v);
            first = false;
        }
        if let Some(v) = dto.slot_size {
            if !first { builder.push(", "); }
            builder.push("slot_size = ").push_bind(v);
            first = false;
        }
        if let Some(ref v) = dto.filtration_grade {
            if !first { builder.push(", "); }
            builder.push("filtration_grade = ").push_bind(v);
            first = false;
        }
        if let Some(v) = dto.base_od {
            if !first { builder.push(", "); }
            builder.push("base_od = ").push_bind(v);
            first = false;
        }
        if let Some(v) = dto.base_wt {
            if !first { builder.push(", "); }
            builder.push("base_wt = ").push_bind(v);
            first = false;
        }
        if let Some(ref v) = dto.base_grade {
            if !first { builder.push(", "); }
            builder.push("base_grade = ").push_bind(v);
            first = false;
        }
        if let Some(ref v) = dto.base_end_type {
            if !first { builder.push(", "); }
            builder.push("base_end_type = ").push_bind(v);
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
        if let Some(v) = dto.location_id {
            if !first { builder.push(", "); }
            builder.push("location_id = ").push_bind(v);
        }
    }
}