//! HTTP 路由组装
//!
//! 路由总入口——各业务模块实现后在此装配。
//! 分层：public 路由（/auth/login）/auth/refresh）+ protected 路由（带 JWT 中间件）。
//! 依赖注入：Extension<SqlitePool> + Extension<JwtSecret>（非 State<Arc<AppState>>）。

use std::time::Duration;

use axum::middleware as axum_mw;
use axum::routing::{get, post};
use axum::Extension;
use axum::Router;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::config::Config;
use crate::http::auth as auth_handlers;
use crate::http::catalog as catalog_handlers;
use crate::http::finance as finance_handlers;
use crate::http::inventory as inventory_handlers;
use crate::http::locations as location_handlers;
use crate::http::parties as parties_handlers;
use crate::http::purchase as purchase_handlers;
use crate::http::receipt as receipt_handlers;
use crate::http::sales as sales_handlers;
use crate::http::reports as reports_handlers;
use crate::http::shipment as shipment_handlers;
use crate::http::workflow as workflow_handlers;
use crate::middleware::auth::auth_middleware;
use crate::middleware::auth::JwtSecret;
use crate::middleware::rbac::{require_admin, require_permission};

pub mod auth;
pub mod catalog;
pub mod finance;
pub mod inventory;
pub mod locations;
pub mod parties;
pub mod purchase;
pub mod receipt;
pub mod reports;
pub mod sales;
pub mod shipment;
pub mod workflow;

pub fn router(pool: sqlx::SqlitePool, jwt_secret: String) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .expose_headers([axum::http::header::HeaderName::from_static("x-request-id")]);

    let auth_protected: Router = Router::new()
        .route("/auth/me", get(auth_handlers::me))
        .route("/auth/logout", post(auth_handlers::logout))
        .route(
            "/users",
            get(auth_handlers::list_users).post(auth_handlers::create_user),
        )
        .route(
            "/users/{id}",
            axum::routing::put(auth_handlers::update_user).delete(auth_handlers::delete_user),
        )
        .route("/roles", get(auth_handlers::list_roles))
        .route("/permissions", get(auth_handlers::list_permissions))
        .route("/operation-logs", get(auth_handlers::list_operation_logs))
        .route_layer(axum_mw::from_fn_with_state(
            "user.manage",
            require_permission,
        ))
        .route_layer(axum_mw::from_fn_with_state((), require_admin))
        .route_layer(axum_mw::from_fn(auth_middleware))
        .route_layer(Extension(pool.clone()))
        .route_layer(Extension(JwtSecret(jwt_secret.clone())));

    let catalog_read: Router = Router::new()
        .route("/items", get(catalog_handlers::list_items))
        .route("/items/categories", get(catalog_handlers::list_categories))
        .route("/items/{id}", get(catalog_handlers::get_item))
        .route_layer(axum_mw::from_fn_with_state("item.read", require_permission))
        .route_layer(axum_mw::from_fn(auth_middleware))
        .route_layer(Extension(pool.clone()))
        .route_layer(Extension(JwtSecret(jwt_secret.clone())));

let catalog_write: Router = Router::new()
.route("/items", post(catalog_handlers::create_item))
.route("/items/import", post(catalog_handlers::import_items_csv))
.route(
"/items/{id}",
axum::routing::put(catalog_handlers::update_item).delete(catalog_handlers::delete_item),
        )
        .route_layer(axum_mw::from_fn_with_state(
            "item.write",
            require_permission,
        ))
        .route_layer(axum_mw::from_fn(auth_middleware))
        .route_layer(Extension(pool.clone()))
        .route_layer(Extension(JwtSecret(jwt_secret.clone())));

    let parties_read: Router = Router::new()
        .route("/suppliers", get(parties_handlers::list_suppliers))
        .route("/suppliers/{id}", get(parties_handlers::get_supplier))
        .route("/customers", get(parties_handlers::list_customers))
        .route("/customers/{id}", get(parties_handlers::get_customer))
        .route_layer(axum_mw::from_fn_with_state(
            "order.read",
            require_permission,
        ))
        .route_layer(axum_mw::from_fn(auth_middleware))
        .route_layer(Extension(pool.clone()))
        .route_layer(Extension(JwtSecret(jwt_secret.clone())));

    let parties_write: Router = Router::new()
        .route("/suppliers", post(parties_handlers::create_supplier))
        .route(
            "/suppliers/{id}",
            axum::routing::put(parties_handlers::update_supplier)
                .delete(parties_handlers::delete_supplier),
        )
        .route("/customers", post(parties_handlers::create_customer))
        .route(
            "/customers/{id}",
            axum::routing::put(parties_handlers::update_customer)
                .delete(parties_handlers::delete_customer),
        )
        .route_layer(axum_mw::from_fn_with_state(
            "order.write",
            require_permission,
        ))
        .route_layer(axum_mw::from_fn(auth_middleware))
        .route_layer(Extension(pool.clone()))
        .route_layer(Extension(JwtSecret(jwt_secret.clone())));

    let warehouse_read: Router = Router::new()
        .route("/warehouses", get(location_handlers::list_warehouses))
        .route("/warehouses/{id}", get(location_handlers::get_warehouse))
        .route("/locations", get(location_handlers::list_locations))
        .route("/locations/{id}", get(location_handlers::get_location))
        .route_layer(axum_mw::from_fn_with_state(
            "stock.read",
            require_permission,
        ))
        .route_layer(axum_mw::from_fn(auth_middleware))
        .route_layer(Extension(pool.clone()))
        .route_layer(Extension(JwtSecret(jwt_secret.clone())));

    let warehouse_write: Router = Router::new()
        .route("/warehouses", post(location_handlers::create_warehouse))
        .route(
            "/warehouses/{id}",
            axum::routing::put(location_handlers::update_warehouse)
                .delete(location_handlers::delete_warehouse),
        )
        .route("/locations", post(location_handlers::create_location))
        .route(
            "/locations/{id}",
            axum::routing::put(location_handlers::update_location)
                .delete(location_handlers::delete_location),
        )
        .route_layer(axum_mw::from_fn_with_state(
            "stock.write",
            require_permission,
        ))
        .route_layer(axum_mw::from_fn(auth_middleware))
        .route_layer(Extension(pool.clone()))
        .route_layer(Extension(JwtSecret(jwt_secret.clone())));

    let inventory_read: Router = Router::new()
        .route("/inbounds", get(inventory_handlers::list_inbounds))
        .route("/inbounds/{id}", get(inventory_handlers::get_inbound))
        .route("/outbounds", get(inventory_handlers::list_outbounds))
        .route("/outbounds/{id}", get(inventory_handlers::get_outbound))
        .route("/stock", get(inventory_handlers::list_stock))
        .route("/inventory-logs", get(inventory_handlers::list_logs))
        .route("/inventory/available", get(inventory_handlers::get_available_qty))
        .route(
            "/check-records",
            get(inventory_handlers::list_check_sessions),
        )
        .route(
            "/check-records/{id}",
            get(inventory_handlers::get_check_session),
        )
        .route(
            "/inventory/checks",
            get(inventory_handlers::list_check_sessions),
        )
        .route(
            "/inventory/checks/{id}",
            get(inventory_handlers::get_check_session),
        )
        .route_layer(axum_mw::from_fn_with_state(
            "stock.read",
            require_permission,
        ))
        .route_layer(axum_mw::from_fn(auth_middleware))
        .route_layer(Extension(pool.clone()))
        .route_layer(Extension(JwtSecret(jwt_secret.clone())));

    let inventory_write: Router = Router::new()
        .route("/inbounds", post(inventory_handlers::create_inbound))
        .route(
            "/inbounds/{id}/post",
            post(inventory_handlers::post_inbound),
        )
        .route("/outbounds", post(inventory_handlers::create_outbound))
        .route(
            "/outbounds/{id}/post",
            post(inventory_handlers::post_outbound),
        )
        .route(
            "/check-records",
            post(inventory_handlers::create_check_session),
        )
        .route(
            "/check-records/{id}",
            axum::routing::put(inventory_handlers::record_actual_qty),
        )
        .route(
            "/check-records/{id}/post",
            post(inventory_handlers::post_check_session),
        )
        .route(
            "/inventory/checks",
            post(inventory_handlers::create_check_session),
        )
        .route(
            "/inventory/checks/{id}",
            axum::routing::put(inventory_handlers::record_actual_qty),
        )
        .route(
            "/inventory/checks/{id}/count",
            post(inventory_handlers::record_actual_qty),
        )
        .route(
            "/inventory/checks/{id}/post",
            post(inventory_handlers::post_check_session),
        )
        .route_layer(axum_mw::from_fn_with_state(
            "stock.write",
            require_permission,
        ))
        .route_layer(axum_mw::from_fn(auth_middleware))
        .route_layer(Extension(pool.clone()))
        .route_layer(Extension(JwtSecret(jwt_secret.clone())));

    let sales_read: Router = Router::new()
        .route("/sales-orders", get(sales_handlers::list_orders))
        .route("/sales-orders/{id}", get(sales_handlers::get_order))
        .route("/reservations", get(sales_handlers::list_reservations))
        .route_layer(axum_mw::from_fn_with_state(
            "order.read",
            require_permission,
        ))
        .route_layer(axum_mw::from_fn(auth_middleware))
        .route_layer(Extension(pool.clone()))
        .route_layer(Extension(JwtSecret(jwt_secret.clone())));

    let sales_write: Router = Router::new()
        .route("/sales-orders", post(sales_handlers::create_order))
        .route(
            "/sales-orders/{id}",
            axum::routing::put(sales_handlers::update_order).delete(sales_handlers::delete_order),
        )
        .route(
            "/sales-orders/{id}/submit",
            post(sales_handlers::submit_order),
        )
        .route(
            "/sales-orders/{id}/cancel",
            post(sales_handlers::cancel_order),
        )
        .route_layer(axum_mw::from_fn_with_state(
            "order.write",
            require_permission,
        ))
        .route_layer(axum_mw::from_fn(auth_middleware))
        .route_layer(Extension(pool.clone()))
        .route_layer(Extension(JwtSecret(jwt_secret.clone())));

    let sales_approve: Router = Router::new()
        .route(
            "/sales-orders/{id}/approve",
            post(sales_handlers::approve_order),
        )
        .route(
            "/sales-orders/{id}/reject",
            post(sales_handlers::reject_order),
        )
        .route_layer(axum_mw::from_fn_with_state(
            "order.approve",
            require_permission,
        ))
        .route_layer(axum_mw::from_fn(auth_middleware))
        .route_layer(Extension(pool.clone()))
        .route_layer(Extension(JwtSecret(jwt_secret.clone())));

    let purchase_read: Router = Router::new()
        .route(
            "/purchase-orders",
            get(purchase_handlers::list_purchase_orders),
        )
        .route(
            "/purchase-orders/{id}",
            get(purchase_handlers::get_purchase_order),
        )
        .route_layer(axum_mw::from_fn_with_state(
            "order.read",
            require_permission,
        ))
        .route_layer(axum_mw::from_fn(auth_middleware))
        .route_layer(Extension(pool.clone()))
        .route_layer(Extension(JwtSecret(jwt_secret.clone())));

    let purchase_write: Router = Router::new()
        .route(
            "/purchase-orders",
            post(purchase_handlers::create_purchase_order),
        )
        .route(
            "/purchase-orders/{id}",
            axum::routing::put(purchase_handlers::update_purchase_order)
                .delete(purchase_handlers::delete_purchase_order),
        )
        .route(
            "/purchase-orders/{id}/cancel",
            post(purchase_handlers::cancel_purchase_order),
        )
        .route_layer(axum_mw::from_fn_with_state(
            "order.write",
            require_permission,
        ))
        .route_layer(axum_mw::from_fn(auth_middleware))
        .route_layer(Extension(pool.clone()))
        .route_layer(Extension(JwtSecret(jwt_secret.clone())));

    let purchase_approve: Router = Router::new()
        .route(
            "/purchase-orders/{id}/submit",
            post(purchase_handlers::submit_purchase_order),
        )
        .route(
            "/purchase-orders/{id}/approve",
            post(purchase_handlers::approve_purchase_order),
        )
        .route(
            "/purchase-orders/{id}/reject",
            post(purchase_handlers::reject_purchase_order),
        )
        .route_layer(axum_mw::from_fn_with_state(
            "order.approve",
            require_permission,
        ))
        .route_layer(axum_mw::from_fn(auth_middleware))
        .route_layer(Extension(pool.clone()))
        .route_layer(Extension(JwtSecret(jwt_secret.clone())));

    let receipt_write: Router = Router::new()
        .route(
            "/purchase-orders/{id}/receive",
            post(receipt_handlers::receive_purchase_order),
        )
        .route_layer(axum_mw::from_fn_with_state(
            "stock.write",
            require_permission,
        ))
        .route_layer(axum_mw::from_fn(auth_middleware))
        .route_layer(Extension(pool.clone()))
        .route_layer(Extension(JwtSecret(jwt_secret.clone())));

    let shipment_write: Router = Router::new()
        .route(
            "/sales-orders/{id}/ship",
            post(shipment_handlers::ship_sales_order),
        )
        .route_layer(axum_mw::from_fn_with_state(
            "stock.write",
            require_permission,
        ))
        .route_layer(axum_mw::from_fn(auth_middleware))
        .route_layer(Extension(pool.clone()))
        .route_layer(Extension(JwtSecret(jwt_secret.clone())));

    let workflow_admin: Router = Router::new()
        .route(
            "/workflows",
            post(workflow_handlers::create_workflow).get(workflow_handlers::list_workflows),
        )
        .route(
            "/workflows/{id}",
            axum::routing::put(workflow_handlers::update_workflow)
                .get(workflow_handlers::get_workflow)
                .delete(workflow_handlers::delete_workflow),
        )
        .route_layer(axum_mw::from_fn_with_state(
            "user.manage",
            require_permission,
        ))
        .route_layer(axum_mw::from_fn(auth_middleware))
        .route_layer(Extension(pool.clone()))
        .route_layer(Extension(JwtSecret(jwt_secret.clone())));

    let workflow_view: Router = Router::new()
        .route(
            "/workflow-instances",
            get(workflow_handlers::list_instances),
        )
        .route(
            "/workflow-instances/{id}",
            get(workflow_handlers::get_instance),
        )
        .route("/workflow-tasks", get(workflow_handlers::list_tasks))
        .route(
            "/workflow-tasks/{task_id}/complete",
            post(workflow_handlers::complete_task),
        )
        .route_layer(axum_mw::from_fn_with_state(
            "order.read",
            require_permission,
        ))
        .route_layer(axum_mw::from_fn(auth_middleware))
        .route_layer(Extension(pool.clone()))
        .route_layer(Extension(JwtSecret(jwt_secret.clone())));

    let finance_read: Router = Router::new()
        .route("/accounts", get(finance_handlers::list_accounts))
        .route(
            "/journal-entries",
            get(finance_handlers::list_journal_entries),
        )
        .route("/invoices", get(finance_handlers::list_invoices))
        .route("/payments", get(finance_handlers::list_payments))
        .route("/trial-balance", get(finance_handlers::trial_balance))
        .route_layer(axum_mw::from_fn_with_state(
            "finance.read",
            require_permission,
        ))
        .route_layer(axum_mw::from_fn(auth_middleware))
        .route_layer(Extension(pool.clone()))
        .route_layer(Extension(JwtSecret(jwt_secret.clone())));

    let finance_write: Router = Router::new()
        .route("/accounts", post(finance_handlers::create_account))
        .route(
            "/journal-entries",
            post(finance_handlers::create_journal_entry),
        )
        .route(
            "/journal-entries/{id}/post",
            post(finance_handlers::post_journal_entry),
        )
        .route("/invoices", post(finance_handlers::create_invoice))
        .route("/payments", post(finance_handlers::create_payment))
        .route_layer(axum_mw::from_fn_with_state(
            "finance.write",
            require_permission,
        ))
        .route_layer(axum_mw::from_fn(auth_middleware))
        .route_layer(Extension(pool.clone()))
        .route_layer(Extension(JwtSecret(jwt_secret.clone())));

    let reports_read: Router = Router::new()
        .route("/reports/inventory-summary", get(reports_handlers::inventory_summary_report))
        .route("/reports/inbound-outbound", get(reports_handlers::inbound_outbound_report))
        .route("/reports/sales-trend", get(reports_handlers::sales_trend_report))
        .route("/reports/finance-summary", get(reports_handlers::finance_summary_report))
        .route_layer(axum_mw::from_fn_with_state(
            "report.read",
            require_permission,
        ))
        .route_layer(axum_mw::from_fn(auth_middleware))
        .route_layer(Extension(pool.clone()))
        .route_layer(Extension(JwtSecret(jwt_secret.clone())));

    let protected = auth_protected
        .merge(catalog_read)
        .merge(catalog_write)
        .merge(parties_read)
        .merge(parties_write)
        .merge(warehouse_read)
        .merge(warehouse_write)
        .merge(inventory_read)
        .merge(inventory_write)
        .merge(sales_read)
        .merge(sales_write)
        .merge(sales_approve)
        .merge(purchase_read)
        .merge(purchase_write)
        .merge(purchase_approve)
        .merge(receipt_write)
        .merge(shipment_write)
        .merge(workflow_admin)
        .merge(workflow_view)
        .merge(finance_read)
        .merge(finance_write)
        .merge(reports_read);

    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/auth/login", post(auth_handlers::login))
        .route("/auth/refresh", post(auth_handlers::refresh))
        .merge(protected)
        .layer(Extension(pool))
        .layer(Extension(JwtSecret(jwt_secret)))
        .layer(Extension(Config::from_env().expect("config load")))
        .layer(TraceLayer::new_for_http().make_span_with(|req: &axum::extract::Request| {
            tracing::info_span!("req", method = %req.method(), uri = %req.uri())
        }))
        .layer(cors)
}

#[allow(dead_code)]
fn _duration_ms(d: u64) -> Duration {
    Duration::from_millis(d)
}
