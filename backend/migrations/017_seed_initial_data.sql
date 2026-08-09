-- 017_seed_initial_data.sql
-- Seed initial data for suppliers, customers, locations, and generic items.
-- Uses ON CONFLICT DO NOTHING to be idempotent — safe to re-run.
-- SQLite port: NOW() -> datetime('now'), TRUE/FALSE -> 1/0.
-- The former pipe seeds (seamless_pipes / screen_pipes rows) are removed.

-- Sample Suppliers
INSERT INTO suppliers (supplier_code, name, contact_person, phone, email, address, is_active, notes, created_at, updated_at)
VALUES
  ('SUP001', '宝鸡石油钢管有限责任公司', '张经理', '029-33888888', 'baoji@example.com', '陕西省宝鸡市', 1, '主要无缝管供应商', datetime('now'), datetime('now')),
  ('SUP002', '天津钢管制造有限公司', '李经理', '022-24888888', 'tianjin@example.com', '天津市东丽区', 1, '天津钢管集团', datetime('now'), datetime('now')),
  ('SUP003', '衡阳华菱钢管有限公司', '王经理', '0734-8888888', 'hengyang@example.com', '湖南省衡阳市', 1, '衡阳钢管', datetime('now'), datetime('now')),
  ('SUP004', '江苏常宝钢管股份有限公司', '赵经理', '0519-88888888', 'changbao@example.com', '江苏省常州市', 1, '常宝股份', datetime('now'), datetime('now'))
ON CONFLICT DO NOTHING;

-- Sample Customers
INSERT INTO customers (customer_code, name, contact_person, phone, email, address, is_active, notes, created_at, updated_at)
VALUES
  ('CUS001', '中国石油天然气集团有限公司', '刘经理', '010-59988888', 'cnpc@example.com', '北京市东城区', 1, '中石油', datetime('now'), datetime('now')),
  ('CUS002', '中国石油化工集团有限公司', '陈经理', '010-59968888', 'sinopec@example.com', '北京市朝阳区', 1, '中石化', datetime('now'), datetime('now')),
  ('CUS003', '中国海洋石油集团有限公司', '周经理', '010-84528888', 'cnooc@example.com', '北京市东城区', 1, '中海油', datetime('now'), datetime('now')),
  ('CUS004', '延长石油集团有限责任公司', '吴经理', '029-86698888', 'yanchang@example.com', '陕西省西安市', 1, '延长石油', datetime('now'), datetime('now'))
ON CONFLICT DO NOTHING;

-- Sample Warehouse Locations
INSERT INTO locations (zone_code, shelf_code, level_code, full_code, description, capacity, used_count, is_active, created_at, updated_at)
VALUES
  ('A', '01', '1', 'A-01-1', 'A区-1号架-1层', 50, 0, 1, datetime('now'), datetime('now')),
  ('A', '01', '2', 'A-01-2', 'A区-1号架-2层', 50, 0, 1, datetime('now'), datetime('now')),
  ('A', '01', '3', 'A-01-3', 'A区-1号架-3层', 50, 0, 1, datetime('now'), datetime('now')),
  ('A', '02', '1', 'A-02-1', 'A区-2号架-1层', 50, 0, 1, datetime('now'), datetime('now')),
  ('A', '02', '2', 'A-02-2', 'A区-2号架-2层', 50, 0, 1, datetime('now'), datetime('now')),
  ('A', '02', '3', 'A-02-3', 'A区-2号架-3层', 50, 0, 1, datetime('now'), datetime('now')),
  ('B', '01', '1', 'B-01-1', 'B区-1号架-1层', 50, 0, 1, datetime('now'), datetime('now')),
  ('B', '01', '2', 'B-01-2', 'B区-1号架-2层', 50, 0, 1, datetime('now'), datetime('now')),
  ('B', '01', '3', 'B-01-3', 'B区-1号架-3层', 50, 0, 1, datetime('now'), datetime('now')),
  ('B', '02', '1', 'B-02-1', 'B区-2号架-1层', 50, 0, 1, datetime('now'), datetime('now')),
  ('B', '02', '2', 'B-02-2', 'B区-2号架-2层', 50, 0, 1, datetime('now'), datetime('now')),
  ('B', '02', '3', 'B-02-3', 'B区-2号架-3层', 50, 0, 1, datetime('now'), datetime('now')),
  ('C', '01', '1', 'C-01-1', 'C区-1号架-1层', 50, 0, 1, datetime('now'), datetime('now')),
  ('C', '01', '2', 'C-01-2', 'C区-1号架-2层', 50, 0, 1, datetime('now'), datetime('now')),
  ('C', '01', '3', 'C-01-3', 'C区-1号架-3层', 50, 0, 1, datetime('now'), datetime('now')),
  ('C', '02', '1', 'C-02-1', 'C区-2号架-1层', 50, 0, 1, datetime('now'), datetime('now')),
  ('C', '02', '2', 'C-02-2', 'C区-2号架-2层', 50, 0, 1, datetime('now'), datetime('now')),
  ('C', '02', '3', 'C-02-3', 'C区-2号架-3层', 50, 0, 1, datetime('now'), datetime('now'))
ON CONFLICT DO NOTHING;

-- Sample Generic Items (in stock / active)
INSERT INTO items (sku, name, category, unit, spec, price, status, created_at, updated_at)
VALUES
  ('ITM0001', '碳钢板 Q235B', '原材料', '张', '8mm×1500×6000', 4280.00, 'active', datetime('now'), datetime('now')),
  ('ITM0002', '不锈钢板 304', '原材料', '张', '2mm×1000×2000', 15800.00, 'active', datetime('now'), datetime('now')),
  ('ITM0003', '铝合金型材 6063-T5', '原材料', '根', '40×40×3mm', 96.50, 'active', datetime('now'), datetime('now')),
  ('ITM0004', '铜棒 H62', '原材料', '千克', 'Φ30', 68.00, 'active', datetime('now'), datetime('now')),
  ('ITM0005', '深沟球轴承 6205', '标准件', '个', '25×52×15', 18.80, 'active', datetime('now'), datetime('now'))
ON CONFLICT DO NOTHING;
