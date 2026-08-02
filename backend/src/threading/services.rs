//! Threading services — API 5CT engineering calculations.
//!
//! Formulas follow API Bulletin 5C3 / Barlow:
//! - Burst pressure (Barlow): 0.875 * 2 * YS * wt / od
//! - Joint efficiency: round ~0.85, buttress ~0.95, premium ~1.0
//! - Collapse: API 5C3 empirical regime approximation for typical casing
//! - Tension capacity: cross-section area * YS * joint_efficiency

use sqlx::PgPool;

use crate::dto::threading_dto::{
    CreateThreadingRecordRequest, DesignCheckRequest, ThreadCalcRequest,
};
use crate::error::AppError;
use crate::models::threading::ThreadingRecord;
use crate::threading::repos::ThreadingRepo;

pub struct ThreadingService;

impl ThreadingService {
    // -----------------------------------------------------------------------
    // Records
    // -----------------------------------------------------------------------

    pub async fn create_record(
        pool: &PgPool,
        tenant_id: i64,
        dto: &CreateThreadingRecordRequest,
        operator: Option<i64>,
    ) -> Result<ThreadingRecord, AppError> {
        if dto.od <= 0.0 || dto.wt <= 0.0 || dto.wt >= dto.od / 2.0 {
            return Err(AppError::Validation(format!(
                "Invalid geometry: od={} wt={}",
                dto.od, dto.wt
            )));
        }
        let record = ThreadingRecord {
            id: 0,
            tenant_id,
            pipe_id: dto.pipe_id,
            pipe_number: dto.pipe_number.clone(),
            thread_type: dto.thread_type.clone(),
            od: dto.od,
            wt: dto.wt,
            grade: dto.grade.clone(),
            threads_per_inch: dto.threads_per_inch,
            pitch_diameter: dto.pitch_diameter,
            makeup_torque: dto.makeup_torque,
            machined_at: chrono::Utc::now(),
            operator,
            notes: dto.notes.clone(),
            created_at: chrono::Utc::now(),
        };
        ThreadingRepo::create_record(pool, tenant_id, &record)
            .await
            .map_err(AppError::from)
    }

    pub async fn list_records(pool: &PgPool, tenant_id: i64, pipe_id: Option<i64>) -> Result<Vec<ThreadingRecord>, AppError> {
        ThreadingRepo::list_records(pool, tenant_id, pipe_id).await.map_err(AppError::from)
    }

    // -----------------------------------------------------------------------
    // Engineering calculations
    // -----------------------------------------------------------------------

    fn yield_strength(grade: &str) -> f64 {
        match grade.to_uppercase().as_str() {
            "H40" => 40_000.0,
            "J55" => 55_000.0,
            "K55" => 55_000.0,
            "N80" => 80_000.0,
            "L80" => 80_000.0,
            "C90" => 90_000.0,
            "C95" => 95_000.0,
            "T95" => 95_000.0,
            "P110" => 110_000.0,
            "Q125" => 125_000.0,
            _ => 55_000.0, // conservative default
        }
    }

    fn joint_efficiency(connection_type: &str) -> f64 {
        match connection_type.to_lowercase().as_str() {
            "premium" => 1.0,
            "buttress" => 0.95,
            _ => 0.85, // API round thread
        }
    }

    /// Burst pressure via Barlow (psi), with 87.5% wall allowance.
    fn burst_pressure(od: f64, wt: f64, grade: &str) -> f64 {
        0.875 * 2.0 * Self::yield_strength(grade) * wt / od
    }

    /// API 5C3 collapse pressure (psi) using the standard regime formulas.
    fn collapse_pressure(od: f64, wt: f64, grade: &str) -> f64 {
        let d_over_t = od / wt;
        let ys = Self::yield_strength(grade);
        if d_over_t < 14.0 {
            // Yield-strength collapse regime: Pc = 2*YS*(D/t - 1)/(D/t)^2
            (2.0 * ys * (d_over_t - 1.0)) / (d_over_t * d_over_t)
        } else if d_over_t < 25.0 {
            // Plastic collapse regime: Pc = YS*(A/(D/t) - B) - C  (A=2.876, B=0.026233, C=1955)
            ys * (2.876 / d_over_t - 0.026233) - 1955.0
        } else {
            // Transition regime (conservative lower bound)
            800_000.0 / (d_over_t * (d_over_t - 1.0))
        }
    }

    /// Tension capacity (lbs): cross-section area × YS × joint efficiency.
    fn tension_capacity(od: f64, wt: f64, grade: &str, connection_type: &str) -> f64 {
        let area = std::f64::consts::PI / 4.0 * (od * od - (od - 2.0 * wt) * (od - 2.0 * wt));
        area * Self::yield_strength(grade) * Self::joint_efficiency(connection_type)
    }

    /// Full calc result with caching by (od, wt, grade, connection).
    pub async fn calc(pool: &PgPool, tenant_id: i64, dto: &ThreadCalcRequest) -> Result<CalcResult, AppError> {
        // Cache lookup first.
        if let Some(c) = ThreadingRepo::cache_get(
            pool, tenant_id, "casing", dto.od, dto.wt, &dto.grade, &dto.connection_type,
        )
        .await
        .map_err(AppError::from)?
        {
            return Ok(CalcResult {
                od: dto.od,
                wt: dto.wt,
                grade: dto.grade.clone(),
                connection_type: dto.connection_type.clone(),
                joint_efficiency: c.joint_efficiency,
                burst_pressure: c.burst_pressure,
                collapse_pressure: c.collapse_pressure,
                tension_capacity: c.tension_capacity,
                cached: true,
            });
        }

        let result = CalcResult {
            od: dto.od,
            wt: dto.wt,
            grade: dto.grade.clone(),
            connection_type: dto.connection_type.clone(),
            joint_efficiency: Self::joint_efficiency(&dto.connection_type),
            burst_pressure: Self::burst_pressure(dto.od, dto.wt, &dto.grade),
            collapse_pressure: Self::collapse_pressure(dto.od, dto.wt, &dto.grade),
            tension_capacity: Self::tension_capacity(dto.od, dto.wt, &dto.grade, &dto.connection_type),
            cached: false,
        };
        ThreadingRepo::cache_put(
            pool, tenant_id, "casing", dto.od, dto.wt, &dto.grade, &dto.connection_type,
            result.joint_efficiency, result.burst_pressure, result.collapse_pressure,
            result.tension_capacity,
        )
        .await
        .map_err(AppError::from)?;
        Ok(result)
    }

    /// Casing design check: safety factors at depth against burst/collapse
    /// and connection tension.
    pub async fn design_check(
        pool: &PgPool,
        tenant_id: i64,
        dto: &DesignCheckRequest,
    ) -> Result<DesignCheckOutput, AppError> {
        let calc = Self::calc(
            pool,
            tenant_id,
            &ThreadCalcRequest {
                od: dto.od,
                wt: dto.wt,
                grade: dto.grade.clone(),
                connection_type: dto.connection_type.clone(),
            },
        )
        .await?;

        // External pressure at depth (hydrostatic): P = ρ g h (Pa → psi).
        let rho = if dto.fluid_density > 0.0 { dto.fluid_density } else { 1025.0 };
        let external_psi = rho * 9.81 * dto.depth / 6894.76;

        // Internal burst margin: burst / (external + assumed 1.1× derating).
        let burst_sf = calc.burst_pressure / (external_psi.max(1.0));
        let collapse_sf = calc.collapse_pressure / external_psi.max(1.0);
        // Tension: weight of pipe below this joint (approx per meter).
        let area_m2 = std::f64::consts::PI / 4.0 * ((dto.od / 1000.0).powi(2) - ((dto.od - 2.0 * dto.wt) / 1000.0).powi(2));
        let weight_per_m_kg = area_m2 * 7850.0;
        let tension_lbs = weight_per_m_kg * dto.depth * 2.20462;
        let tension_sf = calc.tension_capacity / tension_lbs.max(1.0);

        Ok(DesignCheckOutput {
            depth: dto.depth,
            external_pressure_psi: external_psi,
            burst_safety_factor: burst_sf,
            collapse_safety_factor: collapse_sf,
            tension_safety_factor: tension_sf,
            joint_strength: calc,
            verdict: if burst_sf >= 1.25 && collapse_sf >= 1.1 && tension_sf >= 1.6 {
                "safe".to_string()
            } else {
                "unsafe".to_string()
            },
        })
    }
}

#[derive(Debug, serde::Serialize)]
pub struct CalcResult {
    pub od: f64,
    pub wt: f64,
    pub grade: String,
    pub connection_type: String,
    pub joint_efficiency: f64,
    pub burst_pressure: f64,
    pub collapse_pressure: f64,
    pub tension_capacity: f64,
    pub cached: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct DesignCheckOutput {
    pub depth: f64,
    pub external_pressure_psi: f64,
    pub burst_safety_factor: f64,
    pub collapse_safety_factor: f64,
    pub tension_safety_factor: f64,
    pub joint_strength: CalcResult,
    pub verdict: String,
}
