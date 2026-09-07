-- 014_contract_no.sql — 采购/销售订单增加合同号 (contract_no)
-- 可选字段：订单可无合同号（NULL）。不改已执行迁移（规则 #234）。
ALTER TABLE purchase_orders ADD COLUMN contract_no TEXT;
ALTER TABLE sales_orders ADD COLUMN contract_no TEXT;