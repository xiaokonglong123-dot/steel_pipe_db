//! 应用层（用例）：编排 domain + auth + infra。
//!
//! 约束：不得依赖 axum —— 由 CI 的 `scripts/ci/layering.sh` 断言。

#![forbid(unsafe_code)]

pub mod actions;
pub mod auth;
pub mod bootstrap;
pub mod catalog;
pub mod inventory;
pub mod users;

pub use auth::{AuthService, AuthServiceError, Identity, LimitScope, LoginSuccess, Session};
pub use catalog::{CatalogError, CatalogService, LocationView, StandardInput, StandardView};
pub use inventory::{
    HistoryView, InventoryError, InventoryService, ScreenSpecView, SeamlessSpecView, StockInInput,
    TubeDetailView, TubePage, TubeQuery, TubeView,
};
pub use steelpipe_infra::{DbPool, InfraError};
pub use users::{NewUser, UserError, UserService, UserView};
