//! Portal services — portal login (Argon2 + JWT), party-scoped PO/SO views,
//! acceptance events. Reuses the same JWT secret as the main auth.

use argon2::{
    password_hash::{SaltString, PasswordHash, PasswordHasher, PasswordVerifier},
    Argon2,
};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::dto::portal_dto::{AcceptPurchaseRequest, CreatePortalAccountRequest, PortalLoginRequest};
use crate::error::AppError;
use crate::middleware::auth::JwtSecret;
use crate::models::portal::{PortalAccount, PortalEvent};
use crate::portal::repos::{PortalAccountRepo, PortalEventRepo};

pub struct PortalService;

impl PortalService {
    pub async fn create_account(
        pool: &SqlitePool,
        tenant_id: i64,
        dto: &CreatePortalAccountRequest,
    ) -> Result<PortalAccount, AppError> {
        if !matches!(dto.party_type.as_str(), "supplier" | "customer") {
            return Err(AppError::Validation(format!("Invalid party type: {}", dto.party_type)));
        }
        if PortalAccountRepo::find_by_username(pool, &dto.username).await?.is_some() {
            return Err(AppError::Validation(format!("Username '{}' already exists", dto.username)));
        }
        let salt = SaltString::encode_b64(Uuid::new_v4().as_bytes())
            .map_err(|_| AppError::Internal("Password hashing failed".into()))?;
        let hash = Argon2::default()
            .hash_password(dto.password.as_bytes(), &salt)
            .map_err(|_| AppError::Internal("Password hashing failed".into()))?
            .to_string();
        PortalAccountRepo::create(pool, tenant_id, &dto.party_type, dto.party_id, &dto.username, &hash)
            .await
            .map_err(AppError::from)
    }

    /// Portal login: verify Argon2, issue a JWT with party claims.
    pub async fn login(
        pool: &SqlitePool,
        dto: &PortalLoginRequest,
        jwt_secret: &JwtSecret,
    ) -> Result<(String, PortalAccount), AppError> {
        let account = PortalAccountRepo::find_by_username(pool, &dto.username)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Invalid username or password".into()))?;
        if !account.is_active {
            return Err(AppError::Unauthorized("Portal account is disabled".into()));
        }
        let parsed = PasswordHash::new(&account.password_hash)
            .map_err(|_| AppError::Internal("Corrupt password hash".into()))?;
        Argon2::default()
            .verify_password(dto.password.as_bytes(), &parsed)
            .map_err(|_| AppError::Unauthorized("Invalid username or password".into()))?;

        let claims = serde_json::json!({
            "sub": account.id,
            "username": account.username,
            "party_type": account.party_type,
            "party_id": account.party_id,
            "portal": true,
            "exp": chrono::Utc::now().timestamp() + 24 * 3600,
        });
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(jwt_secret.0.as_bytes()),
        )
        .map_err(|_| AppError::Internal("Token generation failed".into()))?;
        PortalAccountRepo::touch_login(pool, account.id).await.map_err(AppError::from)?;
        Ok((token, account))
    }

    /// Purchase orders visible to a supplier portal account.
    pub async fn supplier_purchases(pool: &SqlitePool, _tenant_id: i64, supplier_id: i64) -> Result<Vec<PortalPurchaseRow>, AppError> {
        sqlx::query_as::<_, PortalPurchaseRow>(
            "SELECT id, order_no, order_date, status, total_amount, notes \
             FROM purchase_orders WHERE supplier_id = ? AND deleted_at IS NULL \
             ORDER BY id DESC LIMIT 200",
        )
        .bind(supplier_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }

    /// Supplier accepts a PO (records portal_event; updates PO status to approved).
    pub async fn accept_purchase(
        pool: &SqlitePool,
        tenant_id: i64,
        supplier_id: i64,
        purchase_order_id: i64,
        dto: &AcceptPurchaseRequest,
    ) -> Result<PortalEvent, AppError> {
        // PO must belong to this supplier and be pending.
        let owned: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM purchase_orders WHERE id = ? AND supplier_id = ? AND deleted_at IS NULL",
        )
        .bind(purchase_order_id)
        .bind(supplier_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
        if owned == 0 {
            return Err(AppError::NotFound(format!("Purchase order not found for this supplier: {}", purchase_order_id)));
        }
        sqlx::query("UPDATE purchase_orders SET status = 'approved', updated_at = datetime('now') WHERE id = ?")
            .bind(purchase_order_id)
            .execute(pool)
            .await
            .map_err(AppError::from)?;
        PortalEventRepo::create(pool, tenant_id, "supplier", supplier_id, "po_accepted", purchase_order_id, dto.notes.as_deref())
            .await
            .map_err(AppError::from)
    }

    /// Sales orders visible to a customer portal account.
    pub async fn customer_sales(pool: &SqlitePool, _tenant_id: i64, customer_id: i64) -> Result<Vec<PortalSalesRow>, AppError> {
        sqlx::query_as::<_, PortalSalesRow>(
            "SELECT id, order_no, order_date, status, total_amount, notes \
             FROM sales_orders WHERE customer_id = ? AND deleted_at IS NULL \
             ORDER BY id DESC LIMIT 200",
        )
        .bind(customer_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }

    /// Customer acknowledges a sales order (records event).
    pub async fn acknowledge_sales(
        pool: &SqlitePool,
        tenant_id: i64,
        customer_id: i64,
        sales_order_id: i64,
    ) -> Result<PortalEvent, AppError> {
        let owned: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sales_orders WHERE id = ? AND customer_id = ? AND deleted_at IS NULL",
        )
        .bind(sales_order_id)
        .bind(customer_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
        if owned == 0 {
            return Err(AppError::NotFound(format!("Sales order not found for this customer: {}", sales_order_id)));
        }
        PortalEventRepo::create(pool, tenant_id, "customer", customer_id, "so_acknowledged", sales_order_id, None)
            .await
            .map_err(AppError::from)
    }

    pub async fn events(pool: &SqlitePool, tenant_id: i64, party_type: &str, party_id: i64) -> Result<Vec<PortalEvent>, AppError> {
        PortalEventRepo::list(pool, tenant_id, party_type, party_id).await.map_err(AppError::from)
    }
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct PortalPurchaseRow {
    pub id: i64,
    pub order_no: String,
    pub order_date: chrono::DateTime<chrono::Utc>,
    pub status: String,
    pub total_amount: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct PortalSalesRow {
    pub id: i64,
    pub order_no: String,
    pub order_date: chrono::DateTime<chrono::Utc>,
    pub status: String,
    pub total_amount: Option<f64>,
    pub notes: Option<String>,
}
