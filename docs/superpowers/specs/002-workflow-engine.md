# 002 — 工作流引擎 (Phase 1)

> **版本**: v1.0
> **日期**: 2026-08-02
> **依赖**: 001-auth-identity (用户/角色)
> **依赖**: 普遍 — 所有模块都通过 workflow 引擎

---

## 1. 目标

为 ERP 系统提供统一的审批工作流引擎：支持多条件路由、委托、催办和动态参与者。

## 2. 功能

| 功能 | 描述 |
| ------ | ------ |
| **Workflow 定义** | 通过 JSON 流程图定义（BPMN-like） |
| **Workflow 实例** | 当业务流程创建时将定义实例化 |
| **审批线路** | 线性、并行、分支、回流 |
| **动态分配** | 审批人可以按角色、部门、具体人分配 |
| **委托/催办** | 审批人不在时可以委托代理；逾期未处理自动提醒 |
| **回调** | 审批完成后调用该流程，一个业务 service (e.g. approve 采购订单) |
| **可视化设计器** | 前端画图工具（react-flow） |

## 3. 数据模型

> 数据库为 SQLite3（`sqlite://data/erp.db?mode=rwc`），JSON 定义以 TEXT 存储，时间用 TEXT（`datetime('now')`）。

```sql
CREATE TABLE workflow_definitions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    model_type TEXT,                -- 'purchase', 'sales', 'leave', 'finance', 'process', ...
    status TEXT DEFAULT 'draft',    -- draft / active / archived
    definition TEXT NOT NULL,       -- 流程定义 (JSON 结构)
    version INTEGER DEFAULT 1,
    created_by INTEGER,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now')),
    deleted_at TEXT
);

CREATE TABLE workflow_instances (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    definition_id INTEGER REFERENCES workflow_definitions(id),
    entity_type TEXT,               -- 'purchase_order', 'sales_order', 'leave_request'
    entity_id INTEGER,              -- the business object ID
    title TEXT,
    status TEXT DEFAULT 'running',  -- running / completed / rejected / cancelled
    started_by INTEGER REFERENCES users(id),
    started_at TEXT DEFAULT (datetime('now')),
    completed_at TEXT,
    current_step_index INTEGER
);

CREATE TABLE approval_nodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    instance_id INTEGER REFERENCES workflow_instances(id),
    step_index INTEGER,             -- 0,1,2, ... within instance
    step_id TEXT,                   -- from definition JSON (e.g. 'step_approve_manager')
    assignee_user_id INTEGER REFERENCES users(id),
    assignee_role_id INTEGER REFERENCES roles(id),  -- if role-based
    assignee_dep_id INTEGER,        -- if department-based
    status TEXT DEFAULT 'pending',  -- pending / approved / rejected / skipped
    approved_by INTEGER REFERENCES users(id),
    approved_at TEXT,
    approval_reason TEXT,
    due_date TEXT
);

CREATE TABLE workflow_delegations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    original_user_id INTEGER NOT NULL REFERENCES users(id),
    delegated_user_id INTEGER NOT NULL,
    starts_at TEXT DEFAULT (datetime('now')),
    ends_at TEXT NOT NULL,
    is_active INTEGER DEFAULT 1
);

CREATE TABLE workflow_escalations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    node_id INTEGER REFERENCES approval_nodes(id),
    escalation_level INTEGER DEFAULT 1,   -- 1st, 2nd notice to next level
    notified_at TEXT
);
```

## 4. 工作流示例: 采购订单审批

```
start → approve_manager → if amount > 50k → approve_director
                          → if category = 'special' → quality_team
                          → else → approve_manager → end
```

## 5. API

| Method | Path | Description |
| -------- | ------ | ------------- |
| GET | `/api/workflows/definitions` | List workflow定义 templates |
| POST | `/api/workflows/definitions` | Create new流程定义 |
| PUT | `/api/workflows/definitions/:id` | Update |
| DELETE | `/api/workflows/definitions/:id` | Archived (软删除) |
| GET | `/api/workflows/definitions/:id` | Detail |
| POST | `/api/workflows/instances` | Start a workflow (via business action) |
| GET | `/api/workflows/my-tasks` | Pending tasks for current user |
| GET | `/api/workflows/tasks/:nodeId` | Show task detail |
| POST | `/api/workflows/tasks/:nodeId/approve` | Approve |
| POST | `/api/workflows/tasks/:nodeId/reject` | Reject |
| POST | `/api/workflows/tasks/delegate` | Delegate |

## 6. 后端实现

```rust
// backend/src/workflow/services.rs
pub struct WorkflowService;

impl WorkflowService {
    pub async fn start(
        pool: &SqlitePool, def_id: i64, entity_id: i64, started_by: i64
    ) -> Result<WorkflowInstance, AppError> {
        // 1. 加载定义
        // 2. 加载 current entity
        // 3. 计算每一步 (conditions) → define nodes
        // 4. 实例化 approval_nodes
        // 5. notify initial assignee
    }

    pub async fn approve(
        pool, node_id, user_id, reason
    ) -> Result<(), AppError> {
        // 1. 查找节点
        // 2. 检查 user 是否有权
        // 3. 标记节点完成
        // 4. 移动下一步
        // 5. 如果是最终步骤: 触发回调
    }
}
```

## 7. 前端

- `features/workflow/pages/WorkflowDesignerPage.tsx` → react-flow 拖放式流图设计器
- `features/workflow/pages/WorkflowApprovalListPage.tsx` → 用户待审批列表
- `features/workflow/pages/WorkflowApprovalDetailPage.tsx` → 审批详情 + 原因 + 审批链

## 8. 与其他模块的联动

```rust
// 模块A (如 purchase_order) 创建时:
event = "workflow.instances.created"
   payload: { instance_id, entity_id: 42 }
// 模块A 设置:
task = "order.approve"  ← callback → 标记 order.status = 'approved'
```
