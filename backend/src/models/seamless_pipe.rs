use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Seamless pipe DB row. API 5CT standard master data — the real deal.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SeamlessPipe {
    pub id: i64,
    /// Pipe number — unique ID for each individual pipe.
    pub pipe_number: String,
    /// Batch number — groups pipes from the same production run.
    pub batch_number: Option<String>,
    /// Pipe type: casing or tubing. Choose your fighter.
    pub pipe_type: String,
    /// Steel grade: J55 / N80 / L80 / P110 — all the API 5CT classics.
    pub grade: String,
    /// Outer Diameter in mm. The big number.
    pub od: f64,
    /// Wall Thickness in mm. Skinny vs chunky.
    pub wt: f64,
    /// Length in meters. How long's your pipe?
    pub length: Option<f64>,
    /// Weight per unit length (kg/m).
    pub weight_per_unit: Option<f64>,
    /// End finish type: STC / LTC / BTC, etc.
    pub end_type: Option<String>,
    /// Coupling / collar type.
    pub coupling_type: Option<String>,
    /// Coupling outer diameter (mm).
    pub coupling_od: Option<f64>,
    /// Coupling length (mm).
    pub coupling_length: Option<f64>,
    /// Heat number / furnace batch.
    pub heat_number: Option<String>,
    /// Serial number within the heat.
    pub serial_number: Option<String>,
    /// Manufacturer name.
    pub manufacturer: Option<String>,
    /// Production date.
    pub production_date: Option<String>,
    /// Quality certificate number.
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
