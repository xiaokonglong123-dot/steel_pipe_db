#![allow(dead_code)]

//! ERP Server — backend crate for the generic ERP system.
//!
//! Dependency injection is via Axum [`Extension`] layers — no global state struct.

pub mod cache;
pub mod auth;
pub mod workflow;
pub mod hr;
pub mod finance;
pub mod procurement;
pub mod sales_crm;
pub mod inventory_atp;
pub mod manufacturing;
pub mod project;
pub mod bi;
pub mod portal;
pub mod notification;
pub mod assets;
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
