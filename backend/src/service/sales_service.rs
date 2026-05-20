use std::sync::Arc;

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::domain::{
    Customer, OrderStatus,
    SalesOrder, SalesOrderItem,
};
use crate::error::{AppError, AppResult};
use crate::repository::sales_repo::{
    CustomerRepo,
    SalesOrderRepo, SalesOrderItemRepo,
};

// ── DTOs ────────────────────────────────────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
pub struct CreateCustomerDto {
    pub name: String,
    pub contact_person: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateCustomerDto {
    pub name: Option<String>,
    pub contact_person: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
pub struct SalesOrderItemDto {
    pub pipe_type: String,
    pub grade: String,
    pub od: f64,
    pub wt: f64,
    pub quantity: i32,
    pub unit_price: f64,
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateSalesOrderDto {
    pub customer_id: String,
    pub items: Vec<SalesOrderItemDto>,
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

#[derive(Debug, serde::Deserialize)]
pub struct AtpQuery {
    pub pipe_type: String,
    pub grade: String,
    pub od: f64,
    pub wt: f64,
}

#[derive(Debug, serde::Serialize)]
pub struct OutboundRef {
    pub outbound_id: String,
    pub outbound_no: String,
}

#[derive(Debug, serde::Serialize)]
pub struct AtpResult {
    pub pipe_type: String,
    pub grade: String,
    pub od: f64,
    pub wt: f64,
    pub in_stock: i64,
    pub allocated: i64,
    pub available: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct SalesOrderWithCustomer {
    #[serde(flatten)]
    pub order: SalesOrder,
    pub customer_name: Option<String>,
    pub operator_name: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct OrderDetail<T, U> {
    #[serde(flatten)]
    pub order: T,
    pub items: Vec<U>,
    pub party_name: Option<String>,
    pub operator_name: Option<String>,
    pub inbound_refs: Option<Vec<OutboundRef>>,
    pub outbound_refs: Option<Vec<OutboundRef>>,
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

pub struct SalesService {
    pool: SqlitePool,
}

impl SalesService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn from_state(state: &Arc<crate::AppState>) -> Self {
        Self::new(state.db.clone())
    }

    // ── Customer ────────────────────────────────────────────────────────────

    pub async fn create_customer(&self, dto: CreateCustomerDto) -> AppResult<Customer> {
        let customer = Customer {
            id: Uuid::new_v4().to_string(),
            name: dto.name,
            contact_person: dto.contact_person,
            phone: dto.phone,
            email: dto.email,
            address: dto.address,
            is_active: dto.is_active.unwrap_or(true),
            created_at: String::new(),
            updated_at: String::new(),
        };
        CustomerRepo::create(&self.pool, &customer).await?;
        let result = CustomerRepo::find_by_id(&self.pool, &customer.id).await?;
        Ok(result)
    }

    pub async fn list_customers(
        &self,
        search: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> AppResult<(Vec<Customer>, i64)> {
        let total = CustomerRepo::count(&self.pool, search).await?;
        let items = CustomerRepo::list(&self.pool, search, page, page_size).await?;
        Ok((items, total))
    }

    pub async fn update_customer(&self, id: &str, dto: UpdateCustomerDto) -> AppResult<Customer> {
        let existing = CustomerRepo::find_by_id(&self.pool, id).await?;
        let updated = Customer {
            id: existing.id,
            name: dto.name.unwrap_or(existing.name),
            contact_person: or_opt(dto.contact_person, existing.contact_person),
            phone: or_opt(dto.phone, existing.phone),
            email: or_opt(dto.email, existing.email),
            address: or_opt(dto.address, existing.address),
            is_active: dto.is_active.unwrap_or(existing.is_active),
            created_at: existing.created_at,
            updated_at: String::new(),
        };
        CustomerRepo::update(&self.pool, &updated).await?;
        CustomerRepo::find_by_id(&self.pool, id).await
    }

    pub async fn delete_customer(&self, id: &str) -> AppResult<()> {
        CustomerRepo::delete(&self.pool, id).await
    }

    // ── Sales Order ─────────────────────────────────────────────────────────

    pub async fn create_sales_order(
        &self,
        dto: CreateSalesOrderDto,
        operator_id: &str,
    ) -> AppResult<OrderDetail<SalesOrder, SalesOrderItem>> {
        // Validate customer exists
        CustomerRepo::find_by_id(&self.pool, &dto.customer_id).await?;

        // Validate ATP for each item
        for item in &dto.items {
            let atp = self.compute_atp(&item.pipe_type, &item.grade, item.od, item.wt).await?;
            if atp.available < item.quantity as i64 {
                return Err(AppError::BadRequest(format!(
                    "Insufficient stock for {} {} {}x{}: requested {}, available {}",
                    item.pipe_type, item.grade, item.od, item.wt,
                    item.quantity, atp.available
                )));
            }
        }

        let order_id = Uuid::new_v4().to_string();
        let order_no = generate_order_no(&self.pool, "SO", "sales_orders").await?;

        let mut total_amount = 0.0_f64;
        let so_items: Vec<SalesOrderItem> = dto.items.iter().map(|item| {
            let subtotal = item.quantity as f64 * item.unit_price;
            total_amount += subtotal;
            SalesOrderItem {
                id: Uuid::new_v4().to_string(),
                order_id: order_id.clone(),
                pipe_type: item.pipe_type.clone(),
                grade: item.grade.clone(),
                od: item.od,
                wt: item.wt,
                quantity: item.quantity,
                delivered_quantity: 0,
                unit_price: item.unit_price,
                subtotal,
            }
        }).collect();

        let order = SalesOrder {
            id: order_id.clone(),
            order_no,
            customer_id: dto.customer_id.clone(),
            status: OrderStatus::Draft.as_str().to_string(),
            total_amount,
            notes: dto.notes,
            operator_id: operator_id.to_string(),
            created_at: String::new(),
            updated_at: String::new(),
        };

        SalesOrderRepo::create(&self.pool, &order, &so_items).await?;
        let (saved_order, _) = SalesOrderRepo::find_by_id(&self.pool, &order_id).await?;

        let customer = CustomerRepo::find_by_id(&self.pool, &dto.customer_id).await?;

        Ok(OrderDetail {
            order: saved_order,
            items: so_items,
            party_name: Some(customer.name),
            operator_name: None,
            inbound_refs: None,
            outbound_refs: Some(Vec::new()),
        })
    }

    pub async fn approve_sales_order(&self, id: &str) -> AppResult<SalesOrder> {
        let (order, _) = SalesOrderRepo::find_by_id(&self.pool, id).await?;
        match order.status.as_str() {
            "draft" => {
                SalesOrderRepo::update_status(&self.pool, id, "pending").await?;
            }
            "pending" => {
                SalesOrderRepo::update_status(&self.pool, id, "approved").await?;
            }
            _ => {
                return Err(AppError::BadRequest(format!(
                    "Cannot approve sales order in status '{}'", order.status
                )));
            }
        }
        let (updated, _) = SalesOrderRepo::find_by_id(&self.pool, id).await?;
        Ok(updated)
    }

    pub async fn cancel_sales_order(&self, id: &str) -> AppResult<SalesOrder> {
        let (order, _) = SalesOrderRepo::find_by_id(&self.pool, id).await?;
        match order.status.as_str() {
            "draft" | "pending" => {
                SalesOrderRepo::update_status(&self.pool, id, "cancelled").await?;
            }
            _ => {
                return Err(AppError::BadRequest(format!(
                    "Cannot cancel sales order in status '{}'", order.status
                )));
            }
        }
        let (updated, _) = SalesOrderRepo::find_by_id(&self.pool, id).await?;
        Ok(updated)
    }

    pub async fn get_sales_order(&self, id: &str) -> AppResult<OrderDetail<SalesOrder, SalesOrderItem>> {
        let (order, items) = SalesOrderRepo::find_by_id(&self.pool, id).await?;
        let customer_name = CustomerRepo::find_by_id(&self.pool, &order.customer_id).await
            .ok().map(|c| c.name);
        let operator_name = self.get_operator_name(&order.operator_id).await;
        let outbound_refs = self.get_outbound_refs_by_order(id).await?;
        Ok(OrderDetail {
            order,
            items,
            party_name: customer_name,
            operator_name,
            inbound_refs: None,
            outbound_refs: Some(outbound_refs),
        })
    }

    pub async fn list_sales_orders(
        &self,
        filter: &OrderFilter,
    ) -> AppResult<(Vec<SalesOrderWithCustomer>, i64)> {
        let page = filter.page.unwrap_or(1).max(1);
        let page_size = filter.page_size.unwrap_or(20).clamp(1, 200);
        let total = SalesOrderRepo::count(
            &self.pool,
            filter.status.as_deref(),
            filter.customer_id.as_deref(),
        ).await?;
        let rows = SalesOrderRepo::list(
            &self.pool,
            filter.status.as_deref(),
            filter.customer_id.as_deref(),
            page,
            page_size,
        ).await?;
        let items = rows.into_iter().map(|(so, name, op_name)| SalesOrderWithCustomer {
            order: so,
            customer_name: name,
            operator_name: op_name,
        }).collect();
        Ok((items, total))
    }

    /// Links outbound items to a sales order, incrementing delivered_quantity.
    pub async fn link_outbound_to_so(&self, outbound_id: &str, so_id: &str) -> AppResult<()> {
        let (_, items) = SalesOrderRepo::find_by_id(&self.pool, so_id).await?;

        #[derive(sqlx::FromRow)]
        struct OutboundPipeSpec {
            pipe_type: String,
            pipe_id: String,
        }

        let outbound_items: Vec<OutboundPipeSpec> = sqlx::query_as(
            "SELECT pipe_type, pipe_id FROM outbound_items WHERE outbound_id = ?"
        )
        .bind(outbound_id)
        .fetch_all(&self.pool)
        .await?;

        for oi in &outbound_items {
            let (grade, od, wt) = match oi.pipe_type.as_str() {
                "seamless" => {
                    let row: (String, f64, f64) = sqlx::query_as(
                        "SELECT grade, od, wt FROM seamless_pipes WHERE id = ?"
                    )
                    .bind(&oi.pipe_id)
                    .fetch_optional(&self.pool).await?
                    .ok_or_else(|| AppError::NotFound("Pipe not found".into()))?;
                    (row.0, row.1, row.2)
                }
                "screen" => {
                    let row: (String, f64, f64) = sqlx::query_as(
                        "SELECT grade, od, wt FROM screen_pipes WHERE id = ?"
                    )
                    .bind(&oi.pipe_id)
                    .fetch_optional(&self.pool).await?
                    .ok_or_else(|| AppError::NotFound("Pipe not found".into()))?;
                    (row.0, row.1, row.2)
                }
                _ => return Err(AppError::BadRequest(format!("Unknown pipe_type: {}", oi.pipe_type))),
            };

            let matching_items: Vec<SalesOrderItem> = items.iter()
                .filter(|i| i.pipe_type == oi.pipe_type && i.grade == grade
                    && (i.od - od).abs() < 0.001 && (i.wt - wt).abs() < 0.001)
                .cloned()
                .collect();

            if matching_items.is_empty() {
                return Err(AppError::BadRequest(format!(
                    "No sales order item matches pipe {} ({} {} {}x{})",
                    oi.pipe_id, oi.pipe_type, grade, od, wt
                )));
            }

            let target = &matching_items[0];
            let new_qty = target.delivered_quantity + 1;
            SalesOrderRepo::update_delivered_quantity(&self.pool, &target.id, new_qty).await?;
        }

        // Auto-complete if all items fully delivered
        Self::auto_complete_so(&self.pool, so_id).await?;

        Ok(())
    }

    async fn auto_complete_so(pool: &SqlitePool, so_id: &str) -> AppResult<()> {
        let items = SalesOrderItemRepo::find_by_order(pool, so_id).await?;
        let all_delivered = items.iter().all(|i| i.delivered_quantity >= i.quantity);
        if all_delivered {
            SalesOrderRepo::update_status(pool, so_id, "completed").await?;
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

    async fn get_outbound_refs_by_order(&self, order_id: &str) -> AppResult<Vec<OutboundRef>> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT id, outbound_no FROM outbound_records WHERE order_id = ?"
        )
        .bind(order_id)
        .fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|(id, no)| OutboundRef {
            outbound_id: id,
            outbound_no: no,
        }).collect())
    }

    // ── ATP ─────────────────────────────────────────────────────────────────
    
    pub async fn get_atp(&self, query: &AtpQuery) -> AppResult<AtpResult> {
        let result = self.compute_atp(&query.pipe_type, &query.grade, query.od, query.wt).await?;
        Ok(result)
    }

    async fn compute_atp(&self, pipe_type: &str, grade: &str, od: f64, wt: f64) -> AppResult<AtpResult> {
        let in_stock = self.count_in_stock(pipe_type, grade, od, wt).await?;
        let allocated = self.sum_undelivered_sales(pipe_type, grade, od, wt).await?;
        let available = in_stock - allocated;

        Ok(AtpResult {
            pipe_type: pipe_type.to_string(),
            grade: grade.to_string(),
            od,
            wt,
            in_stock,
            allocated,
            available,
        })
    }

    async fn count_in_stock(&self, pipe_type: &str, grade: &str, od: f64, wt: f64) -> AppResult<i64> {
        match pipe_type {
            "seamless" => {
                let count: (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM seamless_pipes WHERE grade = ? AND ABS(od - ?) < 0.001 AND ABS(wt - ?) < 0.001 AND status = 'in_stock' AND deleted_at IS NULL"
                )
                .bind(grade).bind(od).bind(wt)
                .fetch_one(&self.pool).await?;
                Ok(count.0)
            }
            "screen" => {
                let count: (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM screen_pipes WHERE grade = ? AND ABS(od - ?) < 0.001 AND ABS(wt - ?) < 0.001 AND status = 'in_stock' AND deleted_at IS NULL"
                )
                .bind(grade).bind(od).bind(wt)
                .fetch_one(&self.pool).await?;
                Ok(count.0)
            }
            "all" => {
                let s: (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM seamless_pipes WHERE grade = ? AND ABS(od - ?) < 0.001 AND ABS(wt - ?) < 0.001 AND status = 'in_stock' AND deleted_at IS NULL"
                )
                .bind(grade).bind(od).bind(wt)
                .fetch_one(&self.pool).await?;
                let sc: (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM screen_pipes WHERE grade = ? AND ABS(od - ?) < 0.001 AND ABS(wt - ?) < 0.001 AND status = 'in_stock' AND deleted_at IS NULL"
                )
                .bind(grade).bind(od).bind(wt)
                .fetch_one(&self.pool).await?;
                Ok(s.0 + sc.0)
            }
            _ => Err(AppError::BadRequest(format!("Unknown pipe_type: {}", pipe_type))),
        }
    }

    /// Sums (quantity - delivered_quantity) for non-draft, non-cancelled, non-completed
    /// sales order items matching the given specs.
    async fn sum_undelivered_sales(&self, pipe_type: &str, grade: &str, od: f64, wt: f64) -> AppResult<i64> {
        let total: (i64,) = sqlx::query_as(
            "SELECT COALESCE(SUM(soi.quantity - soi.delivered_quantity), 0)
             FROM sales_order_items soi
             JOIN sales_orders so ON so.id = soi.order_id
             WHERE soi.pipe_type = ?
               AND soi.grade = ?
               AND ABS(soi.od - ?) < 0.001
               AND ABS(soi.wt - ?) < 0.001
               AND so.status NOT IN ('draft', 'cancelled', 'completed')"
        )
        .bind(pipe_type).bind(grade).bind(od).bind(wt)
        .fetch_one(&self.pool).await?;
        Ok(total.0)
    }
}

fn or_opt<T>(new: Option<T>, existing: Option<T>) -> Option<T> {
    match new {
        Some(v) => Some(v),
        None => existing,
    }
}
