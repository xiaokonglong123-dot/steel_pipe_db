use axum::{middleware, Router};
use sqlx::SqlitePool;

use crate::handlers::auth_handler;
use crate::handlers::inventory_handler;
use crate::handlers::pipe_handler;

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

    Router::new()
        .merge(public_auth)
        .merge(authenticated)
        .merge(pipe_routes)
        .merge(admin_routes)
        .merge(inventory_routes)
        .layer(tower_http::cors::CorsLayer::permissive())
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(axum::Extension(pool))
        .layer(axum::Extension(jwt_secret))
}
