#[cfg(test)]
mod tests {
    use crate::database::{Database, InventoryRecord, SteelPipe};
    use std::fs;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn next_test_id() -> u32 {
        TEST_COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    fn setup_test_db() -> (Database, String) {
        let test_id = next_test_id();
        let db_path = format!("test_db_{}.db", test_id);
        let _ = fs::remove_file(&db_path);
        let db = Database::new(&db_path).unwrap();
        (db, db_path)
    }

    fn cleanup_test_db(path: &str) {
        let _ = fs::remove_file(path);
    }

    fn create_test_pipe(id: &str, qty: i32) -> SteelPipe {
        SteelPipe {
            id: None,
            pipe_id: id.to_string(),
            diameter: 50.0,
            thickness: 3.0,
            length: 6.0,
            material: "碳钢".to_string(),
            quantity: qty,
            location: Some("A区".to_string()),
            supplier: Some("供应商A".to_string()),
            entry_date: String::new(),
            last_update: None,
            status: "在库".to_string(),
        }
    }

    #[test]
    fn test_new_database() {
        let (db, path) = setup_test_db();
        assert!(db.conn.lock().is_ok());
        cleanup_test_db(&path);
    }

    #[test]
    fn test_add_pipe() {
        let (db, path) = setup_test_db();
        let pipe = create_test_pipe("TEST-001", 100);
        assert!(db.add_pipe(&pipe).is_ok());
        let pipes = db.get_pipes().unwrap();
        assert_eq!(pipes.len(), 1);
        cleanup_test_db(&path);
    }

    #[test]
    fn test_add_pipe_duplicate() {
        let (db, path) = setup_test_db();
        let pipe1 = create_test_pipe("TEST-001", 100);
        let pipe2 = create_test_pipe("TEST-001", 50);
        assert!(db.add_pipe(&pipe1).is_ok());
        assert!(db.add_pipe(&pipe2).is_ok());
        let pipes = db.get_pipes().unwrap();
        assert_eq!(pipes.len(), 1);
        assert_eq!(pipes[0].quantity, 150);
        cleanup_test_db(&path);
    }

    #[test]
    fn test_add_pipe_invalid() {
        let (db, path) = setup_test_db();
        let pipe = SteelPipe {
            id: None,
            pipe_id: "".to_string(),
            diameter: 50.0,
            thickness: 3.0,
            length: 6.0,
            material: "碳钢".to_string(),
            quantity: 100,
            location: None,
            supplier: None,
            entry_date: String::new(),
            last_update: None,
            status: "在库".to_string(),
        };
        assert!(db.add_pipe(&pipe).is_err());
        cleanup_test_db(&path);
    }

    #[test]
    fn test_get_pipe_by_id() {
        let (db, path) = setup_test_db();
        let pipe = create_test_pipe("TEST-002", 200);
        db.add_pipe(&pipe).unwrap();
        let found = db.get_pipe_by_id("TEST-002").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().quantity, 200);
        cleanup_test_db(&path);
    }

    #[test]
    fn test_update_pipe_quantity() {
        let (db, path) = setup_test_db();
        let pipe = create_test_pipe("TEST-003", 100);
        db.add_pipe(&pipe).unwrap();
        db.update_pipe_quantity("TEST-003", -30).unwrap();
        let updated = db.get_pipe_by_id("TEST-003").unwrap().unwrap();
        assert_eq!(updated.quantity, 70);
        cleanup_test_db(&path);
    }

    #[test]
    fn test_update_pipe_quantity_insufficient() {
        let (db, path) = setup_test_db();
        let pipe = create_test_pipe("TEST-004", 10);
        db.add_pipe(&pipe).unwrap();
        let result = db.update_pipe_quantity("TEST-004", -20);
        assert!(result.is_err());
        cleanup_test_db(&path);
    }

    #[test]
    fn test_delete_pipe() {
        let (db, path) = setup_test_db();
        let pipe = create_test_pipe("TEST-005", 50);
        db.add_pipe(&pipe).unwrap();
        let record = InventoryRecord {
            id: None,
            pipe_id: "TEST-005".to_string(),
            operation_type: "入库".to_string(),
            quantity: 50,
            operation_date: String::new(),
            operator: "test".to_string(),
            remarks: None,
        };
        db.add_inventory_record(&record).unwrap();
        db.delete_pipe("TEST-005").unwrap();
        let pipes = db.get_pipes().unwrap();
        assert_eq!(pipes.len(), 0);
        cleanup_test_db(&path);
    }

    #[test]
    fn test_update_pipe() {
        let (db, path) = setup_test_db();
        let pipe = create_test_pipe("TEST-006", 100);
        db.add_pipe(&pipe).unwrap();
        let mut updated_pipe = db.get_pipe_by_id("TEST-006").unwrap().unwrap();
        updated_pipe.diameter = 60.0;
        updated_pipe.material = "不锈钢".to_string();
        db.update_pipe(&updated_pipe).unwrap();
        let fetched = db.get_pipe_by_id("TEST-006").unwrap().unwrap();
        assert_eq!(fetched.diameter, 60.0);
        cleanup_test_db(&path);
    }

    #[test]
    fn test_add_inventory_record() {
        let (db, path) = setup_test_db();
        let record = InventoryRecord {
            id: None,
            pipe_id: "TEST-007".to_string(),
            operation_type: "入库".to_string(),
            quantity: 100,
            operation_date: String::new(),
            operator: "张三".to_string(),
            remarks: Some("测试记录".to_string()),
        };
        assert!(db.add_inventory_record(&record).is_ok());
        let records = db.get_inventory_records(None, None, None, None).unwrap();
        assert_eq!(records.len(), 1);
        cleanup_test_db(&path);
    }

    #[test]
    fn test_get_statistics() {
        let (db, path) = setup_test_db();
        let pipe1 = create_test_pipe("STAT-001", 100);
        let pipe2 = SteelPipe {
            id: None,
            pipe_id: "STAT-002".to_string(),
            diameter: 80.0,
            thickness: 5.0,
            length: 12.0,
            material: "不锈钢".to_string(),
            quantity: 200,
            location: None,
            supplier: None,
            entry_date: String::new(),
            last_update: None,
            status: "在库".to_string(),
        };
        db.add_pipe(&pipe1).unwrap();
        db.add_pipe(&pipe2).unwrap();
        db.update_pipe_quantity("STAT-001", -30).unwrap();
        let stats = db.get_statistics().unwrap();
        assert_eq!(stats.total_types, 2);
        cleanup_test_db(&path);
    }

    #[test]
    fn test_get_low_stock_pipes() {
        let (db, path) = setup_test_db();
        let pipe1 = create_test_pipe("LOW-001", 5);
        let pipe2 = create_test_pipe("LOW-002", 50);
        db.add_pipe(&pipe1).unwrap();
        db.add_pipe(&pipe2).unwrap();
        let low_stock = db.get_low_stock_pipes(10).unwrap();
        assert_eq!(low_stock.len(), 1);
        cleanup_test_db(&path);
    }

    #[test]
    fn test_get_pipes_paginated() {
        let (db, path) = setup_test_db();
        for i in 0..15 {
            let pipe = create_test_pipe(&format!("PAGE-{}", i), 10);
            db.add_pipe(&pipe).unwrap();
        }
        let page1 = db.get_pipes_paginated(0, 10).unwrap();
        assert_eq!(page1.len(), 10);
        let count = db.get_pipes_count().unwrap();
        assert_eq!(count, 15);
        cleanup_test_db(&path);
    }

    #[test]
    fn test_import_pipes_from_csv() {
        let (db, path) = setup_test_db();
        let csv = "钢管编号,直径,壁厚,长度,材质,数量\nCSV-001,50.0,3.0,6.0,碳钢,100\nCSV-002,80.0,5.0,12.0,不锈钢,200";
        let (success, fail) = db.import_pipes_from_csv(csv, "test").unwrap();
        assert_eq!(success, 2);
        assert_eq!(fail, 0);
        cleanup_test_db(&path);
    }

    #[test]
    fn test_import_pipes_from_csv_invalid() {
        let (db, path) = setup_test_db();
        let csv = "钢管编号,直径\nBAD-001,invalid";
        let result = db.import_pipes_from_csv(csv, "test");
        assert!(result.is_err());
        cleanup_test_db(&path);
    }

    #[test]
    fn test_export_inventory_to_csv() {
        let (db, path) = setup_test_db();
        let pipe = create_test_pipe("EXP-001", 100);
        db.add_pipe(&pipe).unwrap();
        let csv = db.export_inventory_to_csv().unwrap();
        assert!(csv.contains("EXP-001"));
        cleanup_test_db(&path);
    }

    #[test]
    fn test_export_records_to_csv() {
        let (db, path) = setup_test_db();
        let record = InventoryRecord {
            id: None,
            pipe_id: "REC-001".to_string(),
            operation_type: "入库".to_string(),
            quantity: 100,
            operation_date: String::new(),
            operator: "test".to_string(),
            remarks: None,
        };
        db.add_inventory_record(&record).unwrap();
        let csv = db.export_records_to_csv(None, None, None, None).unwrap();
        assert!(csv.contains("REC-001"));
        cleanup_test_db(&path);
    }

    #[test]
    fn test_export_inventory_to_excel() {
        let (db, path) = setup_test_db();
        let pipe = create_test_pipe("XLS-001", 100);
        db.add_pipe(&pipe).unwrap();
        let xlsx_path = format!("test_{}.xlsx", next_test_id());
        assert!(db.export_inventory_to_excel(&xlsx_path).is_ok());
        assert!(fs::metadata(&xlsx_path).is_ok());
        let _ = fs::remove_file(&xlsx_path);
        cleanup_test_db(&path);
    }

    #[test]
    fn test_export_records_to_excel() {
        let (db, path) = setup_test_db();
        let record = InventoryRecord {
            id: None,
            pipe_id: "REC-001".to_string(),
            operation_type: "入库".to_string(),
            quantity: 100,
            operation_date: String::new(),
            operator: "test".to_string(),
            remarks: None,
        };
        db.add_inventory_record(&record).unwrap();
        let xlsx_path = format!("test_{}.xlsx", next_test_id());
        assert!(db
            .export_records_to_excel(&xlsx_path, None, None, None, None)
            .is_ok());
        assert!(fs::metadata(&xlsx_path).is_ok());
        let _ = fs::remove_file(&xlsx_path);
        cleanup_test_db(&path);
    }

    #[test]
    fn test_log_operation() {
        let (db, path) = setup_test_db();
        db.log_operation(
            "add_pipe",
            "pipe",
            "LOG-001",
            "",
            "{\"qty\": 100}",
            "test",
            "测试",
        )
        .unwrap();
        let logs = db.get_operation_logs(10).unwrap();
        assert_eq!(logs.len(), 1);
        cleanup_test_db(&path);
    }

    #[test]
    fn test_undo_last_operation() {
        let (db, path) = setup_test_db();
        db.log_operation(
            "add_pipe",
            "pipe",
            "UNDO-001",
            "",
            "{\"qty\": 100}",
            "test",
            "测试",
        )
        .unwrap();
        let result = db.undo_last_operation().unwrap();
        assert!(result.contains("add_pipe"));
        let logs = db.get_operation_logs(10).unwrap();
        assert_eq!(logs.len(), 0);
        cleanup_test_db(&path);
    }

    #[test]
    fn test_undo_no_operations() {
        let (db, path) = setup_test_db();
        let result = db.undo_last_operation();
        assert!(result.is_err());
        cleanup_test_db(&path);
    }

    #[test]
    fn test_get_inventory_records_filtered() {
        let (db, path) = setup_test_db();
        let r1 = InventoryRecord {
            id: None,
            pipe_id: "FILT-001".to_string(),
            operation_type: "入库".to_string(),
            quantity: 100,
            operation_date: String::new(),
            operator: "test".to_string(),
            remarks: None,
        };
        let r2 = InventoryRecord {
            id: None,
            pipe_id: "FILT-002".to_string(),
            operation_type: "出库".to_string(),
            quantity: 50,
            operation_date: String::new(),
            operator: "test".to_string(),
            remarks: None,
        };
        db.add_inventory_record(&r1).unwrap();
        db.add_inventory_record(&r2).unwrap();
        let in_records = db
            .get_inventory_records(None, Some("入库"), None, None)
            .unwrap();
        assert_eq!(in_records.len(), 1);
        cleanup_test_db(&path);
    }

    #[test]
    fn test_pipe_validation() {
        let (db, path) = setup_test_db();
        let pipe = SteelPipe {
            id: None,
            pipe_id: "  ".to_string(),
            diameter: 50.0,
            thickness: 3.0,
            length: 6.0,
            material: "碳钢".to_string(),
            quantity: 100,
            location: None,
            supplier: None,
            entry_date: String::new(),
            last_update: None,
            status: "在库".to_string(),
        };
        assert!(db.add_pipe(&pipe).is_err());
        cleanup_test_db(&path);
    }
}
