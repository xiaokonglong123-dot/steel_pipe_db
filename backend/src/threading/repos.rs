//! Threading repositories.

use sqlx::PgPool;
use crate::models::threading::{ThreadGeometryCache, ThreadingRecord};

pub struct ThreadingRepo;

impl ThreadingRepo {
    pub async fn create_record(
        pool: &PgPool,
        tenant_id: i64,
        r: &ThreadingRecord,
    ) -> Result<ThreadingRecord, sqlx::Error> {
        sqlx::query_as::<_, ThreadingRecord>(
            "INSERT INTO threading_records \
             (tenant_id, pipe_id, pipe_number, thread_type, od, wt, grade, threads_per_inch, \
              pitch_diameter, makeup_torque, operator, notes) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) \
             RETURNING id, tenant_id, pipe_id, pipe_number, thread_type, od, wt, grade, \
                       threads_per_inch, pitch_diameter, makeup_torque, machined_at, \
                       operator, notes, created_at",
        )
        .bind(tenant_id)
        .bind(r.pipe_id)
        .bind(&r.pipe_number)
        .bind(&r.thread_type)
        .bind(r.od)
        .bind(r.wt)
        .bind(&r.grade)
        .bind(r.threads_per_inch)
        .bind(r.pitch_diameter)
        .bind(r.makeup_torque)
        .bind(r.operator)
        .bind(&r.notes)
        .fetch_one(pool)
        .await
    }

    pub async fn list_records(pool: &PgPool, tenant_id: i64, pipe_id: Option<i64>) -> Result<Vec<ThreadingRecord>, sqlx::Error> {
        sqlx::query_as::<_, ThreadingRecord>(
            "SELECT id, tenant_id, pipe_id, pipe_number, thread_type, od, wt, grade, \
                    threads_per_inch, pitch_diameter, makeup_torque, machined_at, \
                    operator, notes, created_at \
             FROM threading_records WHERE tenant_id = $1 \
             AND ($2::bigint IS NULL OR pipe_id = $2) ORDER BY id DESC LIMIT 500",
        )
        .bind(tenant_id)
        .bind(pipe_id)
        .fetch_all(pool)
        .await
    }

    pub async fn cache_get(
        pool: &PgPool,
        tenant_id: i64,
        pipe_type: &str,
        od: f64,
        wt: f64,
        grade: &str,
        connection_type: &str,
    ) -> Result<Option<ThreadGeometryCache>, sqlx::Error> {
        sqlx::query_as::<_, ThreadGeometryCache>(
            "SELECT id, tenant_id, pipe_type, od, wt, grade, connection_type, \
                    joint_efficiency, burst_pressure, collapse_pressure, tension_capacity, created_at \
             FROM thread_geometry_cache \
             WHERE tenant_id = $1 AND pipe_type = $2 AND od = $3 AND wt = $4 \
               AND grade = $5 AND connection_type = $6",
        )
        .bind(tenant_id)
        .bind(pipe_type)
        .bind(od)
        .bind(wt)
        .bind(grade)
        .bind(connection_type)
        .fetch_optional(pool)
        .await
    }

    pub async fn cache_put(
        pool: &PgPool,
        tenant_id: i64,
        pipe_type: &str,
        od: f64,
        wt: f64,
        grade: &str,
        connection_type: &str,
        joint_efficiency: f64,
        burst_pressure: f64,
        collapse_pressure: f64,
        tension_capacity: f64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO thread_geometry_cache \
             (tenant_id, pipe_type, od, wt, grade, connection_type, joint_efficiency, \
              burst_pressure, collapse_pressure, tension_capacity) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
             ON CONFLICT (pipe_type, od, wt, grade, connection_type) DO UPDATE SET \
               joint_efficiency = EXCLUDED.joint_efficiency, \
               burst_pressure = EXCLUDED.burst_pressure, \
               collapse_pressure = EXCLUDED.collapse_pressure, \
               tension_capacity = EXCLUDED.tension_capacity",
        )
        .bind(tenant_id)
        .bind(pipe_type)
        .bind(od)
        .bind(wt)
        .bind(grade)
        .bind(connection_type)
        .bind(joint_efficiency)
        .bind(burst_pressure)
        .bind(collapse_pressure)
        .bind(tension_capacity)
        .execute(pool)
        .await?;
        Ok(())
    }
}
