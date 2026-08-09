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
            Query(query): Query<crate::parties::supplier_handler::SearchQuery>,
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
/// Macro to generate common CRUD service methods for party-like entities (suppliers, customers).
///
/// Generates: `create`, `update`, `delete`, `get`, `list`, `search`, `list_active` methods.
/// Each method delegates to the corresponding repo method and converts errors.
macro_rules! party_service {
    (
        service_name: $service:ident,
        model: $model:ident,
        repo: $repo:ident,
        create_dto: $create_dto:ident,
        update_dto: $update_dto:ident,
        filter: $filter:ident,
        code_field: $code_field:ident,
        code_dup_error: $code_dup_error:ident,
        not_found_error: $not_found_error:ident,
        prefix: $prefix:expr
    ) => {
        pub struct $service;

        impl $service {
            fn generate_code() -> String {
                let serial = uuid::Uuid::new_v4().to_string();
                format!("{}{}", $prefix, &serial[..8])
            }

            pub async fn create(
                pool: &sqlx::SqlitePool,
                dto: &$create_dto,
            ) -> Result<$model, crate::error::AppError> {
                let code = match &dto.$code_field {
                    Some(c) if !c.is_empty() => {
                        if $repo::find_by_code(pool, c)
                            .await
                            .map_err(crate::error::AppError::from)?
                            .is_some()
                        {
                            return Err(crate::error::AppError::$code_dup_error(format!(
                                "{} code '{}' already exists",
                                stringify!($model), c
                            )));
                        }
                        c.clone()
                    }
                    _ => Self::generate_code(),
                };

                $repo::create(pool, dto, &code)
                    .await
                    .map_err(|e| {
                        if let sqlx::Error::Database(ref db_err) = e {
                            if db_err.message().contains("UNIQUE constraint failed") {
                                return crate::error::AppError::$code_dup_error(format!(
                                    "{} code '{}' already exists",
                                    stringify!($model),
                                    code
                                ));
                            }
                        }
                        crate::error::AppError::from(e)
                    })
            }

            pub async fn update(
                pool: &sqlx::SqlitePool,
                id: i64,
                dto: &$update_dto,
            ) -> Result<$model, crate::error::AppError> {
                $repo::update(pool, id, dto)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => {
                            crate::error::AppError::$not_found_error(format!(
                                "{} id={} not found",
                                stringify!($model),
                                id
                            ))
                        }
                        other => crate::error::AppError::from(other),
                    })
            }

            pub async fn delete(pool: &sqlx::SqlitePool, id: i64) -> Result<(), crate::error::AppError> {
                let existing = $repo::find_by_id(pool, id)
                    .await
                    .map_err(crate::error::AppError::from)?
                    .ok_or_else(|| crate::error::AppError::$not_found_error(format!(
                        "{} id={} not found", stringify!($model), id
                    )))?;

                if existing.deleted_at.is_some() {
                    return Err(crate::error::AppError::$not_found_error(format!(
                        "{} id={} has been deleted", stringify!($model), id
                    )));
                }

                $repo::delete(pool, id).await.map_err(crate::error::AppError::from)
            }

            pub async fn get(pool: &sqlx::SqlitePool, id: i64) -> Result<$model, crate::error::AppError> {
                $repo::find_by_id(pool, id)
                    .await
                    .map_err(crate::error::AppError::from)?
                    .ok_or_else(|| crate::error::AppError::$not_found_error(format!(
                        "{} id={} not found", stringify!($model), id
                    )))
            }

            pub async fn list(
                pool: &sqlx::SqlitePool,
                filter: &$filter,
                params: &crate::dto::common::PaginationParams,
            ) -> Result<(Vec<$model>, u64), crate::error::AppError> {
                $repo::list(pool, filter, params)
                    .await
                    .map_err(crate::error::AppError::from)
            }

            pub async fn search(pool: &sqlx::SqlitePool, query: &str) -> Result<Vec<$model>, crate::error::AppError> {
                if query.trim().is_empty() {
                    return Err(crate::error::AppError::Validation("Search query is required".into()));
                }
                $repo::search(pool, query)
                    .await
                    .map_err(crate::error::AppError::from)
            }

            pub async fn list_active(pool: &sqlx::SqlitePool) -> Result<Vec<$model>, crate::error::AppError> {
                $repo::find_all_active(pool)
                    .await
                    .map_err(crate::error::AppError::from)
            }
        }
    };
}

pub(crate) use party_service;

