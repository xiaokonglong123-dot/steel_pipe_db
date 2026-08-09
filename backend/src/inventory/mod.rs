//! Inventory domain — stock, inbound/outbound, locations, checks, trace, legacy ATP.

pub mod atp_handler;
pub mod check_handler;
pub mod check_repo;
pub mod check_service;
pub mod inbound_handler;
pub mod inbound_repo;
pub mod inbound_service;
pub mod inventory_handler;
pub mod inventory_log_repo;
pub mod inventory_query_service;
pub mod inventory_repo;
pub mod location_handler;
pub mod location_repo;
pub mod location_service;
pub mod outbound_handler;
pub mod outbound_repo;
pub mod outbound_service;
pub mod trace_service;
