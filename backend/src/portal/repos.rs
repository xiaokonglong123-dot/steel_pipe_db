//! Portal repositories.

use sqlx::SqlitePool;
use crate::models::portal::{PortalAccount, PortalEvent};

pub struct PortalAccountRepo;

impl PortalAccountRepo {
    pub async fn create(
        pool: &SqlitePool,
        tenant_id: i64,
        party_type: &str,
        party_id: i64,
        username: &str,
        password_hash: &str,
    ) -> Result<PortalAccount, sqlx::Error> {
        sqlx::query_as::<_, PortalAccount>(
            "INSERT INTO portal_accounts (tenant_id, party_type, party_id, username, password_hash) \
             VALUES (?, ?, ?, ?, ?) \
             RETURNING id, tenant_id, party_type, party_id, username, password_hash, is_active, last_login_at, created_at",
        )
        .bind(tenant_id)
        .bind(party_type)
        .bind(party_id)
        .bind(username)
        .bind(password_hash)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_username(pool: &SqlitePool, username: &str) -> Result<Option<PortalAccount>, sqlx::Error> {
        sqlx::query_as::<_, PortalAccount>(
            "SELECT id, tenant_id, party_type, party_id, username, password_hash, is_active, last_login_at, created_at \
             FROM portal_accounts WHERE username = ?",
        )
        .bind(username)
        .fetch_optional(pool)
        .await
    }

    pub async fn touch_login(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE portal_accounts SET last_login_at = datetime('now') WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
}

pub struct PortalEventRepo;

impl PortalEventRepo {
    pub async fn create(
        pool: &SqlitePool,
        tenant_id: i64,
        party_type: &str,
        party_id: i64,
        event_type: &str,
        ref_id: i64,
        notes: Option<&str>,
    ) -> Result<PortalEvent, sqlx::Error> {
        sqlx::query_as::<_, PortalEvent>(
            "INSERT INTO portal_events (tenant_id, party_type, party_id, event_type, ref_id, notes) \
             VALUES (?, ?, ?, ?, ?, ?) \
             RETURNING id, tenant_id, party_type, party_id, event_type, ref_id, notes, created_at",
        )
        .bind(tenant_id)
        .bind(party_type)
        .bind(party_id)
        .bind(event_type)
        .bind(ref_id)
        .bind(notes)
        .fetch_one(pool)
        .await
    }

    pub async fn list(pool: &SqlitePool, tenant_id: i64, party_type: &str, party_id: i64) -> Result<Vec<PortalEvent>, sqlx::Error> {
        sqlx::query_as::<_, PortalEvent>(
            "SELECT id, tenant_id, party_type, party_id, event_type, ref_id, notes, created_at \
             FROM portal_events WHERE tenant_id = ? AND party_type = ? AND party_id = ? \
             ORDER BY id DESC LIMIT 200",
        )
        .bind(tenant_id)
        .bind(party_type)
        .bind(party_id)
        .fetch_all(pool)
        .await
    }
}
