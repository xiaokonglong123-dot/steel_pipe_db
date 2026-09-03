//! inventory_repo — 按子域拆分 (ADR-R2)：balance / log / inbound / outbound / stock，对外路径经 re-export 保持不变

mod balance;
pub use balance::*;
mod log;
pub use log::*;
mod inbound;
pub use inbound::*;
mod outbound;
pub use outbound::*;
mod stock;
pub use stock::*;
