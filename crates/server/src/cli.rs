//! 命令行接口。子命令与设计文档 §16 的运维脚本一一对应。

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "steelpipe",
    version,
    about = "无缝钢管与筛管库存系统（单二进制 + PostgreSQL）"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// 启动 HTTP 服务（前端已内嵌）
    Serve,
    /// 输出 OpenAPI 文档（前端类型生成 / CI 漂移检查）
    Openapi,
    /// 审计链操作
    Audit {
        #[command(subcommand)]
        cmd: AuditCmd,
    },
    /// 库存派生列（used_count）对账
    Reconcile,
}

#[derive(Debug, Subcommand)]
pub enum AuditCmd {
    /// 分批校验哈希链完整性（默认 1000 行/批）
    Verify {
        #[arg(long, default_value_t = 1000)]
        batch: usize,
    },
}
