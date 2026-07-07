#![allow(dead_code)]

//! Steel Pipe DB — backend crate for the API 5CT pipe inventory management system.
//!
//! Dependency injection is via Axum [`Extension`] layers — no global state struct.

pub mod cache;
pub mod cache_invalidator;
pub mod config;
pub mod domain;
pub mod dto;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod repositories;
pub mod response;
pub mod router;
pub mod services;
