//! SteelPipe 单二进制入口。

mod assets;
mod cli;
mod cli_ops;
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
            AuditCmd::Verify { batch } => cli_ops::verify(cli.database.as_deref(), batch).await,
        },
        Command::Reconcile => cli_ops::reconcile(cli.database.as_deref()).await,
        Command::SeedStandards { path } => {
            cli_ops::seed_standards(cli.database.as_deref(), &path).await
        }
    }
}
