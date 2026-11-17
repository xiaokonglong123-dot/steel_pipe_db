//! [`steelpipe_api::DbProbe`] 的 SQLite 实现。

use std::time::Instant;

use async_trait::async_trait;
use steelpipe_api::DbProbe;
use steelpipe_infra::{DbPool, InfraError};

pub struct DbPingProbe {
    pool: DbPool,
}

impl DbPingProbe {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DbProbe for DbPingProbe {
    async fn ping(&self) -> Result<u64, String> {
        let started = Instant::now();
        steelpipe_infra::ping(&self.pool)
            .await
            .map_err(|e| InfraError::Connect(e).to_string())?;
        Ok(started.elapsed().as_millis() as u64)
    }
}
