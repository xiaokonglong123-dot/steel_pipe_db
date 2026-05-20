use sqlx::SqlitePool;

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(include_str!("../../migrations/001_init.sql"))
        .execute(pool)
        .await?;
    sqlx::query(include_str!("../../migrations/002_inventory_check.sql"))
        .execute(pool)
        .await?;
    sqlx::query(include_str!("../../migrations/003_label_print.sql"))
        .execute(pool)
        .await?;
    Ok(())
}
