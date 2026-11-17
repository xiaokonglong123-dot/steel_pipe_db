//! 应用层（用例）：编排 domain + auth + infra。
//!
//! 约束：不得依赖 axum —— 由 CI 的 `scripts/ci/layering.sh` 断言。

#![forbid(unsafe_code)]

pub mod actions;
pub mod auth;
pub mod bootstrap;
pub mod catalog;
pub mod checks;
pub mod import;
pub mod inventory;
pub mod ops;
pub mod stats;
pub mod users;

pub use auth::{AuthService, AuthServiceError, Identity, LimitScope, LoginSuccess, Session};
pub use catalog::{CatalogError, CatalogService, LocationView, StandardInput, StandardView};
pub use checks::{CheckDetailView, CheckError, CheckService, CheckSessionView, ConfirmOutcome};
pub use import::{ImportError, ImportReport, ImportService, RowFailure};
pub use inventory::{
    HistoryView, InventoryError, InventoryService, ScreenSpecView, SeamlessSpecView, StockInInput,
    TubeDetailView, TubePage, TubeQuery, TubeSort, TubeView,
};
pub use ops::{
    AuditEntryView, AuditPageView, DriftFix, OpsError, OpsService, SeedSummary, VerifySummary,
};
pub use stats::{DashboardView, StatsError, StatsService};
pub use steelpipe_infra::{DbPool, InfraError};
pub use users::{NewUser, UserError, UserService, UserView};
