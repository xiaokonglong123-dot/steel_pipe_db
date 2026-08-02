//! Auth domain — RBAC (roles, permissions, tenants, departments) plus
//! identity services. Authentication (login/refresh/logout) lives in
//! `src/services/auth_service.rs` and is planned to merge here later.

pub mod handlers;
pub mod repos;
pub mod services;
