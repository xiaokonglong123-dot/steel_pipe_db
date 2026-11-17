//! SteelPipe 单二进制入口。

mod assets;
mod cli;
mod config;
mod probe;
mod serve;

use anyhow::Context;
use clap::Parser;

use crate::cli::{AuditCmd, Cli, Command};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Serve => serve::run().await,
        Command::Openapi => {
            let json = steelpipe_api::openapi_json_string().context("生成 OpenAPI 失败")?;
            println!("{json}");
            Ok(())
        }
        Command::Audit { cmd } => match cmd {
            AuditCmd::Verify { batch } => anyhow::bail!(
                "M5 实现：审计链分批校验（batch = {batch}）。当前仅有 `/api` 健康检查可用。"
            ),
        },
        Command::Reconcile => anyhow::bail!("M5 实现：used_count 对账"),
    }
}
