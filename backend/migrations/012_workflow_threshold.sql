-- 012_workflow_threshold.sql — P2.3 审批流多级/条件支持
-- 给 workflow_transitions 加 amount_threshold（TEXT，存 Decimal::to_string()）
-- 含义：若 amount_threshold 非空，则当 business_amount >= amount_threshold 时此 transition 才生效。
-- 配合现有 required_role 字段实现"金额阈值触发高级审批"。

ALTER TABLE workflow_transitions ADD COLUMN amount_threshold TEXT;
