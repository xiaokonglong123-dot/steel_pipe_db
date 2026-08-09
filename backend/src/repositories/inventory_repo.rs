use sqlx::SqlitePool;

/// Helper struct for inserting into `inventory_logs` — not a DB model.
#[derive(Debug, Clone)]
pub struct CreateInventoryLog {
    pub item_id: i64,
    pub quantity: f64,
    pub change_type: String,
    pub ref_type: Option<String>,
    pub ref_id: Option<i64>,
    pub from_location_id: Option<i64>,
    pub to_location_id: Option<i64>,
    pub notes: Option<String>,
    pub created_by: Option<i64>,
}

/// Helper struct for seeding inventory check items — not a DB model.
#[derive(Debug, Clone)]
pub struct CheckInitItem {
    pub item_id: i64,
    pub expected_quantity: f64,
}

/// On-hand quantity grouped by item category.
#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct CategoryCount {
    pub category: String,
    pub quantity: f64,
}

/// On-hand quantity grouped by location.
#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct LocationCount {
    pub location_id: Option<i64>,
    pub quantity: f64,
}

/// Inventory queries over the quantity-based schema.
///
/// On-hand stock is **derived** from `inventory_logs`: inbound/check_adjust
/// rows add quantity, outbound/transfer rows subtract it. There is no
/// materialized stock table in this schema.
pub struct InventoryRepo;

impl InventoryRepo {
    /// Signed on-hand quantity for a single item.
    /// - `change_type IN ('inbound', 'check_adjust')` adds.
    /// - `change_type IN ('outbound', 'transfer')` subtracts.
    pub async fn stock_on_hand(pool: &SqlitePool, item_id: i64) -> Result<f64, sqlx::Error> {
        let (v,): (f64,) = sqlx::query_as(
            "SELECT CAST(COALESCE(SUM(
                 CASE WHEN change_type IN ('inbound', 'check_adjust') THEN quantity
                      ELSE -quantity END) , 0.0) AS REAL) AS v
             FROM inventory_logs WHERE item_id = ?",
        )
        .bind(item_id)
        .fetch_one(pool)
        .await?;
        Ok(v)
    }

    /// Signed on-hand quantity for a single item scoped to one location.
    /// A log counts toward a location when the location appears on either side
    /// of the movement (from_location_id / to_location_id).
    pub async fn stock_on_hand_at_location(
        pool: &SqlitePool,
        item_id: i64,
        location_id: i64,
    ) -> Result<f64, sqlx::Error> {
        let (v,): (f64,) = sqlx::query_as(
            "SELECT CAST(COALESCE(SUM(
                 CASE WHEN to_location_id = ? THEN quantity
                      WHEN from_location_id = ? THEN -quantity
                      ELSE 0 END), 0.0) AS REAL) AS v
             FROM inventory_logs
             WHERE item_id = ?
               AND (from_location_id = ? OR to_location_id = ?)",
        )
        .bind(location_id)
        .bind(location_id)
        .bind(item_id)
        .bind(location_id)
        .bind(location_id)
        .fetch_one(pool)
        .await?;
        Ok(v)
    }

    /// Total signed on-hand quantity across all items.
    pub async fn get_total_in_stock(pool: &SqlitePool) -> Result<f64, sqlx::Error> {
        let (v,): (f64,) = sqlx::query_as(
            "SELECT CAST(COALESCE(SUM(
                 CASE WHEN change_type IN ('inbound', 'check_adjust') THEN quantity
                      ELSE -quantity END) , 0.0) AS REAL) AS v
             FROM inventory_logs",
        )
        .fetch_one(pool)
        .await?;
        Ok(v)
    }

    /// On-hand quantity grouped by item category (via items master join).
    pub async fn get_count_by_category(pool: &SqlitePool) -> Result<Vec<CategoryCount>, sqlx::Error> {
        sqlx::query_as::<_, CategoryCount>(
            "SELECT COALESCE(i.category, '未分类') AS category,
                    CAST(COALESCE(SUM(
                        CASE WHEN l.change_type IN ('inbound', 'check_adjust') THEN l.quantity
                             ELSE -l.quantity END) , 0.0) AS REAL) AS quantity
             FROM inventory_logs l
             JOIN items i ON i.id = l.item_id AND i.deleted_at IS NULL
             GROUP BY i.category
             ORDER BY i.category",
        )
        .fetch_all(pool)
        .await
    }

    /// On-hand quantity grouped by location.
    pub async fn get_count_by_location(pool: &SqlitePool) -> Result<Vec<LocationCount>, sqlx::Error> {
        // A log is counted at each of its endpoints; union keeps both sides.
        sqlx::query_as::<_, LocationCount>(
            "SELECT location_id, CAST(COALESCE(SUM(delta), 0.0) AS REAL) AS quantity FROM (
                 SELECT from_location_id AS location_id, -quantity AS delta
                 FROM inventory_logs WHERE from_location_id IS NOT NULL
                 UNION ALL
                 SELECT to_location_id AS location_id, quantity AS delta
                 FROM inventory_logs WHERE to_location_id IS NOT NULL
             ) GROUP BY location_id ORDER BY location_id",
        )
        .fetch_all(pool)
        .await
    }

    /// ATP rows: available quantity per item (optionally filtered by item /
    /// location). Available = on-hand - reserved.
    pub async fn find_atp(
        pool: &SqlitePool,
        item_id: &Option<i64>,
        location_id: &Option<i64>,
    ) -> Result<Vec<(i64, Option<String>, f64, Option<i64>)>, sqlx::Error> {
        let mut conditions: Vec<String> = vec!["i.deleted_at IS NULL".into()];
        let mut binds: Vec<String> = Vec::new();

        if let Some(id) = item_id {
            conditions.push("l.item_id = ?".into());
            binds.push(id.to_string());
        }
        if let Some(loc) = location_id {
            conditions.push("(l.from_location_id = ? OR l.to_location_id = ?)".into());
            binds.push(loc.to_string());
            binds.push(loc.to_string());
        }
        let where_clause = conditions.join(" AND ");

        let sql = format!(
            "SELECT i.id AS item_id, i.sku,
                    CAST(COALESCE(SUM(CASE WHEN l.change_type IN ('inbound', 'check_adjust')
                                      THEN l.quantity ELSE -l.quantity END) , 0.0) AS REAL) AS on_hand,
                    NULL AS location_id
             FROM items i
             LEFT JOIN inventory_logs l ON l.item_id = i.id
             WHERE {where_clause}
             GROUP BY i.id, i.sku
             HAVING COALESCE(SUM(CASE WHEN l.change_type IN ('inbound', 'check_adjust')
                                       THEN l.quantity ELSE -l.quantity END) , 0.0) > 0
             ORDER BY i.sku",
            where_clause = where_clause,
        );

        let mut q = sqlx::query_as::<_, (i64, Option<String>, f64, Option<i64>)>(&sql);
        for b in &binds {
            q = q.bind(b.as_str());
        }
        q.fetch_all(pool).await
    }

    /// All (item_id, on-hand quantity) pairs with positive stock, optionally
    /// scoped to a location. Used to initialize inventory checks.
    pub async fn list_stock(
        pool: &SqlitePool,
        location_id: Option<i64>,
    ) -> Result<Vec<(i64, f64)>, sqlx::Error> {
        let (sql, binds): (String, Vec<String>) = match location_id {
            Some(loc) => {
                let sql = format!(
                    "SELECT l.item_id,
                            CAST(COALESCE(SUM(CASE WHEN l.to_location_id = ? THEN l.quantity
                                     WHEN l.from_location_id = ? THEN -l.quantity
                                     ELSE 0 END), 0.0) AS REAL) AS qty
                     FROM inventory_logs l
                     WHERE (l.from_location_id = ? OR l.to_location_id = ?)
                     GROUP BY l.item_id
                     HAVING SUM(CASE WHEN l.to_location_id = ? THEN l.quantity
                                     WHEN l.from_location_id = ? THEN -l.quantity
                                     ELSE 0 END) > 0",
                );
                let b = loc.to_string();
                (sql, vec![b.clone(), b.clone(), b.clone(), b.clone(), b.clone(), b.clone()])
            }
            None => {
                let sql = "SELECT l.item_id,
                                  CAST(COALESCE(SUM(CASE WHEN l.change_type IN ('inbound', 'check_adjust')
                                           THEN l.quantity ELSE -l.quantity END), 0.0) AS REAL) AS qty
                           FROM inventory_logs l
                           GROUP BY l.item_id
                           HAVING SUM(CASE WHEN l.change_type IN ('inbound', 'check_adjust')
                                           THEN l.quantity ELSE -l.quantity END) > 0"
                    .to_string();
                (sql, Vec::new())
            }
        };

        let mut q = sqlx::query_as::<_, (i64, f64)>(&sql);
        for b in &binds {
            q = q.bind(b.as_str());
        }
        q.fetch_all(pool).await
    }
}
