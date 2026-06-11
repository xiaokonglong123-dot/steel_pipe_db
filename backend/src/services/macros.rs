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
                    .map_err(crate::error::AppError::from)
            }

            pub async fn update(
                pool: &sqlx::SqlitePool,
                id: i64,
                dto: &$update_dto,
            ) -> Result<$model, crate::error::AppError> {
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

                $repo::update(pool, id, dto)
                    .await
                    .map_err(crate::error::AppError::from)
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
