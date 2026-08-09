# 008 — 制造管理 (Phase 3)

> **版本**: v1.0
> **日期**: 2026-08-02
> **依赖**: 002-workflow-engine, 007-inventory-atp
> **状态**: Draft

---

## 1. 目标

完整的离散制造管理：BOM、生产工单、路径作业、在制品、设备维护、质检深化。

## 2. 功能范围

| 子模块 | 描述 |
| -------- | ------ |
| **物料清单 (BOM)** | 树状 BOM，显示每环节所需商品和设备 |
| **生产工单** | 基于订单生成工单 -> 需要 BOM + routing |
| **路径规划 (Routing)** | 各工序 + 设备要求 + 标准工时 |
| **现场执行** | 打卡工单选进度(开始/完成/签收/移动) |
| **质检深化** | 在制品检验、缺陷分析 (PDCA 循环)、NCR |
| **设备管理** | 设备注册、维护计划、稼动率 |

## 3. 数据模型

> 数据库为 SQLite3（`sqlite://data/erp.db?mode=rwc`），商品统一引用 `items.id`，质检为制造过程的质量检验记录（Inspection）。

```sql
CREATE TABLE boms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    product_id INTEGER REFERENCES items(id),  -- 商品
    name TEXT,
    production_version INTEGER,
    status TEXT DEFAULT 'active',
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE bom_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bom_id INTEGER REFERENCES boms(id),
    parent_item_id INTEGER REFERENCES bom_items(id),
    item_type TEXT,     -- raw_material, semi_product, sub_assembly, product
    item_id INTEGER REFERENCES items(id),  -- 商品
    quantity REAL,
    unit TEXT            -- pcs, kg, m
);

CREATE TABLE work_orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_no TEXT UNIQUE,
    bom_id INTEGER REFERENCES boms(id),
    sales_order_id INTEGER,              -- 为客户生产
    planned_qty REAL,
    actual_qty REAL,
    status TEXT DEFAULT 'draft',   -- draft/scheduled/in_progress/quality_check/done/cancelled
    scheduled_start TEXT,
    scheduled_end TEXT,
    started_at TEXT,
    completed_at TEXT
);

CREATE TABLE work_order_steps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    work_order_id INTEGER REFERENCES work_orders(id),
    routing_ops_id INTEGER REFERENCES routing_ops(id),
    sequence INTEGER,
    status TEXT DEFAULT 'pending',
    assigned_equipment_id INTEGER,
    hours_taken REAL,
    inspected_by INTEGER REFERENCES users(id)
);

CREATE TABLE routing_ops (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    operation_name TEXT,   -- 下料, 加工, 热处理, 组装...
    workstation_type TEXT, -- cutting, machining, heat_treatment, assembly...
    standard_hours_each REAL,
    requires_qc INTEGER DEFAULT 1
);

CREATE TABLE quality_inspections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    work_order_step_id INTEGER REFERENCES work_order_steps(id),
    test_type TEXT,        -- dimension, hardness, visual, functional...
    value REAL,
    tolerance_range TEXT,  -- 上限下限
    result TEXT DEFAULT 'pass', -- pass/fail
    ncr_id INTEGER REFERENCES ncr_outputs(id),
    inspector_id INTEGER REFERENCES users(id),
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE ncr_outputs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ncr_no TEXT UNIQUE,
    defected_item_id INTEGER REFERENCES items(id),  -- 不合格商品
    defect_description TEXT,
    corrective_action TEXT,
    root_cause TEXT,
    status TEXT DEFAULT 'open'
);

CREATE TABLE equipment_register (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    eq_name TEXT,
    maintenance_interval_days INTEGER
);
```

## 4. API

| Method | Path | Description |
| -------- | ------ | ------------- |
| BOM | | |
| GET | `/api/manufacturing/boms` | BOM list |
| POST | `/api/manufacturing/boms` | Create BOM |
| GET | `/api/manufacturing/boms/:id` | Detail |
| 工单 | | |
| GET | `/api/manufacturing/work-orders` | 工单列表 |
| POST | `/api/manufacturing/work-orders` | Create |
| POST | `/api/manufacturing/work-orders/:id/start` | Start |
| POST | `/api/manufacturing/work-orders/:id/complete-step` | Complete step |
| 质检 | | |
| POST | `/api/manufacturing/inspections` | Log inspection |
| POST | `/api/manufacturing/ncr` | 创建不合格品单 |

## 5. 前端

- `features/manufacturing/pages/BomListPage.tsx`
- `features/manufacturing/pages/BomDetailPage.tsx` → 树状 BOM viewer
- `features/manufacturing/pages/WorkOrderListPage.tsx`
- `features/manufacturing/pages/WorkOrderDetailPage.tsx` → 工单 + 进度
- `features/manufacturing/pages/QualityInspectionsListPage.tsx`
