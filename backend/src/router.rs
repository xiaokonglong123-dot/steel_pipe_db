use axum::{middleware, Router};
use sqlx::SqlitePool;

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

pub fn create_app(pool: SqlitePool, jwt_secret: String) -> Router {
    let public_auth = Router::new()
        .route("/api/v1/auth/login", axum::routing::post(auth_handler::login_handler))
        .route(
            "/api/v1/auth/refresh",
            axum::routing::post(auth_handler::refresh_handler),
        );

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
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    let admin_routes = Router::new()
        .route(
            "/api/v1/users",
            axum::routing::get(auth_handler::list_users_handler)
                .post(auth_handler::create_user_handler),
        )
        .route(
            "/api/v1/users/{id}",
            axum::routing::put(auth_handler::update_user_handler),
        )
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ))
        .route_layer(middleware::from_fn(|req, next| {
            crate::middleware::rbac::require_role(req, next, &["admin"])
        }));

    let pipe_routes = Router::new()
        .route(
            "/api/v1/seamless-pipes",
            axum::routing::get(pipe_handler::list_seamless_pipes_handler)
                .post(pipe_handler::create_seamless_pipe_handler),
        )
        .route(
            "/api/v1/seamless-pipes/{id}",
            axum::routing::get(pipe_handler::get_seamless_pipe_handler)
                .put(pipe_handler::update_seamless_pipe_handler)
                .delete(pipe_handler::delete_seamless_pipe_handler),
        )
        .route(
            "/api/v1/screen-pipes",
            axum::routing::get(pipe_handler::list_screen_pipes_handler)
                .post(pipe_handler::create_screen_pipe_handler),
        )
        .route(
            "/api/v1/screen-pipes/{id}",
            axum::routing::get(pipe_handler::get_screen_pipe_handler)
                .put(pipe_handler::update_screen_pipe_handler)
                .delete(pipe_handler::delete_screen_pipe_handler),
        )
        .route(
            "/api/v1/pipes/search",
            axum::routing::get(pipe_handler::search_pipes_handler),
        )
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    let inventory_routes = Router::new()
        .route(
            "/api/v1/inbound-records",
            axum::routing::post(inventory_handler::create_inbound_handler)
                .get(inventory_handler::list_inbound_handler),
        )
        .route(
            "/api/v1/inbound-records/{id}",
            axum::routing::get(inventory_handler::get_inbound_handler)
                .delete(inventory_handler::delete_inbound_handler),
        )
        .route(
            "/api/v1/inbound-records/{id}/approve",
            axum::routing::post(inventory_handler::approve_inbound_handler),
        )
        .route(
            "/api/v1/inbound-records/{id}/reject",
            axum::routing::post(inventory_handler::reject_inbound_handler),
        )
        .route(
            "/api/v1/outbound-records",
            axum::routing::post(inventory_handler::create_outbound_handler)
                .get(inventory_handler::list_outbound_handler),
        )
        .route(
            "/api/v1/outbound-records/{id}",
            axum::routing::get(inventory_handler::get_outbound_handler)
                .delete(inventory_handler::delete_outbound_handler),
        )
        .route(
            "/api/v1/outbound-records/{id}/approve",
            axum::routing::post(inventory_handler::approve_outbound_handler),
        )
        .route(
            "/api/v1/outbound-records/{id}/reject",
            axum::routing::post(inventory_handler::reject_outbound_handler),
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
            axum::routing::get(inventory_handler::list_locations_handler)
                .post(inventory_handler::create_location_handler),
        )
        .route(
            "/api/v1/locations/{id}",
            axum::routing::get(inventory_handler::get_location_handler)
                .put(inventory_handler::update_location_handler)
                .delete(inventory_handler::delete_location_handler),
        )
        .route(
            "/api/v1/inventory/checks",
            axum::routing::post(inventory_handler::create_check_handler)
                .get(inventory_handler::list_checks_handler),
        )
        .route(
            "/api/v1/inventory/checks/{id}",
            axum::routing::get(inventory_handler::get_check_handler),
        )
        .route(
            "/api/v1/inventory/checks/{check_id}/items/{item_id}",
            axum::routing::put(inventory_handler::submit_check_item_handler),
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
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    let data_io_routes = Router::new()
        .route(
            "/api/v1/data-io/import/{entity_type}",
            axum::routing::post(data_io_handler::import_handler),
        )
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

    let supplier_routes = Router::new()
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
            axum::routing::get(supplier_handler::list_suppliers_handler)
                .post(supplier_handler::create_supplier_handler),
        )
        .route(
            "/api/v1/suppliers/{id}",
            axum::routing::get(supplier_handler::get_supplier_handler)
                .put(supplier_handler::update_supplier_handler)
                .delete(supplier_handler::delete_supplier_handler),
        )
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    let customer_routes = Router::new()
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
            axum::routing::get(customer_handler::list_customers_handler)
                .post(customer_handler::create_customer_handler),
        )
        .route(
            "/api/v1/customers/{id}",
            axum::routing::get(customer_handler::get_customer_handler)
                .put(customer_handler::update_customer_handler)
                .delete(customer_handler::delete_customer_handler),
        )
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    let purchase_routes = Router::new()
        .route(
            "/api/v1/purchase-orders",
            axum::routing::get(purchase_handler::list_purchase_orders_handler)
                .post(purchase_handler::create_purchase_order_handler),
        )
        .route(
            "/api/v1/purchase-orders/{id}",
            axum::routing::get(purchase_handler::get_purchase_order_handler)
                .put(purchase_handler::update_purchase_order_handler)
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
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    let sales_routes = Router::new()
        .route(
            "/api/v1/sales-orders",
            axum::routing::get(sales_handler::list_sales_orders_handler)
                .post(sales_handler::create_sales_order_handler),
        )
        .route(
            "/api/v1/sales-orders/{id}",
            axum::routing::get(sales_handler::get_sales_order_handler)
                .put(sales_handler::update_sales_order_handler)
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
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    let quality_routes = Router::new()
        .route(
            "/api/v1/quality/certs",
            axum::routing::get(quality_handler::list_certs_handler)
                .post(quality_handler::create_cert_handler),
        )
        .route(
            "/api/v1/quality/certs/{id}",
            axum::routing::get(quality_handler::get_cert_handler)
                .put(quality_handler::update_cert_handler)
                .delete(quality_handler::delete_cert_handler),
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
            axum::routing::post(quality_handler::create_attachment_handler)
                .get(quality_handler::list_attachments_handler),
        )
        .route(
            "/api/v1/quality/attachments/{id}",
            axum::routing::delete(quality_handler::delete_attachment_handler),
        )
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    let contract_routes = Router::new()
        .route(
            "/api/v1/contracts",
            axum::routing::get(contract_handler::list_contracts_handler)
                .post(contract_handler::create_contract_handler),
        )
        .route(
            "/api/v1/contracts/{id}",
            axum::routing::get(contract_handler::get_contract_handler)
                .put(contract_handler::update_contract_handler)
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
            axum::routing::get(contract_handler::list_contract_payments_handler)
                .post(contract_handler::add_contract_payment_handler),
        )
        .route(
            "/api/v1/contracts/{contract_id}/payments/{payment_id}",
            axum::routing::put(contract_handler::update_contract_payment_handler)
                .delete(contract_handler::delete_contract_payment_handler),
        )
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

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

    let label_routes = Router::new()
        .route(
            "/api/v1/labels/pipe/{pipe_type}/{pipe_id}",
            axum::routing::get(label_handler::get_pipe_label_handler),
        )
        .route(
            "/api/v1/labels/batch",
            axum::routing::post(label_handler::create_batch_labels_handler),
        )
        .route(
            "/api/v1/labels/quality/{cert_id}",
            axum::routing::get(label_handler::get_quality_label_handler),
        )
        .route(
            "/api/v1/labels/shipping",
            axum::routing::post(label_handler::create_shipping_label_handler),
        )
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    Router::new()
        .merge(public_auth)
        .merge(authenticated)
        .merge(pipe_routes)
        .merge(admin_routes)
        .merge(inventory_routes)
        .merge(data_io_routes)
        .merge(supplier_routes)
        .merge(customer_routes)
        .merge(purchase_routes)
        .merge(sales_routes)
        .merge(quality_routes)
        .merge(contract_routes)
        .merge(report_routes)
        .merge(label_routes)
        .layer(tower_http::cors::CorsLayer::permissive())
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(axum::Extension(pool))
        .layer(axum::Extension(jwt_secret))
}
