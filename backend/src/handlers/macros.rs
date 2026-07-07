/// Macro to generate common CRUD handler functions for party-like entities (suppliers, customers).
///
/// Generates: `list_*_handler`, `create_*_handler`, `get_*_handler`, `update_*_handler`,
/// `delete_*_handler`, `search_*_handler`, `list_active_*_handler`.
macro_rules! party_handler {
    (
        service: $service:ident,
        model: $model:ident,
        create_dto: $create_dto:ident,
        update_dto: $update_dto:ident,
        filter: $filter:ident,
        list_fn: $list_fn:ident,
        create_fn: $create_fn:ident,
        get_fn: $get_fn:ident,
        update_fn: $update_fn:ident,
        delete_fn: $delete_fn:ident,
        search_fn: $search_fn:ident,
        active_fn: $active_fn:ident
    ) => {
        use axum::extract::{Extension, Path, Query};
        use axum::response::IntoResponse;
        use axum::Json;
        use sqlx::SqlitePool;
        use validator::Validate;

        pub async fn $list_fn(
            Extension(pool): Extension<SqlitePool>,
            Query(filter): Query<$filter>,
        ) -> Result<Json<crate::response::PaginatedResponse<$model>>, crate::error::AppError> {
            let pagination = crate::dto::common::PaginationParams {
                page: filter.page,
                page_size: filter.page_size,
                sort_by: filter.sort_by.clone(),
                sort_order: filter.sort_order.clone(),
            };
            let page = pagination.page();
            let page_size = pagination.page_size();

            let (items, total) = $service::list(&pool, &filter, &pagination).await?;

            Ok(crate::response::PaginatedResponse::ok(items, total, page, page_size))
        }

        pub async fn $create_fn(
            Extension(pool): Extension<SqlitePool>,
            Json(req): Json<$create_dto>,
        ) -> Result<axum::response::Response, crate::error::AppError> {
            req.validate()
                .map_err(|e| crate::error::AppError::Validation(e.to_string()))?;
            let item = $service::create(&pool, &req).await?;
            Ok(crate::response::ApiResponse::created(item))
        }

        pub async fn $get_fn(
            Extension(pool): Extension<SqlitePool>,
            Path(id): Path<i64>,
        ) -> Result<Json<crate::response::ApiResponse<$model>>, crate::error::AppError> {
            let item = $service::get(&pool, id).await?;
            Ok(crate::response::ApiResponse::ok(item))
        }

        pub async fn $update_fn(
            Extension(pool): Extension<SqlitePool>,
            Path(id): Path<i64>,
            Json(req): Json<$update_dto>,
        ) -> Result<Json<crate::response::ApiResponse<$model>>, crate::error::AppError> {
            req.validate()
                .map_err(|e| crate::error::AppError::Validation(e.to_string()))?;
            let item = $service::update(&pool, id, &req).await?;
            Ok(crate::response::ApiResponse::ok(item))
        }

        pub async fn $delete_fn(
            Extension(pool): Extension<SqlitePool>,
            Path(id): Path<i64>,
        ) -> Result<axum::response::Response, crate::error::AppError> {
            $service::delete(&pool, id).await?;
            Ok((axum::http::StatusCode::NO_CONTENT, ()).into_response())
        }

        pub async fn $search_fn(
            Extension(pool): Extension<SqlitePool>,
            Query(query): Query<crate::handlers::supplier_handler::SearchQuery>,
        ) -> Result<Json<crate::response::ApiResponse<Vec<$model>>>, crate::error::AppError> {
            let results = $service::search(&pool, &query.q).await?;
            Ok(crate::response::ApiResponse::ok(results))
        }

        pub async fn $active_fn(
            Extension(pool): Extension<SqlitePool>,
        ) -> Result<Json<crate::response::ApiResponse<Vec<$model>>>, crate::error::AppError> {
            let items = $service::list_active(&pool).await?;
            Ok(crate::response::ApiResponse::ok(items))
        }
    };
}

pub(crate) use party_handler;
