//! Route definitions for the Steel Pipe DB API.
//!
//! # Organization Strategy
//!
//! Routes are grouped by authentication and authorization requirements:
//! - **Public** — no auth required (login, refresh)
//! - **Authenticated** — any logged-in user (logout, me, change-password)
//! - **Role-protected write** — admin/warehouse/qc/sales per domain
//! - **Authenticated read** — any logged-in user can read business data
//!
//! # Middleware Layering Order (innermost → outermost)
//!
//! ```text
//! route_layer(auth_middleware)
//!   → route_layer(rbac::require_role)
//!     → layer(CORS)
//!       → layer(Trace + RequestId)
//!         → layer(Extension<SqlitePool>)
//!           → layer(Extension<JwtSecret>)
//!             → layer(Extension<RateLimiter>)
//! ```
//!
//! # RBAC Quick Reference
//!
//! | Domain     | Read (any auth) | Write (roles)                    |
//! |------------|:---------------:|----------------------------------|
//! | Users      | admin           | admin                            |
//! | Pipes      | ✅              | admin, warehouse                 |
//! | Inbound    | ✅              | admin, warehouse                 |
//! | Outbound   | ✅              | admin, warehouse                 |
//! | Quality    | ✅              | admin, qc                        |
//! | Sales      | ✅              | admin, sales                     |
//! | Purchases  | ✅              | admin, warehouse, sales          |
//! | Suppliers  | ✅              | admin, warehouse, sales          |
//! | Customers  | ✅              | admin, warehouse, sales          |
//! | Contracts  | ✅              | admin, warehouse, sales          |
//! | Data IO    | ✅              | admin (import), any (export)     |
//! | Labels     | ✅              | admin, warehouse (write)         |
//! | Reports    | ✅              | — (read-only)                    |

use std::time::Duration as StdDuration;

use axum::{
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        HeaderName, HeaderValue, Method,
    },
    middleware, Router,
};
use sqlx::SqlitePool;
use tower::ServiceBuilder;
use tower_http::{request_id::MakeRequestUuid, ServiceBuilderExt};

use crate::middleware::rate_limit::{
    rate_limit_import, rate_limit_login, rate_limit_password_change, RateLimiter,
};
use crate::middleware::auth::JwtSecret;

use crate::handlers::atp_handler;
use crate::handlers::auth_handler;
use crate::handlers::contract_handler;
use crate::handlers::customer_handler;
use crate::handlers::data_io_handler;
use crate::handlers::inventory_handler;
use crate::handlers::label_handler;
use crate::handlers::pipe_handler;
use crate::handlers::purchase_handler;
use crate::handlers::quality_handler;
use crate::handlers::report_handler;
use crate::handlers::sales_handler;
use crate::handlers::supplier_handler;

// Helper functions for route groups with role-protected write operations
// Each returns a Router with auth_middleware + require_role on all endpoints.

fn admin_write_routes() -> Router {
    // Import routes — rate-limited separately (10/min per IP)
    let import_routes = Router::new()
        .route(
            "/api/v1/data-io/import/{entity_type}",
            axum::routing::post(data_io_handler::import_handler),
        )
        .route_layer(middleware::from_fn(rate_limit_import));

    Router::new()
        .merge(import_routes)
        .route(
            "/api/v1/users",
            axum::routing::post(auth_handler::create_user_handler),
        )
        .route(
            "/api/v1/users/{id}",
            axum::routing::put(auth_handler::update_user_handler)
                .delete(auth_handler::delete_user_handler),
        )
        .route(
            "/api/v1/users/{id}/role",
            axum::routing::put(auth_handler::change_role_handler),
        )
        .route_layer(middleware::from_fn(|req, next| {
            crate::middleware::rbac::require_role(req, next, &["admin"])
        }))
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ))
}

fn warehouse_write_routes() -> Router {
    Router::new()
        // Inbound
        .route(
            "/api/v1/inbound-records",
            axum::routing::post(inventory_handler::create_inbound_handler),
        )
        .route(
            "/api/v1/inbound-records/{id}",
            axum::routing::delete(inventory_handler::delete_inbound_handler),
        )
        .route(
            "/api/v1/inbound-records/{id}/approve",
            axum::routing::post(inventory_handler::approve_inbound_handler),
        )
        .route(
            "/api/v1/inbound-records/{id}/reject",
            axum::routing::post(inventory_handler::reject_inbound_handler),
        )
        // Outbound
        .route(
            "/api/v1/outbound-records",
            axum::routing::post(inventory_handler::create_outbound_handler),
        )
        .route(
            "/api/v1/outbound-records/{id}",
            axum::routing::delete(inventory_handler::delete_outbound_handler),
        )
        .route(
            "/api/v1/outbound-records/{id}/approve",
            axum::routing::post(inventory_handler::approve_outbound_handler),
        )
        .route(
            "/api/v1/outbound-records/{id}/reject",
            axum::routing::post(inventory_handler::reject_outbound_handler),
        )
        // Locations
        .route(
            "/api/v1/locations",
            axum::routing::post(inventory_handler::create_location_handler),
        )
        .route(
            "/api/v1/locations/{id}",
            axum::routing::put(inventory_handler::update_location_handler)
                .delete(inventory_handler::delete_location_handler),
        )
        // Inventory checks
        .route(
            "/api/v1/inventory/checks",
            axum::routing::post(inventory_handler::create_check_handler),
        )
        .route(
            "/api/v1/inventory/checks/{id}/complete",
            axum::routing::post(inventory_handler::complete_check_handler),
        )
        .route(
            "/api/v1/inventory/checks/{check_id}/items/{item_id}",
            axum::routing::put(inventory_handler::submit_check_item_handler),
        )
        // Location assign / transfer
        .route(
            "/api/v1/inventory/locations/{id}/assign",
            axum::routing::post(inventory_handler::assign_location_handler),
        )
        .route(
            "/api/v1/inventory/pipes/{pipe_type}/{pipe_id}/transfer-location",
            axum::routing::post(inventory_handler::transfer_location_handler),
        )
        // Batch inbound
        .route(
            "/api/v1/inbound-records/batch",
            axum::routing::post(inventory_handler::batch_create_inbound_handler),
        )
        // Labels (warehouse function)
        .route(
            "/api/v1/labels/batch",
            axum::routing::post(label_handler::create_batch_labels_handler),
        )
        .route(
            "/api/v1/labels/shipping",
            axum::routing::post(label_handler::create_shipping_label_handler),
        )
        .route_layer(middleware::from_fn(|req, next| {
            crate::middleware::rbac::require_role(req, next, &["admin", "warehouse"])
        }))
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ))
}

fn qc_write_routes() -> Router {
    Router::new()
        .route(
            "/api/v1/quality/certs",
            axum::routing::post(quality_handler::create_cert_handler),
        )
        .route(
            "/api/v1/quality/certs/{id}",
            axum::routing::put(quality_handler::update_cert_handler)
                .delete(quality_handler::delete_cert_handler),
        )
        .route(
            "/api/v1/quality/attachments",
            axum::routing::post(quality_handler::create_attachment_handler),
        )
        .route(
            "/api/v1/quality/attachments/{id}",
            axum::routing::delete(quality_handler::delete_attachment_handler),
        )
        .route_layer(middleware::from_fn(|req, next| {
            crate::middleware::rbac::require_role(req, next, &["admin", "qc"])
        }))
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ))
}

fn sales_write_routes() -> Router {
    Router::new()
        .route(
            "/api/v1/sales-orders",
            axum::routing::post(sales_handler::create_sales_order_handler),
        )
        .route(
            "/api/v1/sales-orders/{id}",
            axum::routing::put(sales_handler::update_sales_order_handler)
                .delete(sales_handler::delete_sales_order_handler),
        )
        .route(
            "/api/v1/sales-orders/{id}/transition",
            axum::routing::post(sales_handler::transition_sales_order_status_handler),
        )
        .route(
            "/api/v1/sales-orders/{order_id}/items/{item_id}",
            axum::routing::put(sales_handler::update_sales_item_handler)
                .delete(sales_handler::delete_sales_item_handler),
        )
        .route(
            "/api/v1/sales-orders/{id}/approve",
            axum::routing::post(sales_handler::approve_sales_order_handler),
        )
        .route(
            "/api/v1/sales-orders/{id}/reject",
            axum::routing::post(sales_handler::reject_sales_order_handler),
        )
        .route(
            "/api/v1/sales-orders/{id}/link-outbound",
            axum::routing::post(sales_handler::link_outbound_to_order_handler),
        )
        .route_layer(middleware::from_fn(|req, next| {
            crate::middleware::rbac::require_role(req, next, &["admin", "sales"])
        }))
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ))
}

fn purchases_write_routes() -> Router {
    Router::new()
        .route(
            "/api/v1/purchase-orders",
            axum::routing::post(purchase_handler::create_purchase_order_handler),
        )
        .route(
            "/api/v1/purchase-orders/{id}",
            axum::routing::put(purchase_handler::update_purchase_order_handler)
                .delete(purchase_handler::delete_purchase_order_handler),
        )
        .route(
            "/api/v1/purchase-orders/{id}/transition",
            axum::routing::post(purchase_handler::transition_purchase_order_status_handler),
        )
        .route(
            "/api/v1/purchase-orders/{order_id}/items/{item_id}",
            axum::routing::put(purchase_handler::update_purchase_item_handler)
                .delete(purchase_handler::delete_purchase_item_handler),
        )
        .route(
            "/api/v1/purchase-orders/{id}/approve",
            axum::routing::post(purchase_handler::approve_purchase_order_handler),
        )
        .route(
            "/api/v1/purchase-orders/{id}/reject",
            axum::routing::post(purchase_handler::reject_purchase_order_handler),
        )
        .route(
            "/api/v1/purchase-orders/{id}/link-inbound",
            axum::routing::post(purchase_handler::link_inbound_to_order_handler),
        )
        .route_layer(middleware::from_fn(|req, next| {
            crate::middleware::rbac::require_role(req, next, &["admin", "warehouse", "sales"])
        }))
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ))
}

fn supplier_customer_write_routes() -> Router {
    Router::new()
        .route(
            "/api/v1/suppliers",
            axum::routing::post(supplier_handler::create_supplier_handler),
        )
        .route(
            "/api/v1/suppliers/{id}",
            axum::routing::put(supplier_handler::update_supplier_handler)
                .delete(supplier_handler::delete_supplier_handler),
        )
        .route(
            "/api/v1/customers",
            axum::routing::post(customer_handler::create_customer_handler),
        )
        .route(
            "/api/v1/customers/{id}",
            axum::routing::put(customer_handler::update_customer_handler)
                .delete(customer_handler::delete_customer_handler),
        )
        .route_layer(middleware::from_fn(|req, next| {
            crate::middleware::rbac::require_role(req, next, &["admin", "warehouse", "sales"])
        }))
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ))
}

fn contract_write_routes() -> Router {
    Router::new()
        .route(
            "/api/v1/contracts",
            axum::routing::post(contract_handler::create_contract_handler),
        )
        .route(
            "/api/v1/contracts/{id}",
            axum::routing::put(contract_handler::update_contract_handler)
                .delete(contract_handler::delete_contract_handler),
        )
        .route(
            "/api/v1/contracts/{id}/status",
            axum::routing::post(contract_handler::update_contract_status_handler),
        )
        .route(
            "/api/v1/contracts/{contract_id}/items",
            axum::routing::post(contract_handler::add_contract_item_handler),
        )
        .route(
            "/api/v1/contracts/{contract_id}/items/{item_id}",
            axum::routing::put(contract_handler::update_contract_item_handler)
                .delete(contract_handler::delete_contract_item_handler),
        )
        .route(
            "/api/v1/contracts/{contract_id}/payments",
            axum::routing::post(contract_handler::add_contract_payment_handler),
        )
        .route(
            "/api/v1/contracts/{contract_id}/payments/{payment_id}",
            axum::routing::put(contract_handler::update_contract_payment_handler)
                .delete(contract_handler::delete_contract_payment_handler),
        )
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ))
        .route_layer(middleware::from_fn(|req, next| {
            crate::middleware::rbac::require_role(req, next, &["admin", "warehouse", "sales"])
        }))
}

// Main app builder — assembles all route groups, middleware, and shared layers

pub fn create_app(pool: SqlitePool, jwt_secret: String) -> Router {
    // Public: no auth required
    let public_auth = Router::new()
        .route("/api/v1/auth/login", axum::routing::post(auth_handler::login_handler))
        .route(
            "/api/v1/auth/refresh",
            axum::routing::post(auth_handler::refresh_handler),
        )
        .route_layer(middleware::from_fn(rate_limit_login));

    // Authenticated (any logged-in user)
    let authenticated = Router::new()
        .route(
            "/api/v1/auth/logout",
            axum::routing::post(auth_handler::logout_handler),
        )
        .route(
            "/api/v1/auth/me",
            axum::routing::get(auth_handler::me_handler),
        )
        .route(
            "/api/v1/users/{id}/change-password",
            axum::routing::post(auth_handler::change_password_handler),
        )
        .route_layer(middleware::from_fn(rate_limit_password_change))
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    // Admin read-only (GET user list)
    let admin_read = Router::new()
        .route(
            "/api/v1/users",
            axum::routing::get(auth_handler::list_users_handler),
        )
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ))
        .route_layer(middleware::from_fn(|req, next| {
            crate::middleware::rbac::require_role(req, next, &["admin"])
        }));

    // Pipe read-only (GET, search)
    let pipe_read = Router::new()
        .route(
            "/api/v1/seamless-pipes",
            axum::routing::get(pipe_handler::list_seamless_pipes_handler),
        )
        .route(
            "/api/v1/seamless-pipes/{id}",
            axum::routing::get(pipe_handler::get_seamless_pipe_handler),
        )
        .route(
            "/api/v1/screen-pipes",
            axum::routing::get(pipe_handler::list_screen_pipes_handler),
        )
        .route(
            "/api/v1/screen-pipes/{id}",
            axum::routing::get(pipe_handler::get_screen_pipe_handler),
        )
        .route(
            "/api/v1/pipes/search",
            axum::routing::get(pipe_handler::search_pipes_handler),
        )
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    // Pipe write (POST, PUT, DELETE)
    let pipe_write = Router::new()
        .route(
            "/api/v1/seamless-pipes",
            axum::routing::post(pipe_handler::create_seamless_pipe_handler),
        )
        .route(
            "/api/v1/seamless-pipes/{id}",
            axum::routing::put(pipe_handler::update_seamless_pipe_handler)
                .delete(pipe_handler::delete_seamless_pipe_handler),
        )
        .route(
            "/api/v1/screen-pipes",
            axum::routing::post(pipe_handler::create_screen_pipe_handler),
        )
        .route(
            "/api/v1/screen-pipes/{id}",
            axum::routing::put(pipe_handler::update_screen_pipe_handler)
                .delete(pipe_handler::delete_screen_pipe_handler),
        )
        .route_layer(middleware::from_fn(|req, next| {
            crate::middleware::rbac::require_role(req, next, &["admin", "warehouse"])
        }))
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    // Inventory read (GET)
    let inventory_read = Router::new()
        .route(
            "/api/v1/inbound-records",
            axum::routing::get(inventory_handler::list_inbound_handler),
        )
        .route(
            "/api/v1/inbound-records/{id}",
            axum::routing::get(inventory_handler::get_inbound_handler),
        )
        .route(
            "/api/v1/outbound-records",
            axum::routing::get(inventory_handler::list_outbound_handler),
        )
        .route(
            "/api/v1/outbound-records/{id}",
            axum::routing::get(inventory_handler::get_outbound_handler),
        )
        .route(
            "/api/v1/inventory",
            axum::routing::get(inventory_handler::list_inventory_handler),
        )
        .route(
            "/api/v1/inventory/logs",
            axum::routing::get(inventory_handler::list_inventory_logs_handler),
        )
        .route(
            "/api/v1/locations",
            axum::routing::get(inventory_handler::list_locations_handler),
        )
        .route(
            "/api/v1/locations/{id}",
            axum::routing::get(inventory_handler::get_location_handler),
        )
        .route(
            "/api/v1/inventory/checks",
            axum::routing::get(inventory_handler::list_checks_handler),
        )
        .route(
            "/api/v1/inventory/checks/{id}",
            axum::routing::get(inventory_handler::get_check_handler),
        )
        .route(
            "/api/v1/trace/pipe/{pipe_type}/{pipe_id}",
            axum::routing::get(inventory_handler::trace_pipe_handler),
        )
        .route(
            "/api/v1/trace/heat-number/{heat_number}",
            axum::routing::get(inventory_handler::trace_heat_handler),
        )
        .route(
            "/api/v1/trace/order/{order_type}/{order_id}",
            axum::routing::get(inventory_handler::trace_order_handler),
        )
        .route(
            "/api/v1/inventory/statistics",
            axum::routing::get(inventory_handler::inventory_statistics_handler),
        )
        .route(
            "/api/v1/inbound-records/{id}/items",
            axum::routing::get(inventory_handler::list_inbound_items_handler),
        )
        .route(
            "/api/v1/outbound-records/{id}/items",
            axum::routing::get(inventory_handler::list_outbound_items_handler),
        )
        .route(
            "/api/v1/atp",
            axum::routing::get(atp_handler::check_atp_handler),
        )
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    // Data IO read (GET export, templates, logs)
    let data_io_read = Router::new()
        .route(
            "/api/v1/data-io/export/{entity_type}",
            axum::routing::get(data_io_handler::export_handler),
        )
        .route(
            "/api/v1/data-io/templates/{entity_type}",
            axum::routing::get(data_io_handler::template_handler),
        )
        .route(
            "/api/v1/data-io/operation-logs",
            axum::routing::get(data_io_handler::list_operation_logs_handler),
        )
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    // Supplier/Customer read (GET, search)
    let supplier_read = Router::new()
        .route(
            "/api/v1/suppliers/search",
            axum::routing::get(supplier_handler::search_suppliers_handler),
        )
        .route(
            "/api/v1/suppliers/active",
            axum::routing::get(supplier_handler::list_active_suppliers_handler),
        )
        .route(
            "/api/v1/suppliers",
            axum::routing::get(supplier_handler::list_suppliers_handler),
        )
        .route(
            "/api/v1/suppliers/{id}",
            axum::routing::get(supplier_handler::get_supplier_handler),
        )
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    let customer_read = Router::new()
        .route(
            "/api/v1/customers/search",
            axum::routing::get(customer_handler::search_customers_handler),
        )
        .route(
            "/api/v1/customers/active",
            axum::routing::get(customer_handler::list_active_customers_handler),
        )
        .route(
            "/api/v1/customers",
            axum::routing::get(customer_handler::list_customers_handler),
        )
        .route(
            "/api/v1/customers/{id}",
            axum::routing::get(customer_handler::get_customer_handler),
        )
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    // Purchase order read (GET)
    let purchase_read = Router::new()
        .route(
            "/api/v1/purchase-orders",
            axum::routing::get(purchase_handler::list_purchase_orders_handler),
        )
        .route(
            "/api/v1/purchase-orders/{id}",
            axum::routing::get(purchase_handler::get_purchase_order_handler),
        )
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    // Sales order read (GET)
    let sales_read = Router::new()
        .route(
            "/api/v1/sales-orders",
            axum::routing::get(sales_handler::list_sales_orders_handler),
        )
        .route(
            "/api/v1/sales-orders/{id}",
            axum::routing::get(sales_handler::get_sales_order_handler),
        )
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    // Quality read (GET)
    let quality_read = Router::new()
        .route(
            "/api/v1/quality/certs",
            axum::routing::get(quality_handler::list_certs_handler),
        )
        .route(
            "/api/v1/quality/certs/{id}",
            axum::routing::get(quality_handler::get_cert_handler),
        )
        .route(
            "/api/v1/quality/grades",
            axum::routing::get(quality_handler::list_grades_handler),
        )
        .route(
            "/api/v1/quality/grades/query",
            axum::routing::get(quality_handler::get_grade_handler),
        )
        .route(
            "/api/v1/quality/attachments",
            axum::routing::get(quality_handler::list_attachments_handler),
        )
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    // Contract read (GET)
    let contract_read = Router::new()
        .route(
            "/api/v1/contracts",
            axum::routing::get(contract_handler::list_contracts_handler),
        )
        .route(
            "/api/v1/contracts/{id}",
            axum::routing::get(contract_handler::get_contract_handler),
        )
        .route(
            "/api/v1/contracts/{contract_id}/payments",
            axum::routing::get(contract_handler::list_contract_payments_handler),
        )
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    // Reports (GET-only)
    let report_routes = Router::new()
        .route(
            "/api/v1/reports/inventory-summary",
            axum::routing::get(report_handler::inventory_summary_handler),
        )
        .route(
            "/api/v1/reports/order-report",
            axum::routing::get(report_handler::order_report_handler),
        )
        .route(
            "/api/v1/reports/quality-report",
            axum::routing::get(report_handler::quality_report_handler),
        )
        .route(
            "/api/v1/reports/dashboard",
            axum::routing::get(report_handler::dashboard_handler),
        )
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    // Labels read (GET)
    let label_read = Router::new()
        .route(
            "/api/v1/labels/pipe/{pipe_type}/{pipe_id}",
            axum::routing::get(label_handler::get_pipe_label_handler),
        )
        .route(
            "/api/v1/labels/quality/{cert_id}",
            axum::routing::get(label_handler::get_quality_label_handler),
        )
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    Router::new()
        // Public
        .merge(public_auth)
        // Authenticated (any role)
        .merge(authenticated)
        // Admin read
        .merge(admin_read)
        // Business read-only (all authenticated users)
        .merge(pipe_read)
        .merge(inventory_read)
        .merge(data_io_read)
        .merge(supplier_read)
        .merge(customer_read)
        .merge(purchase_read)
        .merge(sales_read)
        .merge(quality_read)
        .merge(contract_read)
        .merge(report_routes)
        .merge(label_read)
        // Write-protected (role-checked)
        .merge(admin_write_routes())
        .merge(pipe_write)
        .merge(warehouse_write_routes())
        .merge(qc_write_routes())
        .merge(sales_write_routes())
        .merge(purchases_write_routes())
        .merge(supplier_customer_write_routes())
        .merge(contract_write_routes())
        // Shared layers — outermost (applied first)
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin([
                    "http://localhost:5173".parse::<HeaderValue>().unwrap(),
                ])
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_headers([
                    AUTHORIZATION,
                    CONTENT_TYPE,
                    HeaderName::from_static("x-request-id"),
                ])
                .expose_headers([
                    AUTHORIZATION,
                    CONTENT_TYPE,
                    HeaderName::from_static("x-request-id"),
                ])
                .max_age(StdDuration::from_secs(86400)),
        )
        .layer(
            ServiceBuilder::new()
                .set_x_request_id(MakeRequestUuid::default())
                .layer(tower_http::trace::TraceLayer::new_for_http())
                .propagate_x_request_id(),
        )
        .layer(axum::Extension(pool))
        .layer(axum::Extension(JwtSecret(jwt_secret)))
        .layer(axum::Extension(RateLimiter::new()))
}
