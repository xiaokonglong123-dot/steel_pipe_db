-- 017_seed_initial_data.sql
-- Seed initial data for suppliers, customers, locations, and pipe templates.
-- Uses ON CONFLICT DO NOTHING to be idempotent — safe to re-run.

-- Sample Suppliers
INSERT INTO suppliers (supplier_code, name, contact_person, phone, email, address, is_active, notes, created_at, updated_at)
VALUES
  ('SUP001', '宝鸡石油钢管有限责任公司', '张经理', '029-33888888', 'baoji@example.com', '陕西省宝鸡市', TRUE, '主要无缝管供应商', NOW(), NOW()),
  ('SUP002', '天津钢管制造有限公司', '李经理', '022-24888888', 'tianjin@example.com', '天津市东丽区', TRUE, '天津钢管集团', NOW(), NOW()),
  ('SUP003', '衡阳华菱钢管有限公司', '王经理', '0734-8888888', 'hengyang@example.com', '湖南省衡阳市', TRUE, '衡阳钢管', NOW(), NOW()),
  ('SUP004', '江苏常宝钢管股份有限公司', '赵经理', '0519-88888888', 'changbao@example.com', '江苏省常州市', TRUE, '常宝股份', NOW(), NOW())
ON CONFLICT DO NOTHING;

-- Sample Customers
INSERT INTO customers (customer_code, name, contact_person, phone, email, address, is_active, notes, created_at, updated_at)
VALUES
  ('CUS001', '中国石油天然气集团有限公司', '刘经理', '010-59988888', 'cnpc@example.com', '北京市东城区', TRUE, '中石油', NOW(), NOW()),
  ('CUS002', '中国石油化工集团有限公司', '陈经理', '010-59968888', 'sinopec@example.com', '北京市朝阳区', TRUE, '中石化', NOW(), NOW()),
  ('CUS003', '中国海洋石油集团有限公司', '周经理', '010-84528888', 'cnooc@example.com', '北京市东城区', TRUE, '中海油', NOW(), NOW()),
  ('CUS004', '延长石油集团有限责任公司', '吴经理', '029-86698888', 'yanchang@example.com', '陕西省西安市', TRUE, '延长石油', NOW(), NOW())
ON CONFLICT DO NOTHING;

-- Sample Warehouse Locations
INSERT INTO locations (zone_code, shelf_code, level_code, full_code, description, capacity, used_count, is_active, created_at, updated_at)
VALUES
  ('A', '01', '1', 'A-01-1', 'A区-1号架-1层', 50, 0, TRUE, NOW(), NOW()),
  ('A', '01', '2', 'A-01-2', 'A区-1号架-2层', 50, 0, TRUE, NOW(), NOW()),
  ('A', '01', '3', 'A-01-3', 'A区-1号架-3层', 50, 0, TRUE, NOW(), NOW()),
  ('A', '02', '1', 'A-02-1', 'A区-2号架-1层', 50, 0, TRUE, NOW(), NOW()),
  ('A', '02', '2', 'A-02-2', 'A区-2号架-2层', 50, 0, TRUE, NOW(), NOW()),
  ('A', '02', '3', 'A-02-3', 'A区-2号架-3层', 50, 0, TRUE, NOW(), NOW()),
  ('B', '01', '1', 'B-01-1', 'B区-1号架-1层', 50, 0, TRUE, NOW(), NOW()),
  ('B', '01', '2', 'B-01-2', 'B区-1号架-2层', 50, 0, TRUE, NOW(), NOW()),
  ('B', '01', '3', 'B-01-3', 'B区-1号架-3层', 50, 0, TRUE, NOW(), NOW()),
  ('B', '02', '1', 'B-02-1', 'B区-2号架-1层', 50, 0, TRUE, NOW(), NOW()),
  ('B', '02', '2', 'B-02-2', 'B区-2号架-2层', 50, 0, TRUE, NOW(), NOW()),
  ('B', '02', '3', 'B-02-3', 'B区-2号架-3层', 50, 0, TRUE, NOW(), NOW()),
  ('C', '01', '1', 'C-01-1', 'C区-1号架-1层', 50, 0, TRUE, NOW(), NOW()),
  ('C', '01', '2', 'C-01-2', 'C区-1号架-2层', 50, 0, TRUE, NOW(), NOW()),
  ('C', '01', '3', 'C-01-3', 'C区-1号架-3层', 50, 0, TRUE, NOW(), NOW()),
  ('C', '02', '1', 'C-02-1', 'C区-2号架-1层', 50, 0, TRUE, NOW(), NOW()),
  ('C', '02', '2', 'C-02-2', 'C区-2号架-2层', 50, 0, TRUE, NOW(), NOW()),
  ('C', '02', '3', 'C-02-3', 'C区-2号架-3层', 50, 0, TRUE, NOW(), NOW())
ON CONFLICT DO NOTHING;

-- Sample Seamless Pipes (in stock)
INSERT INTO seamless_pipes (pipe_number, batch_number, pipe_type, grade, od, wt, length, weight_per_unit, end_type, coupling_type, heat_number, serial_number, manufacturer, production_date, cert_number, location_id, status, notes, created_at, updated_at)
VALUES
  ('SP-J55-139.7x7.72-BN001', 'BN-2024-001', 'casing', 'J55', 139.7, 7.72, 9.6, 23.07, 'BTC', 'N80Q', 'HN-2024-001', 'SN-001', '宝鸡石油钢管', '2024-01-15', 'CERT-001', 1, 'in_stock', '标准J55套管', NOW(), NOW()),
  ('SP-N80-139.7x9.17-BN001', 'BN-2024-001', 'casing', 'N80', 139.7, 9.17, 9.6, 27.09, 'BTC', 'N80Q', 'HN-2024-002', 'SN-002', '宝鸡石油钢管', '2024-01-15', 'CERT-002', 1, 'in_stock', '标准N80套管', NOW(), NOW()),
  ('SP-J55-177.8x9.19-BN001', 'BN-2024-002', 'casing', 'J55', 177.8, 9.19, 9.6, 37.06, 'BTC', 'N80Q', 'HN-2024-003', 'SN-003', '天津钢管', '2024-02-01', 'CERT-003', 2, 'in_stock', '标准J55套管', NOW(), NOW()),
  ('SP-N80-177.8x10.36-BN001', 'BN-2024-002', 'casing', 'N80', 177.8, 10.36, 9.6, 41.49, 'BTC', 'N80Q', 'HN-2024-004', 'SN-004', '天津钢管', '2024-02-01', 'CERT-004', 2, 'in_stock', '标准N80套管', NOW(), NOW()),
  ('SP-P110-139.7x9.17-BN001', 'BN-2024-003', 'casing', 'P110', 139.7, 9.17, 9.6, 27.09, 'BTC', 'P110', 'HN-2024-005', 'SN-005', '衡阳华菱', '2024-02-15', 'CERT-005', 3, 'in_stock', '标准P110套管', NOW(), NOW())
ON CONFLICT DO NOTHING;

-- Sample Seamless Pipes (outbound - shipped)
INSERT INTO seamless_pipes (pipe_number, batch_number, pipe_type, grade, od, wt, length, weight_per_unit, end_type, coupling_type, heat_number, serial_number, manufacturer, production_date, cert_number, location_id, status, notes, created_at, updated_at)
VALUES
  ('SP-J55-88.9x6.45-BN001', 'BN-2024-004', 'tubing', 'J55', 88.9, 6.45, 9.6, 11.61, 'EUE', 'EUE', 'HN-2024-006', 'SN-006', '江苏常宝', '2024-03-01', 'CERT-006', NULL, 'outbound', '标准J55油管', NOW(), NOW())
ON CONFLICT DO NOTHING;

-- Sample Seamless Pipes (new - not yet in stock)
INSERT INTO seamless_pipes (pipe_number, batch_number, pipe_type, grade, od, wt, length, weight_per_unit, end_type, coupling_type, heat_number, serial_number, manufacturer, production_date, cert_number, location_id, status, notes, created_at, updated_at)
VALUES
  ('SP-L80-177.8x10.36-BN001', 'BN-2024-005', 'casing', 'L80', 177.8, 10.36, 9.6, 41.49, 'BTC', 'L80', 'HN-2024-007', 'SN-007', '宝鸡石油钢管', '2024-03-15', 'CERT-007', NULL, 'new', '待入库L80套管', NOW(), NOW()),
  ('SP-T95-139.7x9.17-BN001', 'BN-2024-005', 'casing', 'T95', 139.7, 9.17, 9.6, 27.09, 'BTC', 'T95', 'HN-2024-008', 'SN-008', '天津钢管', '2024-03-15', 'CERT-008', NULL, 'new', '待入库T95套管', NOW(), NOW())
ON CONFLICT DO NOTHING;

-- Sample Screen Pipes (in stock)
INSERT INTO screen_pipes (pipe_number, batch_number, screen_type, slot_size, filtration_grade, base_od, base_wt, base_grade, base_end_type, length, weight_per_unit, heat_number, serial_number, manufacturer, production_date, cert_number, location_id, status, notes, created_at, updated_at)
VALUES
  ('SCP-J55-139.7x7.72-BN001', 'BN-2024-006', 'slotted', 0.3, 'standard', 139.7, 7.72, 'J55', 'BTC', 9.6, 23.07, 'HN-2024-009', 'SN-009', '宝鸡石油钢管', '2024-02-20', 'CERT-009', 4, 'in_stock', '标准割缝筛管', NOW(), NOW()),
  ('SCP-N80-177.8x9.19-BN001', 'BN-2024-007', 'slotted', 0.5, 'standard', 177.8, 9.19, 'N80', 'BTC', 9.6, 37.06, 'HN-2024-010', 'SN-010', '天津钢管', '2024-02-20', 'CERT-010', 4, 'in_stock', '标准割缝筛管', NOW(), NOW()),
  ('SCP-J55-139.7x7.72-BN002', 'BN-2024-008', 'wire_wrapped', 0.3, 'standard', 139.7, 7.72, 'J55', 'BTC', 9.6, 23.07, 'HN-2024-011', 'SN-011', '衡阳华菱', '2024-03-10', 'CERT-011', 5, 'in_stock', '绕丝筛管', NOW(), NOW())
ON CONFLICT DO NOTHING;
