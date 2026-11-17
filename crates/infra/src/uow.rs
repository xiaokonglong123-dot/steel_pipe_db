//! 写事务：`BEGIN IMMEDIATE` + 同事务审计写入。
//!
//! 设计 §12.1/§12.3：业务写与审计行必须同一事务、先取写锁。这里把两件事
//! 收成一个类型，仓储的写方法只接 `&mut Uow`，API 层拿不到裸 `DbPool` 写路径。

use sqlx::{Sqlite, SqliteConnection, Transaction};

use crate::{DbPool, InfraError, time};

/// 链尾表只允许一行。
const TIP_ROW: (i64,) = (1,);

/// 一个串行化写事务。drop 未 commit 时自动回滚。
pub struct Uow {
    tx: Transaction<'static, Sqlite>,
    audited: usize,
}

impl Uow {
    /// 取写锁并开事务（等价旧实现的 `BEGIN IMMEDIATE`）。
    pub async fn begin(pool: &DbPool) -> Result<Self, InfraError> {
        let tx = pool.begin_with(crate::WRITE_TXN_BEGIN).await?;
        Ok(Self { tx, audited: 0 })
    }

    /// 业务 SQL 的连接句柄。
    pub fn conn(&mut self) -> &mut SqliteConnection {
        &mut self.tx
    }

    pub fn audited_rows(&self) -> usize {
        self.audited
    }

    /// 写入一条审计行并推进链尾。必须在同一事务内。
    pub async fn audit(
        &mut self,
        user_id: Option<i64>,
        action: &str,
        detail: &str,
    ) -> Result<String, InfraError> {
        let (last_seq, prev): (i64, String) =
            sqlx::query_as("SELECT last_seq, last_hash FROM audit_chain_tip WHERE id = ?")
                .bind(TIP_ROW.0)
                .fetch_one(&mut *self.tx)
                .await
                .map_err(InfraError::Connect)?;

        let ts = time::now_rfc3339();
        let current = steelpipe_audit::compute_hash(&prev, action, detail, &ts);

        sqlx::query(
            "INSERT INTO audit_logs (user_id, action, detail, prev_hash, current_hash, ts, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(user_id)
        .bind(action)
        .bind(detail)
        .bind(&prev)
        .bind(&current)
        .bind(&ts)
        .bind(time::now_rfc3339())
        .execute(&mut *self.tx)
        .await
        .map_err(InfraError::Connect)?;

        sqlx::query(
            "UPDATE audit_chain_tip SET last_seq = ?, last_hash = ?, updated_at = ? WHERE id = ?",
        )
        .bind(last_seq + 1)
        .bind(&current)
        .bind(&ts)
        .bind(TIP_ROW.0)
        .execute(&mut *self.tx)
        .await
        .map_err(InfraError::Connect)?;

        self.audited += 1;
        Ok(current)
    }

    pub async fn commit(self) -> Result<usize, InfraError> {
        let audited = self.audited;
        self.tx.commit().await.map_err(InfraError::Connect)?;
        Ok(audited)
    }

    pub async fn rollback(self) -> Result<(), InfraError> {
        self.tx.rollback().await.map_err(InfraError::Connect)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::temp_pool;

    #[tokio::test]
    async fn audit_advances_tip_and_writes_one_row() {
        let (pool, _path) = temp_pool("uow-audit").await;

        let mut uow = Uow::begin(&pool).await.expect("开事务失败");
        let hash = uow.audit(Some(1), "login", "{}").await.expect("写审计失败");
        assert_eq!(uow.audited_rows(), 1);
        uow.commit().await.expect("提交失败");

        let (seq, tip): (i64, String) =
            sqlx::query_as("SELECT last_seq, last_hash FROM audit_chain_tip WHERE id = 1")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(seq, 1);
        assert_eq!(tip, hash);
        assert_eq!(count_audit(&pool).await, 1);
    }

    #[tokio::test]
    async fn drop_without_commit_rolls_back_everything() {
        let (pool, _path) = temp_pool("uow-rollback").await;
        {
            let mut uow = Uow::begin(&pool).await.unwrap();
            uow.audit(None, "stock_in", r#"{"pipe_id":"P-1"}"#)
                .await
                .unwrap();
            // 不 commit，drop
        }
        let (seq,): (i64,) = sqlx::query_as("SELECT last_seq FROM audit_chain_tip WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(seq, 0, "回滚后链尾必须不动");
        assert_eq!(count_audit(&pool).await, 0);
    }

    #[tokio::test]
    async fn concurrent_writers_serialize_into_one_chain() {
        let (pool, _path) = temp_pool("uow-concurrent").await;
        let mut tasks = Vec::new();
        for i in 0..12 {
            let pool = pool.clone();
            tasks.push(tokio::spawn(async move {
                let mut uow = Uow::begin(&pool).await.expect("开事务失败");
                uow.audit(None, "batch", &format!(r#"{{"i":{i}}}"#))
                    .await
                    .expect("写审计失败");
                uow.commit().await.expect("提交失败");
            }));
        }
        for task in tasks {
            task.await.expect("并发写 panic");
        }

        let rows: Vec<(String, String, String, String)> =
            sqlx::query_as("SELECT prev_hash, action, detail, ts FROM audit_logs ORDER BY id")
                .fetch_all(&pool)
                .await
                .unwrap();
        assert_eq!(rows.len(), 12);

        let entries: Vec<steelpipe_audit::Entry<'_>> = rows
            .iter()
            .map(|(prev, action, detail, ts)| steelpipe_audit::Entry {
                prev_hash: prev,
                action,
                detail: detail.as_str(),
                timestamp: ts,
                current_hash: "",
            })
            .collect();
        // 逐条重算，验证链连续（current_hash 由计算出，未存回视图）
        let mut expected_prev = steelpipe_audit::GENESIS_HASH.to_owned();
        for entry in &entries {
            assert_eq!(entry.prev_hash, expected_prev, "链在中间断开");
            expected_prev = steelpipe_audit::compute_hash(
                entry.prev_hash,
                entry.action,
                entry.detail,
                entry.timestamp,
            );
        }
        let (seq, tip): (i64, String) =
            sqlx::query_as("SELECT last_seq, last_hash FROM audit_chain_tip WHERE id = 1")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(seq, 12);
        assert_eq!(tip, expected_prev);
    }

    async fn count_audit(pool: &DbPool) -> i64 {
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM audit_logs")
            .fetch_one(pool)
            .await
            .unwrap()
    }
}
