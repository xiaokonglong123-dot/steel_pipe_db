-- 009_seed.sql — 种子数据

INSERT INTO roles (id, name, description, is_system) VALUES (1, 'admin', '系统管理员', 1);
INSERT INTO roles (id, name, description, is_system) VALUES (2, 'manager', '业务经理', 1);
INSERT INTO roles (id, name, description, is_system) VALUES (3, 'warehouse', '仓库', 1);
INSERT INTO roles (id, name, description, is_system) VALUES (4, 'purchaser', '采购', 1);
INSERT INTO roles (id, name, description, is_system) VALUES (5, 'sales', '销售', 1);
INSERT INTO roles (id, name, description, is_system) VALUES (6, 'finance', '财务', 1);

INSERT INTO permissions (key, name) VALUES
    ('item.read',   '商品-查看'), ('item.write',  '商品-编辑'),
    ('stock.read',  '库存-查看'), ('stock.write', '库存-操作'),
    ('order.read',  '订单-查看'), ('order.write', '订单-编辑'), ('order.approve', '订单-审批'),
    ('finance.read','财务-查看'), ('finance.write','财务-记账'),
    ('report.read', '报表-查看'),
    ('user.manage', '用户-管理');

INSERT INTO role_permissions (role_id, permission_id) SELECT 1, id FROM permissions;
INSERT INTO role_permissions (role_id, permission_id)
    SELECT 2, id FROM permissions WHERE key IN ('item.read','stock.read','order.read','order.approve','finance.read','report.read');
INSERT INTO role_permissions (role_id, permission_id)
    SELECT 3, id FROM permissions WHERE key IN ('item.read','item.write','stock.read','stock.write','order.read','report.read');
INSERT INTO role_permissions (role_id, permission_id)
    SELECT 4, id FROM permissions WHERE key IN ('item.read','item.write','stock.read','order.read','order.write','report.read');
INSERT INTO role_permissions (role_id, permission_id)
    SELECT 5, id FROM permissions WHERE key IN ('item.read','item.write','stock.read','order.read','order.write','report.read');
INSERT INTO role_permissions (role_id, permission_id)
    SELECT 6, id FROM permissions WHERE key IN ('finance.read','finance.write','report.read');
