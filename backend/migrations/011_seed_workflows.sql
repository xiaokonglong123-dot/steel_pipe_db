-- 011_seed_workflows.sql — 审批流定义种子（数据驱动配置，不改任何已执行迁移）

-- purchase_order 工作流：draft → submitted → approved/rejected
INSERT INTO workflows (id, name, applies_to, is_active) VALUES (1, '采购订单审批', 'purchase_order', 1);
INSERT INTO workflow_states (workflow_id, state_key, doc_status, is_initial, is_final) VALUES
    (1, 'draft',     0, 1, 0),
    (1, 'submitted', 1, 0, 0),
    (1, 'approved',  1, 0, 1),
    (1, 'rejected',  1, 0, 1);
INSERT INTO workflow_transitions (workflow_id, from_state_id, to_state_id, action, required_role, is_auto) VALUES
    (1, (SELECT id FROM workflow_states WHERE workflow_id=1 AND state_key='draft'),     (SELECT id FROM workflow_states WHERE workflow_id=1 AND state_key='submitted'), 'submit',  NULL,           0),
    (1, (SELECT id FROM workflow_states WHERE workflow_id=1 AND state_key='submitted'), (SELECT id FROM workflow_states WHERE workflow_id=1 AND state_key='approved'),  'approve', 'order.approve', 0),
    (1, (SELECT id FROM workflow_states WHERE workflow_id=1 AND state_key='submitted'), (SELECT id FROM workflow_states WHERE workflow_id=1 AND state_key='rejected'),  'reject',  'order.approve', 0);

-- sales_order 工作流：draft → submitted → approved/rejected
INSERT INTO workflows (id, name, applies_to, is_active) VALUES (2, '销售订单审批', 'sales_order', 1);
INSERT INTO workflow_states (workflow_id, state_key, doc_status, is_initial, is_final) VALUES
    (2, 'draft',     0, 1, 0),
    (2, 'submitted', 1, 0, 0),
    (2, 'approved',  1, 0, 1),
    (2, 'rejected',  1, 0, 1);
INSERT INTO workflow_transitions (workflow_id, from_state_id, to_state_id, action, required_role, is_auto) VALUES
    (2, (SELECT id FROM workflow_states WHERE workflow_id=2 AND state_key='draft'),     (SELECT id FROM workflow_states WHERE workflow_id=2 AND state_key='submitted'), 'submit',  NULL,           0),
    (2, (SELECT id FROM workflow_states WHERE workflow_id=2 AND state_key='submitted'), (SELECT id FROM workflow_states WHERE workflow_id=2 AND state_key='approved'),  'approve', 'order.approve', 0),
    (2, (SELECT id FROM workflow_states WHERE workflow_id=2 AND state_key='submitted'), (SELECT id FROM workflow_states WHERE workflow_id=2 AND state_key='rejected'),  'reject',  'order.approve', 0);
