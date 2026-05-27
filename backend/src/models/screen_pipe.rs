use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Screen pipe DB row. Sand-control screens — wire-wrapped, slotted, or premium.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScreenPipe {
    pub id: i64,
    /// Pipe number — unique per screen pipe.
    pub pipe_number: String,
    /// Batch / heat number.
    pub batch_number: Option<String>,
    /// Screen type: wire_wrapped / slotted / premium.
    pub screen_type: String,
    /// Slot width or aperture (mm).
    pub slot_size: Option<f64>,
    /// Filtration grade rating.
    pub filtration_grade: Option<String>,
    /// Base pipe outer diameter (mm).
    pub base_od: f64,
    /// Base pipe wall thickness (mm).
    pub base_wt: f64,
    /// Base pipe steel grade.
    pub base_grade: String,
    /// Base pipe end finish type.
    pub base_end_type: Option<String>,
    /// Length in meters.
    pub length: Option<f64>,
    /// Weight per unit length (kg/m).
    pub weight_per_unit: Option<f64>,
    /// Heat / furnace number.
    pub heat_number: Option<String>,
    /// Serial number within heat.
    pub serial_number: Option<String>,
    /// Manufacturer name.
    pub manufacturer: Option<String>,
    /// Production date.
    pub production_date: Option<String>,
    /// Quality cert number.
    pub cert_number: Option<String>,
    /// Current warehouse location ID.
    pub location_id: Option<i64>,
    /// Stock status: in_stock / outbound / scrapped / in_transit / reserved.
    pub status: String,
    /// Free-form notes.
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}
