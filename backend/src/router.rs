//! Route definitions for the ERP API.
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
//! | Inbound    | ✅              | admin, warehouse                 |
//! | Outbound   | ✅              | admin, warehouse                 |
//! | Sales      | ✅              | admin, sales                     |
//! | Purchases  | ✅              | admin, warehouse, sales          |
//! | Suppliers  | ✅              | admin, warehouse, sales          |
//! | Customers  | ✅              | admin, warehouse, sales          |
//! | Contracts  | ✅              | admin, warehouse, sales          |
//! | Data IO    | templates       | admin (import/logs), admin/warehouse/sales (export) |
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

use crate::cache::CacheManager;
use crate::middleware::auth::JwtSecret;
use crate::middleware::rate_limit::{
    rate_limit_import, rate_limit_login, rate_limit_password_change, RateLimiter,
};
use crate::middleware::security_headers::security_headers;

use crate::inventory::atp_handler;
use crate::auth::handlers_legacy as auth_handler;
use crate::inventory::check_handler;
use crate::contracts::contract_handler;
use crate::parties::customer_handler;
use crate::data_io::data_io_handler;
use crate::health as health_handler;
use crate::inventory::inbound_handler;
use crate::inventory::inventory_handler;
use crate::items::item_handler;
use crate::inventory::location_handler;
use crate::inventory::outbound_handler;
use crate::orders::purchase_handler;
use crate::reports::report_handler;
use crate::orders::sales_handler;
use crate::parties::supplier_handler;

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
            axum::routing::post(inbound_handler::create_inbound_handler),
        )
        .route(
            "/api/v1/inbound-records/{id}",
            axum::routing::put(inbound_handler::update_inbound_handler)
                .delete(inbound_handler::delete_inbound_handler),
        )
        .route(
            "/api/v1/inbound-records/{id}/approve",
            axum::routing::post(inbound_handler::approve_inbound_handler),
        )
        .route(
            "/api/v1/inbound-records/{id}/reject",
            axum::routing::post(inbound_handler::reject_inbound_handler),
        )
        // Outbound
        .route(
            "/api/v1/outbound-records",
            axum::routing::post(outbound_handler::create_outbound_handler),
        )
        .route(
            "/api/v1/outbound-records/{id}",
            axum::routing::put(outbound_handler::update_outbound_handler)
                .delete(outbound_handler::delete_outbound_handler),
        )
        .route(
            "/api/v1/outbound-records/{id}/approve",
            axum::routing::post(outbound_handler::approve_outbound_handler),
        )
        .route(
            "/api/v1/outbound-records/{id}/reject",
            axum::routing::post(outbound_handler::reject_outbound_handler),
        )
        // Locations
        .route(
            "/api/v1/locations",
            axum::routing::post(location_handler::create_location_handler),
        )
        .route(
            "/api/v1/locations/{id}",
            axum::routing::put(location_handler::update_location_handler)
                .delete(location_handler::delete_location_handler),
        )
        // Items (商品 master)
        .route(
            "/api/v1/items",
            axum::routing::post(item_handler::create_item_handler),
        )
        .route(
            "/api/v1/items/{id}",
            axum::routing::put(item_handler::update_item_handler)
                .delete(item_handler::delete_item_handler),
        )
        // Inventory checks
        .route(
            "/api/v1/inventory/checks",
            axum::routing::post(check_handler::create_check_handler),
        )
        .route(
            "/api/v1/inventory/checks/{id}/complete",
            axum::routing::post(check_handler::complete_check_handler),
        )
        .route(
            "/api/v1/inventory/checks/{check_id}/items/{item_id}",
            axum::routing::put(check_handler::submit_check_item_handler),
        )
        // Batch inbound
        .route(
            "/api/v1/inbound-records/batch",
            axum::routing::post(inbound_handler::batch_create_inbound_handler),
        )
        .route_layer(middleware::from_fn(|req, next| {
            crate::middleware::rbac::require_role(req, next, &["admin", "warehouse"])
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
        .route_layer(middleware::from_fn(|req, next| {
            crate::middleware::rbac::require_role(req, next, &["admin", "warehouse", "sales"])
        }))
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ))
}

// Main app builder — assembles all route groups, middleware, and shared layers

pub fn create_app(
    pool: SqlitePool,
    jwt_secret: String,
    cors_origins: Vec<HeaderValue>,
    cache_manager: CacheManager,
) -> Router {
    // Public: no auth required
    let public = Router::new()
        .route(
            "/api/v1/health",
            axum::routing::get(health_handler::health_handler),
        )
        .route(
            "/api/v1/health/ready",
            axum::routing::get(health_handler::readiness_handler),
        );

    let public_auth = Router::new()
        .route(
            "/api/v1/auth/login",
            axum::routing::post(auth_handler::login_handler),
        )
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
            axum::routing::get(auth_handler::me_handler)
                .put(auth_handler::update_own_profile_handler),
        )
        .route(
            "/api/v1/users/{id}/change-password",
            axum::routing::post(auth_handler::change_password_handler),
        )
        .route_layer(middleware::from_fn(rate_limit_password_change))
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    // RBAC admin (roles, permissions, departments, user-role binding)
    let rbac_routes: axum::Router = Router::new()
        .route(
            "/api/v1/auth/permissions",
            axum::routing::get(crate::auth::handlers::list_permissions),
        )
        .route(
            "/api/v1/auth/roles",
            axum::routing::get(crate::auth::handlers::list_roles)
                .post(crate::auth::handlers::create_role),
        )
        .route(
            "/api/v1/auth/roles/{id}",
            axum::routing::put(crate::auth::handlers::update_role)
                .delete(crate::auth::handlers::delete_role),
        )
        .route(
            "/api/v1/auth/roles/{id}/permissions",
            axum::routing::get(crate::auth::handlers::get_role_permissions)
                .put(crate::auth::handlers::set_role_permissions),
        )
        .route(
            "/api/v1/auth/departments",
            axum::routing::get(crate::auth::handlers::list_departments)
                .post(crate::auth::handlers::create_department),
        )
        .route(
            "/api/v1/auth/departments/{id}",
            axum::routing::put(crate::auth::handlers::update_department)
                .delete(crate::auth::handlers::delete_department),
        )
        .route(
            "/api/v1/auth/tenants/{id}",
            axum::routing::get(crate::auth::handlers::get_tenant),
        )
        .route(
            "/api/v1/auth/users/{user_id}/roles",
            axum::routing::put(crate::auth::handlers::assign_user_roles)
                .get(crate::auth::handlers::get_user_roles),
        )
        .route(
            "/api/v1/auth/users/{user_id}/permissions",
            axum::routing::get(crate::auth::handlers::get_user_permissions),
        )
        .route_layer(middleware::from_fn(|req, next| {
            crate::middleware::rbac::require_role(req, next, &["admin"])
        }))
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    // Workflow definition management (admin) + task operations (any authenticated user)
    let workflow_routes = Router::new()
        .route(
            "/api/v1/workflows/definitions",
            axum::routing::get(crate::workflow::handlers::list_definitions)
                .post(crate::workflow::handlers::create_definition),
        )
        .route(
            "/api/v1/workflows/definitions/{id}",
            axum::routing::get(crate::workflow::handlers::get_definition)
                .put(crate::workflow::handlers::update_definition)
                .delete(crate::workflow::handlers::delete_definition),
        )
        .route(
            "/api/v1/workflows/instances",
            axum::routing::post(crate::workflow::handlers::start_instance),
        )
        .route(
            "/api/v1/workflows/my-tasks",
            axum::routing::get(crate::workflow::handlers::my_tasks),
        )
        .route(
            "/api/v1/workflows/tasks/{node_id}",
            axum::routing::get(crate::workflow::handlers::get_task),
        )
        .route(
            "/api/v1/workflows/tasks/{node_id}/approve",
            axum::routing::post(crate::workflow::handlers::approve_task),
        )
        .route(
            "/api/v1/workflows/tasks/{node_id}/reject",
            axum::routing::post(crate::workflow::handlers::reject_task),
        )
        .route(
            "/api/v1/workflows/delegations",
            axum::routing::post(crate::workflow::handlers::delegate_task),
        )
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    // HR module (admin)
    let hr_routes: axum::Router = Router::new()
        .route(
            "/api/v1/hr/employees",
            axum::routing::get(crate::hr::handlers::list_employees)
                .post(crate::hr::handlers::create_employee),
        )
        .route(
            "/api/v1/hr/employees/{id}",
            axum::routing::get(crate::hr::handlers::get_employee)
                .put(crate::hr::handlers::update_employee)
                .delete(crate::hr::handlers::delete_employee),
        )
        .route(
            "/api/v1/hr/employees/{id}/terminate",
            axum::routing::post(crate::hr::handlers::terminate_employee),
        )
        .route(
            "/api/v1/hr/employees/{id}/contracts",
            axum::routing::get(crate::hr::handlers::list_contracts),
        )
        .route(
            "/api/v1/hr/contracts",
            axum::routing::post(crate::hr::handlers::create_contract),
        )
        .route(
            "/api/v1/hr/positions",
            axum::routing::get(crate::hr::handlers::list_positions)
                .post(crate::hr::handlers::create_position),
        )
        .route(
            "/api/v1/hr/attendance",
            axum::routing::get(crate::hr::handlers::list_attendance),
        )
        .route(
            "/api/v1/hr/attendance/check-in",
            axum::routing::post(crate::hr::handlers::check_in),
        )
        .route(
            "/api/v1/hr/attendance/rules",
            axum::routing::get(crate::hr::handlers::list_rules),
        )
        .route(
            "/api/v1/hr/salaries",
            axum::routing::get(crate::hr::handlers::list_salaries)
                .post(crate::hr::handlers::generate_salaries),
        )
        .route(
            "/api/v1/hr/salaries/{id}",
            axum::routing::get(crate::hr::handlers::get_salary),
        )
        .route_layer(middleware::from_fn(|req, next| {
            crate::middleware::rbac::require_role(req, next, &["admin"])
        }))
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    // Finance module (admin): accounts, journal entries, invoices, payments, reports
    let finance_routes: axum::Router = Router::new()
        .route(
            "/api/v1/chart-of-accounts",
            axum::routing::get(crate::finance::handlers::list_accounts)
                .post(crate::finance::handlers::create_account),
        )
        .route(
            "/api/v1/chart-of-accounts/{id}",
            axum::routing::put(crate::finance::handlers::update_account),
        )
        .route(
            "/api/v1/journal-entries",
            axum::routing::get(crate::finance::handlers::list_journal_entries)
                .post(crate::finance::handlers::create_journal_entry),
        )
        .route(
            "/api/v1/journal-entries/{id}",
            axum::routing::get(crate::finance::handlers::get_journal_entry),
        )
        .route(
            "/api/v1/finance/trial-balance",
            axum::routing::get(crate::finance::handlers::trial_balance),
        )
        .route(
            "/api/v1/invoices",
            axum::routing::get(crate::finance::handlers::list_invoices)
                .post(crate::finance::handlers::create_invoice),
        )
        .route(
            "/api/v1/invoices/{id}",
            axum::routing::get(crate::finance::handlers::get_invoice),
        )
        .route(
            "/api/v1/invoices/{id}/confirm",
            axum::routing::post(crate::finance::handlers::confirm_invoice),
        )
        .route(
            "/api/v1/invoices/{id}/void",
            axum::routing::post(crate::finance::handlers::void_invoice),
        )
        .route(
            "/api/v1/payments",
            axum::routing::get(crate::finance::handlers::list_payments)
                .post(crate::finance::handlers::create_payment),
        )
        .route_layer(middleware::from_fn(|req, next| {
            crate::middleware::rbac::require_role(req, next, &["admin"])
        }))
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    // Procurement module (admin): requisitions, goods receipts, supplier quotes
    let procurement_routes: axum::Router = Router::new()
        .route(
            "/api/v1/purchase-requisitions",
            axum::routing::get(crate::procurement::handlers::list_requisitions)
                .post(crate::procurement::handlers::create_requisition),
        )
        .route(
            "/api/v1/purchase-requisitions/{id}",
            axum::routing::get(crate::procurement::handlers::get_requisition)
                .put(crate::procurement::handlers::update_requisition_status),
        )
        .route(
            "/api/v1/po-receipts",
            axum::routing::get(crate::procurement::handlers::list_receipts)
                .post(crate::procurement::handlers::create_receipt),
        )
        .route(
            "/api/v1/po-receipts/{id}",
            axum::routing::get(crate::procurement::handlers::get_receipt),
        )
        .route(
            "/api/v1/supplier-quotes",
            axum::routing::get(crate::procurement::handlers::list_quotes)
                .post(crate::procurement::handlers::create_quote),
        )
        .route(
            "/api/v1/supplier-quotes/{id}/status",
            axum::routing::put(crate::procurement::handlers::update_quote_status),
        )
        .route(
            "/api/v1/suppliers/{supplier_id}/scorecard",
            axum::routing::get(crate::procurement::handlers::supplier_scorecard),
        )
        .route_layer(middleware::from_fn(|req, next| {
            crate::middleware::rbac::require_role(req, next, &["admin"])
        }))
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    // Sales CRM (admin): shipments, quotes, customer credit
    let sales_crm_routes: axum::Router = Router::new()
        .route(
            "/api/v1/shipments",
            axum::routing::get(crate::sales_crm::handlers::list_shipments)
                .post(crate::sales_crm::handlers::create_shipment),
        )
        .route(
            "/api/v1/shipments/{id}/status",
            axum::routing::put(crate::sales_crm::handlers::update_shipment_status),
        )
        .route(
            "/api/v1/sales-quotes",
            axum::routing::get(crate::sales_crm::handlers::list_quotes)
                .post(crate::sales_crm::handlers::create_quote),
        )
        .route(
            "/api/v1/sales-quotes/{id}",
            axum::routing::get(crate::sales_crm::handlers::get_quote),
        )
        .route(
            "/api/v1/sales-quotes/{id}/status",
            axum::routing::put(crate::sales_crm::handlers::update_quote_status),
        )
        .route(
            "/api/v1/sales-quotes/{id}/convert",
            axum::routing::post(crate::sales_crm::handlers::convert_quote),
        )
        .route(
            "/api/v1/customers/{customer_id}/credit",
            axum::routing::get(crate::sales_crm::handlers::customer_credit),
        )
        .route_layer(middleware::from_fn(|req, next| {
            crate::middleware::rbac::require_role(req, next, &["admin"])
        }))
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    // Inventory ATP (admin): reservations, transfers, cycle counts
    let inventory_atp_routes: axum::Router = Router::new()
        .route(
            "/api/v1/inventory/atp/overview",
            axum::routing::get(crate::inventory_atp::handlers::overview),
        )
        .route(
            "/api/v1/inventory/reservations",
            axum::routing::post(crate::inventory_atp::handlers::reserve),
        )
        .route(
            "/api/v1/inventory/reservations/{id}/release",
            axum::routing::post(crate::inventory_atp::handlers::release),
        )
        .route(
            "/api/v1/inventory/transfers",
            axum::routing::get(crate::inventory_atp::handlers::list_transfers)
                .post(crate::inventory_atp::handlers::create_transfer),
        )
        .route(
            "/api/v1/inventory/count-templates",
            axum::routing::get(crate::inventory_atp::handlers::list_count_templates)
                .post(crate::inventory_atp::handlers::create_count_template),
        )
        .route(
            "/api/v1/inventory/count-templates/{template_id}/start",
            axum::routing::post(crate::inventory_atp::handlers::start_count_session),
        )
        .route(
            "/api/v1/inventory/count-sessions",
            axum::routing::get(crate::inventory_atp::handlers::list_count_sessions)
                .post(crate::inventory_atp::handlers::complete_count_session),
        )
        .route_layer(middleware::from_fn(|req, next| {
            crate::middleware::rbac::require_role(req, next, &["admin"])
        }))
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    // Manufacturing module (admin): BOMs, work orders, inspections, NCRs
    let manufacturing_routes: axum::Router = Router::new()
        .route(
            "/api/v1/manufacturing/boms",
            axum::routing::get(crate::manufacturing::handlers::list_boms)
                .post(crate::manufacturing::handlers::create_bom),
        )
        .route(
            "/api/v1/manufacturing/boms/{id}",
            axum::routing::get(crate::manufacturing::handlers::get_bom),
        )
        .route(
            "/api/v1/manufacturing/work-orders",
            axum::routing::get(crate::manufacturing::handlers::list_work_orders)
                .post(crate::manufacturing::handlers::create_work_order),
        )
        .route(
            "/api/v1/manufacturing/work-orders/{id}",
            axum::routing::get(crate::manufacturing::handlers::get_work_order),
        )
        .route(
            "/api/v1/manufacturing/work-orders/{id}/start",
            axum::routing::post(crate::manufacturing::handlers::start_work_order),
        )
        .route(
            "/api/v1/manufacturing/work-orders/{id}/complete-step",
            axum::routing::post(crate::manufacturing::handlers::complete_step),
        )
        .route(
            "/api/v1/manufacturing/inspections",
            axum::routing::get(crate::manufacturing::handlers::list_inspections)
                .post(crate::manufacturing::handlers::create_inspection),
        )
        .route(
            "/api/v1/manufacturing/ncrs",
            axum::routing::get(crate::manufacturing::handlers::list_ncrs)
                .post(crate::manufacturing::handlers::create_ncr),
        )
        .route(
            "/api/v1/manufacturing/ncrs/{id}/resolve",
            axum::routing::post(crate::manufacturing::handlers::resolve_ncr),
        )
        .route_layer(middleware::from_fn(|req, next| {
            crate::middleware::rbac::require_role(req, next, &["admin"])
        }))
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    // Project management (admin): projects, WBS, budget transactions
    let project_routes: axum::Router = Router::new()
        .route(
            "/api/v1/projects",
            axum::routing::get(crate::project::handlers::list_projects)
                .post(crate::project::handlers::create_project),
        )
        .route(
            "/api/v1/projects/{id}",
            axum::routing::get(crate::project::handlers::get_project)
                .put(crate::project::handlers::update_project_status),
        )
        .route(
            "/api/v1/projects/{id}/wbs",
            axum::routing::get(crate::project::handlers::wbs_tree)
                .post(crate::project::handlers::create_wbs),
        )
        .route(
            "/api/v1/projects/{project_id}/wbs/{id}",
            axum::routing::put(crate::project::handlers::update_wbs_progress),
        )
        .route(
            "/api/v1/projects/{id}/financials",
            axum::routing::get(crate::project::handlers::financials),
        )
        .route(
            "/api/v1/projects/{id}/transactions",
            axum::routing::get(crate::project::handlers::list_transactions)
                .post(crate::project::handlers::create_transaction),
        )
        .route_layer(middleware::from_fn(|req, next| {
            crate::middleware::rbac::require_role(req, next, &["admin"])
        }))
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    // Fixed assets (admin): registration, depreciation, disposal
    let assets_routes: axum::Router = Router::new()
        .route(
            "/api/v1/assets",
            axum::routing::get(crate::assets::handlers::list_assets)
                .post(crate::assets::handlers::create_asset),
        )
        .route(
            "/api/v1/assets/{id}",
            axum::routing::get(crate::assets::handlers::get_asset)
                .put(crate::assets::handlers::update_asset),
        )
        .route(
            "/api/v1/assets/{id}/depreciate",
            axum::routing::post(crate::assets::handlers::depreciate),
        )
        .route(
            "/api/v1/assets/{id}/dispose",
            axum::routing::post(crate::assets::handlers::dispose_asset),
        )
        .route_layer(middleware::from_fn(|req, next| {
            crate::middleware::rbac::require_role(req, next, &["admin"])
        }))
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    // Notifications (any authenticated user): own inbox + preferences
    let notification_routes: axum::Router = Router::new()
        .route(
            "/api/v1/notifications",
            axum::routing::get(crate::notification::handlers::list_notifications)
                .post(crate::notification::handlers::send_notification),
        )
        .route(
            "/api/v1/notifications/unread-count",
            axum::routing::get(crate::notification::handlers::unread_count),
        )
        .route(
            "/api/v1/notifications/{id}/read",
            axum::routing::post(crate::notification::handlers::mark_read),
        )
        .route(
            "/api/v1/notifications/preferences",
            axum::routing::get(crate::notification::handlers::list_preferences)
                .put(crate::notification::handlers::update_preference),
        )
        .route(
            "/api/v1/notifications/templates",
            axum::routing::post(crate::notification::handlers::create_template),
        )
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    // Portal admin (admin): create portal accounts
    let portal_admin_routes: axum::Router = Router::new()
        .route(
            "/api/v1/portal/accounts",
            axum::routing::post(crate::portal::handlers::create_account),
        )
        .route_layer(middleware::from_fn(|req, next| {
            crate::middleware::rbac::require_role(req, next, &["admin"])
        }))
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    // Portal API (portal JWT, no internal auth)
    let portal_api_routes: axum::Router = Router::new()
        .route(
            "/api/v1/portal-api/login",
            axum::routing::post(crate::portal::handlers::portal_login),
        )
        .route(
            "/api/v1/portal-api/purchases",
            axum::routing::get(crate::portal::handlers::portal_purchases),
        )
        .route(
            "/api/v1/portal-api/purchases/{id}/accept",
            axum::routing::post(crate::portal::handlers::portal_accept_purchase),
        )
        .route(
            "/api/v1/portal-api/sales",
            axum::routing::get(crate::portal::handlers::portal_sales),
        )
        .route(
            "/api/v1/portal-api/sales/{id}/acknowledge",
            axum::routing::post(crate::portal::handlers::portal_acknowledge_sales),
        )
        .route(
            "/api/v1/portal-api/events",
            axum::routing::get(crate::portal::handlers::portal_events),
        );

    // BI analytics (admin)
    let bi_routes: axum::Router = Router::new()
        .route(
            "/api/v1/bi/sales-trend",
            axum::routing::get(crate::bi::handlers::sales_trend),
        )
        .route(
            "/api/v1/bi/inventory-value",
            axum::routing::get(crate::bi::handlers::inventory_value),
        )
        .route(
            "/api/v1/bi/finance-summary",
            axum::routing::get(crate::bi::handlers::finance_summary),
        )
        .route(
            "/api/v1/bi/supplier-performance",
            axum::routing::get(crate::bi::handlers::supplier_performance),
        )
        .route_layer(middleware::from_fn(|req, next| {
            crate::middleware::rbac::require_role(req, next, &["admin"])
        }))
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

    // Inventory read (GET)
    let inventory_read = Router::new()
        .route(
            "/api/v1/inbound-records",
            axum::routing::get(inbound_handler::list_inbound_handler),
        )
        .route(
            "/api/v1/inbound-records/{id}",
            axum::routing::get(inbound_handler::get_inbound_handler),
        )
        .route(
            "/api/v1/outbound-records",
            axum::routing::get(outbound_handler::list_outbound_handler),
        )
        .route(
            "/api/v1/outbound-records/{id}",
            axum::routing::get(outbound_handler::get_outbound_handler),
        )
        .route(
            "/api/v1/inventory",
            axum::routing::get(inventory_handler::list_inventory_handler),
        )
        .route(
            "/api/v1/inventory/logs",
            axum::routing::get(inventory_handler::list_inventory_logs_handler),
        )
        // Items (商品 master) — read
        .route(
            "/api/v1/items/search",
            axum::routing::get(item_handler::search_items_handler),
        )
        .route(
            "/api/v1/items",
            axum::routing::get(item_handler::list_items_handler),
        )
        .route(
            "/api/v1/items/{id}",
            axum::routing::get(item_handler::get_item_handler),
        )
        .route(
            "/api/v1/locations",
            axum::routing::get(location_handler::list_locations_handler),
        )
        .route(
            "/api/v1/locations/{id}",
            axum::routing::get(location_handler::get_location_handler),
        )
        .route(
            "/api/v1/inventory/checks",
            axum::routing::get(check_handler::list_checks_handler),
        )
        .route(
            "/api/v1/inventory/checks/{id}",
            axum::routing::get(check_handler::get_check_handler),
        )
        .route(
            "/api/v1/trace/items/{item_id}",
            axum::routing::get(inventory_handler::trace_item_handler),
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
            axum::routing::get(inbound_handler::list_inbound_items_handler),
        )
        .route(
            "/api/v1/outbound-records/{id}/items",
            axum::routing::get(outbound_handler::list_outbound_items_handler),
        )
        .route(
            "/api/v1/atp",
            axum::routing::get(atp_handler::check_atp_handler),
        )
        .route(
            "/api/v1/inventory/inbound/search",
            axum::routing::get(inbound_handler::list_inbound_handler),
        )
        .route(
            "/api/v1/inventory/outbound/search",
            axum::routing::get(outbound_handler::list_outbound_handler),
        )
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    let data_io_template_read = Router::new()
        .route(
            "/api/v1/data-io/templates/{entity_type}",
            axum::routing::get(data_io_handler::template_handler),
        )
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    let data_io_export_read = Router::new()
        .route(
            "/api/v1/data-io/export/{entity_type}",
            axum::routing::get(data_io_handler::export_handler),
        )
        .route_layer(middleware::from_fn(|req, next| {
            crate::middleware::rbac::require_role(req, next, &["admin", "warehouse", "sales"])
        }))
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    let data_io_log_read = Router::new()
        .route(
            "/api/v1/data-io/operation-logs",
            axum::routing::get(data_io_handler::list_operation_logs_handler),
        )
        .route_layer(middleware::from_fn(|req, next| {
            crate::middleware::rbac::require_role(req, next, &["admin"])
        }))
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
        .route(
            "/api/v1/purchase-orders/search",
            axum::routing::get(purchase_handler::list_purchase_orders_handler),
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
        .route(
            "/api/v1/sales-orders/search",
            axum::routing::get(sales_handler::list_sales_orders_handler),
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

    Router::new()
        // Public (no auth required)
        .merge(public)
        .merge(public_auth)
        // Authenticated (any role)
        .merge(authenticated)
        // RBAC admin (roles, permissions, departments, user-role binding)
        .merge(rbac_routes)
        // Workflow (definitions admin, tasks any authenticated)
        .merge(workflow_routes)
        // HR module (admin)
        .merge(hr_routes)
        // Finance module (admin)
        .merge(finance_routes)
        // Procurement module (admin)
        .merge(procurement_routes)
        // Sales CRM (admin)
        .merge(sales_crm_routes)
        // Inventory ATP (admin)
        .merge(inventory_atp_routes)
        // Manufacturing module (admin): BOMs, work orders, inspections, NCRs
        .merge(manufacturing_routes)
        // Project management (admin): projects, WBS, budget transactions
        .merge(project_routes)
        // Fixed assets (admin)
        .merge(assets_routes)
        // Notifications (any authenticated user)
        .merge(notification_routes)
        // Portal admin (admin): create portal accounts
        .merge(portal_admin_routes)
        // BI analytics (admin)
        .merge(bi_routes)
        // Portal API (portal JWT — no internal auth)
        .merge(portal_api_routes)
        // Admin read
        .merge(admin_read)
        // Business read-only (all authenticated users)
        .merge(inventory_read)
        .merge(data_io_template_read)
        .merge(data_io_export_read)
        .merge(data_io_log_read)
        .merge(supplier_read)
        .merge(customer_read)
        .merge(purchase_read)
        .merge(sales_read)
        .merge(contract_read)
        .merge(report_routes)
        // Write-protected (role-checked)
        .merge(admin_write_routes())
        .merge(warehouse_write_routes())
        .merge(sales_write_routes())
        .merge(purchases_write_routes())
        .merge(supplier_customer_write_routes())
        .merge(contract_write_routes())
        // Shared layers — outermost (applied first)
        .layer(axum::middleware::from_fn(security_headers))
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin(cors_origins)
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
                .set_x_request_id(MakeRequestUuid)
                .layer(tower_http::trace::TraceLayer::new_for_http())
                .propagate_x_request_id(),
        )
        .layer(axum::Extension(pool))
        .layer(axum::Extension(JwtSecret(jwt_secret)))
        .layer(axum::Extension(RateLimiter::new()))
        .layer(axum::Extension(cache_manager))
        
}
