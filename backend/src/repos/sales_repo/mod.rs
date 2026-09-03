//! sales_repo — 按子域拆分 (ADR-R2)：order/reservation，对外路径经 re-export 保持不变

mod order;
pub use order::*;
mod reservation;
pub use reservation::*;
