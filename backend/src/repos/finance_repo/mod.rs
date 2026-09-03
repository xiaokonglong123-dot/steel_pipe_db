//! finance_repo — 按子域拆分 (ADR-R2)：account/journal/invoice/payment，对外路径经 re-export 保持不变

mod account;
pub use account::*;
mod journal;
pub use journal::*;
mod invoice;
pub use invoice::*;
mod payment;
pub use payment::*;
