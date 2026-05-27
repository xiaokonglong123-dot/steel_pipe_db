#![allow(dead_code)]

//! Steel Pipe DB — backend crate for the API 5CT pipe inventory management system.
//!
//! Architecture: Handler → Service → Repository, with shared types in `domain/`,
//! request/response structs in `dto/`, and DB row mappings in `models/`.
//! Dependency injection is via Axum [`Extension`] layers — no global state struct.

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
