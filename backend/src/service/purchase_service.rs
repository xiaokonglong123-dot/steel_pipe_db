use std::sync::Arc;

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::domain::{
    Supplier, OrderStatus,
    PurchaseOrder, PurchaseOrderItem,
};
use crate::error::{AppError, AppResult};
use crate::repository::purchase_repo::{
    SupplierRepo,
    PurchaseOrderRepo, PurchaseOrderItemRepo,
};

// ── DTOs ────────────────────────────────────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
pub struct CreateSupplierDto {
    pub name: String,
    pub contact_person: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub cert_info: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateSupplierDto {
    pub name: Option<String>,
    pub contact_person: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub cert_info: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
pub struct PurchaseOrderItemDto {
    pub pipe_type: String,
    pub grade: String,
    pub od: f64,
    pub wt: f64,
    pub quantity: i32,
    pub unit_price: f64,
}

#[derive(Debug, serde::Deserialize)]
pub struct CreatePurchaseOrderDto {
    pub supplier_id: String,
    pub items: Vec<PurchaseOrderItemDto>,
    pub notes: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct OrderFilter {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub search: Option<String>,
    pub status: Option<String>,
    pub supplier_id: Option<String>,
    pub customer_id: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct InboundRef {
    pub inbound_id: String,
    pub inbound_no: String,
}

#[derive(Debug, serde::Serialize)]
pub struct PurchaseOrderWithSupplier {
    #[serde(flatten)]
    pub order: PurchaseOrder,
    pub supplier_name: Option<String>,
    pub operator_name: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct OrderDetail<T, U> {
    #[serde(flatten)]
    pub order: T,
    pub items: Vec<U>,
    pub party_name: Option<String>,
    pub operator_name: Option<String>,
    pub inbound_refs: Option<Vec<InboundRef>>,
    pub outbound_refs: Option<Vec<InboundRef>>,
}

// ── Order Number Generator ──────────────────────────────────────────────────

async fn generate_order_no(pool: &SqlitePool, prefix: &str, table: &str) -> AppResult<String> {
    let today = chrono::Local::now().format("%Y%m%d").to_string();
    let pattern = format!("{}-{}%", prefix, today);

    // Build a safe query — table and field are controlled constants, never user input
    let sql = format!(
        "SELECT {} FROM {} WHERE {} LIKE ? ORDER BY {} DESC LIMIT 1",
        "order_no", table, "order_no", "order_no"
    );
    let last: Option<(String,)> = sqlx::query_as(&sql)
        .bind(&pattern)
        .fetch_optional(pool)
        .await?;

    let seq = match last {
        Some((no,)) => {
            let parts: Vec<&str> = no.split('-').collect();
            let last_seq: i32 = parts.last()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            last_seq + 1
        }
        None => 1,
    };

    Ok(format!("{}-{}-{:03}", prefix, today, seq))
}

// ── Service ─────────────────────────────────────────────────────────────────

pub struct PurchaseService {
    pool: SqlitePool,
}

impl PurchaseService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn from_state(state: &Arc<crate::AppState>) -> Self {
        Self::new(state.db.clone())
    }

    // ── Supplier ────────────────────────────────────────────────────────────

    pub async fn create_supplier(&self, dto: CreateSupplierDto) -> AppResult<Supplier> {
        let supplier = Supplier {
            id: Uuid::new_v4().to_string(),
            name: dto.name,
            contact_person: dto.contact_person,
            phone: dto.phone,
            email: dto.email,
            address: dto.address,
            cert_info: dto.cert_info,
            is_active: dto.is_active.unwrap_or(true),
            created_at: String::new(),
            updated_at: String::new(),
        };
        SupplierRepo::create(&self.pool, &supplier).await?;

        // Fetch back for server-populated timestamps
        let result = SupplierRepo::find_by_id(&self.pool, &supplier.id).await?;
        Ok(result)
    }

    pub async fn list_suppliers(
        &self,
        search: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> AppResult<(Vec<Supplier>, i64)> {
        let total = SupplierRepo::count(&self.pool, search).await?;
        let items = SupplierRepo::list(&self.pool, search, page, page_size).await?;
        Ok((items, total))
    }

    pub async fn update_supplier(&self, id: &str, dto: UpdateSupplierDto) -> AppResult<Supplier> {
        let existing = SupplierRepo::find_by_id(&self.pool, id).await?;
        let updated = Supplier {
            id: existing.id,
            name: dto.name.unwrap_or(existing.name),
            contact_person: or_opt(dto.contact_person, existing.contact_person),
            phone: or_opt(dto.phone, existing.phone),
            email: or_opt(dto.email, existing.email),
            address: or_opt(dto.address, existing.address),
            cert_info: or_opt(dto.cert_info, existing.cert_info),
            is_active: dto.is_active.unwrap_or(existing.is_active),
            created_at: existing.created_at,
            updated_at: String::new(),
        };
        SupplierRepo::update(&self.pool, &updated).await?;
        SupplierRepo::find_by_id(&self.pool, id).await
    }

    pub async fn delete_supplier(&self, id: &str) -> AppResult<()> {
        SupplierRepo::delete(&self.pool, id).await
    }

    // ── Purchase Order ──────────────────────────────────────────────────────

    pub async fn create_purchase_order(
        &self,
        dto: CreatePurchaseOrderDto,
        operator_id: &str,
    ) -> AppResult<OrderDetail<PurchaseOrder, PurchaseOrderItem>> {
        let order_id = Uuid::new_v4().to_string();
        let order_no = generate_order_no(&self.pool, "PO", "purchase_orders").await?;

        let mut total_amount = 0.0_f64;
        let po_items: Vec<PurchaseOrderItem> = dto.items.iter().map(|item| {
            let subtotal = item.quantity as f64 * item.unit_price;
            total_amount += subtotal;
            PurchaseOrderItem {
                id: Uuid::new_v4().to_string(),
                order_id: order_id.clone(),
                pipe_type: item.pipe_type.clone(),
                grade: item.grade.clone(),
                od: item.od,
                wt: item.wt,
                quantity: item.quantity,
                received_quantity: 0,
                unit_price: item.unit_price,
                subtotal,
            }
        }).collect();

        let order = PurchaseOrder {
            id: order_id.clone(),
            order_no,
            supplier_id: dto.supplier_id.clone(),
            status: OrderStatus::Draft.as_str().to_string(),
            total_amount,
            notes: dto.notes,
            operator_id: operator_id.to_string(),
            created_at: String::new(),
            updated_at: String::new(),
        };

        // Validate supplier exists
        SupplierRepo::find_by_id(&self.pool, &dto.supplier_id).await?;

        PurchaseOrderRepo::create(&self.pool, &order, &po_items).await?;
        let (saved_order, _) = PurchaseOrderRepo::find_by_id(&self.pool, &order_id).await?;

        let supplier = SupplierRepo::find_by_id(&self.pool, &dto.supplier_id).await?;

        Ok(OrderDetail {
            order: saved_order,
            items: po_items,
            party_name: Some(supplier.name),
            operator_name: None,
            inbound_refs: Some(Vec::new()),
            outbound_refs: None,
        })
    }

    pub async fn approve_purchase_order(&self, id: &str) -> AppResult<PurchaseOrder> {
        let (order, _) = PurchaseOrderRepo::find_by_id(&self.pool, id).await?;
        match order.status.as_str() {
            "draft" => {
                PurchaseOrderRepo::update_status(&self.pool, id, "pending").await?;
            }
            "pending" => {
                PurchaseOrderRepo::update_status(&self.pool, id, "approved").await?;
            }
            _ => {
                return Err(AppError::BadRequest(format!(
                    "Cannot approve purchase order in status '{}'", order.status
                )));
            }
        }
        let (updated, _) = PurchaseOrderRepo::find_by_id(&self.pool, id).await?;
        Ok(updated)
    }

    pub async fn cancel_purchase_order(&self, id: &str) -> AppResult<PurchaseOrder> {
        let (order, _) = PurchaseOrderRepo::find_by_id(&self.pool, id).await?;
        match order.status.as_str() {
            "draft" | "pending" => {
                PurchaseOrderRepo::update_status(&self.pool, id, "cancelled").await?;
            }
            _ => {
                return Err(AppError::BadRequest(format!(
                    "Cannot cancel purchase order in status '{}'", order.status
                )));
            }
        }
        let (updated, _) = PurchaseOrderRepo::find_by_id(&self.pool, id).await?;
        Ok(updated)
    }

    pub async fn get_purchase_order(&self, id: &str) -> AppResult<OrderDetail<PurchaseOrder, PurchaseOrderItem>> {
        let (order, items) = PurchaseOrderRepo::find_by_id(&self.pool, id).await?;
        let supplier_name = SupplierRepo::find_by_id(&self.pool, &order.supplier_id).await
            .ok().map(|s| s.name);
        let operator_name = self.get_operator_name(&order.operator_id).await;
        let inbound_refs = self.get_inbound_refs_by_order(id).await?;
        Ok(OrderDetail {
            order,
            items,
            party_name: supplier_name,
            operator_name,
            inbound_refs: Some(inbound_refs),
            outbound_refs: None,
        })
    }

    pub async fn list_purchase_orders(
        &self,
        filter: &OrderFilter,
    ) -> AppResult<(Vec<PurchaseOrderWithSupplier>, i64)> {
        let page = filter.page.unwrap_or(1).max(1);
        let page_size = filter.page_size.unwrap_or(20).clamp(1, 200);
        let total = PurchaseOrderRepo::count(
            &self.pool,
            filter.status.as_deref(),
            filter.supplier_id.as_deref(),
        ).await?;
        let rows = PurchaseOrderRepo::list(
            &self.pool,
            filter.status.as_deref(),
            filter.supplier_id.as_deref(),
            page,
            page_size,
        ).await?;
        let items = rows.into_iter().map(|(po, name, op_name)| PurchaseOrderWithSupplier {
            order: po,
            supplier_name: name,
            operator_name: op_name,
        }).collect();
        Ok((items, total))
    }

    /// Links inbound items to a purchase order, incrementing received_quantity
    /// for each matching PO item based on the pipe's specs.
    pub async fn link_inbound_to_po(&self, inbound_id: &str, po_id: &str) -> AppResult<()> {
        // Verify PO exists
        let (_, items) = PurchaseOrderRepo::find_by_id(&self.pool, po_id).await?;

        // Get inbound items with pipe specs
        #[derive(sqlx::FromRow)]
        struct InboundPipeSpec {
            pipe_type: String,
            pipe_id: String,
        }

        let inbound_items: Vec<InboundPipeSpec> = sqlx::query_as(
            "SELECT pipe_type, pipe_id FROM inbound_items WHERE inbound_id = ?"
        )
        .bind(inbound_id)
        .fetch_all(&self.pool)
        .await?;

        for ii in &inbound_items {
            // Fetch pipe specs from the appropriate table
            let (grade, od, wt) = match ii.pipe_type.as_str() {
                "seamless" => {
                    let row: (String, f64, f64) = sqlx::query_as(
                        "SELECT grade, od, wt FROM seamless_pipes WHERE id = ?"
                    )
                    .bind(&ii.pipe_id)
                    .fetch_optional(&self.pool).await?
                    .ok_or_else(|| AppError::NotFound("Pipe not found".into()))?;
                    (row.0, row.1, row.2)
                }
                "screen" => {
                    let row: (String, f64, f64) = sqlx::query_as(
                        "SELECT grade, od, wt FROM screen_pipes WHERE id = ?"
                    )
                    .bind(&ii.pipe_id)
                    .fetch_optional(&self.pool).await?
                    .ok_or_else(|| AppError::NotFound("Pipe not found".into()))?;
                    (row.0, row.1, row.2)
                }
                _ => return Err(AppError::BadRequest(format!("Unknown pipe_type: {}", ii.pipe_type))),
            };

            // Find matching PO item
            let matching_items: Vec<PurchaseOrderItem> = items.iter()
                .filter(|i| i.pipe_type == ii.pipe_type && i.grade == grade
                    && (i.od - od).abs() < 0.001 && (i.wt - wt).abs() < 0.001)
                .cloned()
                .collect();

            if matching_items.is_empty() {
                return Err(AppError::BadRequest(format!(
                    "No purchase order item matches pipe {} ({} {} {}x{})",
                    ii.pipe_id, ii.pipe_type, grade, od, wt
                )));
            }

            // Increment received_quantity (use first match)
            let target = &matching_items[0];
            let new_qty = target.received_quantity + 1;
            PurchaseOrderRepo::update_received_quantity(&self.pool, &target.id, new_qty).await?;
        }

        // Auto-complete if all items fully received
        Self::auto_complete_po(&self.pool, po_id).await?;

        Ok(())
    }

    async fn auto_complete_po(pool: &SqlitePool, po_id: &str) -> AppResult<()> {
        let items = PurchaseOrderItemRepo::find_by_order(pool, po_id).await?;
        let all_received = items.iter().all(|i| i.received_quantity >= i.quantity);
        if all_received {
            PurchaseOrderRepo::update_status(pool, po_id, "completed").await?;
        }
        Ok(())
    }

    // ── Helpers ──────────────────────────────────────────────────────────────

    async fn get_operator_name(&self, operator_id: &str) -> Option<String> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT display_name FROM users WHERE id = ?"
        )
        .bind(operator_id)
        .fetch_optional(&self.pool).await
        .ok()?;
        row.map(|r| r.0)
    }

    async fn get_inbound_refs_by_order(&self, order_id: &str) -> AppResult<Vec<InboundRef>> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT id, inbound_no FROM inbound_records WHERE order_id = ?"
        )
        .bind(order_id)
        .fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|(id, no)| InboundRef {
            inbound_id: id,
            inbound_no: no,
        }).collect())
    }
}

fn or_opt<T>(new: Option<T>, existing: Option<T>) -> Option<T> {
    match new {
        Some(v) => Some(v),
        None => existing,
    }
}
