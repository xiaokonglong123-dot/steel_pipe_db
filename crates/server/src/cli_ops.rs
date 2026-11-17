//! CLI 子命令的实现：审计校验、对账、标准规格种子。
//!
//! CLI 不需要 JWT 密钥，因此不加载完整 [crate::config::Config]，只读数据库地址。

use anyhow::{Context, bail};
use steelpipe_domain::Role;
use steelpipe_infra::DbPool;
use steelpipe_infra::repo::users;
use steelpipe_service::{Identity, OpsService};

fn database_url(explicit: Option<&str>) -> String {
    explicit
        .map(ToOwned::to_owned)
        .or_else(|| std::env::var("STEELPIPE_DATABASE_URL").ok())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "sqlite://data/steelpipe.db".to_owned())
}

async fn open(explicit: Option<&str>) -> anyhow::Result<DbPool> {
    let url = database_url(explicit);
    let pool = steelpipe_infra::connect(&url, 4)
        .await
        .with_context(|| format!("打开数据库失败：{url}"))?;
    steelpipe_infra::migrate(&pool)
        .await
        .context("应用迁移失败")?;
    Ok(pool)
}

/// 运维动作的操作者：优先取第一个启用的 admin，没有就退化成 `cli` 身份。
async fn actor(pool: &DbPool) -> Identity {
    if let Ok(rows) = users::list(pool).await {
        if let Some(row) = rows
            .iter()
            .find(|row| row.active && row.role().is_some_and(Role::is_admin))
        {
            return Identity {
                user_id: row.id,
                username: row.username.clone(),
                display_name: row.display_name.clone(),
                role: Role::Admin,
                token_version: row.token_version,
            };
        }
    }
    Identity {
        user_id: 0,
        username: "cli".into(),
        display_name: "运维 CLI".into(),
        role: Role::Admin,
        token_version: 0,
    }
}

pub async fn verify(database: Option<&str>, batch: usize) -> anyhow::Result<()> {
    let pool = open(database).await?;
    let summary = OpsService::new(pool.clone())
        .verify_chain(batch)
        .await
        .context("审计链校验失败")?;
    pool.close().await;
    println!(
        "审计链校验通过：{} 条 / {} 批 / tip={}",
        summary.entries, summary.batches, summary.tip
    );
    Ok(())
}

pub async fn reconcile(database: Option<&str>) -> anyhow::Result<()> {
    let pool = open(database).await?;
    let operator = actor(&pool).await;
    let fixed = OpsService::new(pool.clone())
        .reconcile(&operator)
        .await
        .context("used_count 对账失败")?;
    pool.close().await;

    if fixed.is_empty() {
        println!("used_count 无漂移");
    } else {
        for row in &fixed {
            println!("修复 {}：{} → {}", row.code, row.before, row.after);
        }
        println!("共修复 {} 个库位", fixed.len());
    }
    Ok(())
}

pub async fn seed_standards(database: Option<&str>, path: &str) -> anyhow::Result<()> {
    let csv = std::fs::read_to_string(path).with_context(|| format!("读取 {path} 失败"))?;
    if csv.trim().is_empty() {
        bail!("{path} 是空文件");
    }
    let pool = open(database).await?;
    let operator = actor(&pool).await;
    let summary = OpsService::new(pool.clone())
        .seed_standards(&operator, &csv)
        .await
        .context("种子化标准规格失败")?;
    pool.close().await;
    println!(
        "标准规格：新增 {} 条，跳过（已存在）{} 条",
        summary.inserted, summary.skipped
    );
    Ok(())
}
