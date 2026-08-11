use erp_v2::services::catalog_service;
use sqlx::SqlitePool;

mod common;

async fn bootstrap(pool: &SqlitePool) {
    let _ = sqlx::query("INSERT OR IGNORE INTO users (id, username, display_name, password_hash, is_active, is_admin) VALUES (1, 'admin', 'Administrator', '$argon2id$v=19$m=19456,t=2,p=1$YWFhYQ$hello', 1, 1)").execute(pool).await;
}

#[tokio::test]
async fn import_items_csv_service_records_success_and_skip_invalid() {
    let (pool, _dir) = common::test_pool().await;
    bootstrap(&pool).await;

    let csv_text = "sku,name,category,unit,spec\nI001,商品一,工具,个,L\nI002,商品二,工具,个,M\nI003,,无效空名,,\n";
    let mut reader = csv::Reader::from_reader(std::io::Cursor::new(csv_text.as_bytes()));
    let headers: Vec<String> = reader.headers().unwrap().iter().map(|s| s.trim().to_lowercase()).collect();
    fn pick(h: &[String], aliases: &[&str]) -> Option<usize> { aliases.iter().find_map(|a| h.iter().position(|x| x == a)) }
    let i_sku = pick(&headers, &["sku", "code"]).unwrap();
    let i_name = pick(&headers, &["name", "名称"]).unwrap();
    let i_cat = pick(&headers, &["category", "分类"]);
    let i_unit = pick(&headers, &["unit", "单位"]);
    let i_spec = pick(&headers, &["spec", "specification", "规格"]);

    let mut succeeded = 0usize;
    let mut failed = 0usize;
    let mut row_no = 1usize;
    for record in reader.records() {
        row_no += 1;
        let Ok(rec) = record else { failed += 1; continue; };
        let get = |i: Option<usize>| i.and_then(|x| rec.get(x)).map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
        let sku = match get(Some(i_sku)) { Some(s) => s, None => { failed += 1; continue; } };
        let name = match get(Some(i_name)) { Some(s) => s, None => { failed += 1; continue; } };
        match catalog_service::create_item(&pool, &sku, &name, get(i_cat).as_deref(), get(i_unit).as_deref(), get(i_spec).as_deref()).await {
            Ok(_) => succeeded += 1,
            Err(_) => failed += 1,
        }
    }
    assert_eq!(row_no, 4, "处理了 3 个数据行 + 1 个标题行");
    assert_eq!(succeeded, 2, "2 行成功");
    assert_eq!(failed, 1, "1 行失败（空 name）");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM items WHERE sku IN ('I001','I002')")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(count, 2, "I001 与 I002 已写入 DB");
}

#[tokio::test]
async fn import_items_csv_dup_skus_are_reported_as_failed() {
    let (pool, _dir) = common::test_pool().await;
    bootstrap(&pool).await;

    catalog_service::create_item(&pool, "DUP", "已存在", None, None, None).await.unwrap();

    let csv_text = "sku,name\nDUP,冲突行\nNEW,新增合法\n";
    let mut reader = csv::Reader::from_reader(std::io::Cursor::new(csv_text.as_bytes()));
    let headers: Vec<String> = reader.headers().unwrap().iter().map(|s| s.trim().to_lowercase()).collect();
    let i_sku = headers.iter().position(|s| s == "sku").unwrap();
    let i_name = headers.iter().position(|s| s == "name").unwrap();

    let mut succeeded = 0usize;
    let mut failed = 0usize;
    for rec in reader.records() {
        let Ok(rec) = rec else { failed += 1; continue; };
        let sku = rec.get(i_sku).unwrap().trim().to_string();
        let name = rec.get(i_name).unwrap().trim().to_string();
        match catalog_service::create_item(&pool, &sku, &name, None, None, None).await {
            Ok(_) => succeeded += 1,
            Err(_) => failed += 1,
        }
    }
    assert_eq!(succeeded, 1, "NEW 成功");
    assert_eq!(failed, 1, "DUP 因唯一约束失败");
}
