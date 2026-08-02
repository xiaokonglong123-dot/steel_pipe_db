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
|--------|------|
| **物料清单 (BOM)** | 树状 BOM，显示每环节所需材料和设备 |
| **生产工单** | 基于订单生成工单 -> 需要 BOM + routing |
| **路径规划 (Routing)** | 各工序 + 设备要求 + 标准工时 |
| **现场执行** | 打卡工单选进度(开始/完成/签收/移动) |
| **质检深化** | 在制品检验、缺陷爬虫 (PA 循环)、NCR |
| **设备管理** | 设备注册、维护计划、稼动率 |

## 3. 数据模型

```sql
CREATE TABLE manufacturing.boms (
    id BIGSERIAL PRIMARY KEY,
    product_id BIGINT REFERENCES inventory.seamless_pipes(id), -- 或 screen pipes
    name VARCHAR(200),
    production_version INTEGER,
    status VARCHAR(20) DEFAULT 'active',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE manufacturing.bom_items (
    id BIGSERIAL PRIMARY KEY,
    bom_id BIGINT REFERENCES manufacturing.boms(id),
    parent_item_id BIGINT REFERENCES manufacturing.bom_items(id),
    item_type VARCHAR(20),     -- raw_material, semi_product, sub_assembly, pipe
    item_identifier BIGINT,    -- seam_pipe_id or 其他
    quantity NUMERIC(12,4),
    unit VARCHAR(10),           -- pcs, kg, m
);

CREATE TABLE manufacturing.work_orders (
    id BIGSERIAL PRIMARY KEY,
    order_no VARCHAR(100) UNIQUE,
    bom_id BIGINT REFERENCES manufacturing.boms(id),
    sales_order_id BIGINT,              -- 为客户生产
    planned_qty NUMERIC(12,4),
    actual_qty NUMERIC(12,4),
    status VARCHAR(20) DEFAULT 'draft',   -- draft/scheduled/in_progress/quality_check/done/cancelled
    scheduled_start TIMESTAMPTZ,
    scheduled_end TIMESTAMPTZ,
    started_at, completed_at TIMESTAMPTZ
);

CREATE TABLE manufacturing.work_order_steps (
    id BIGSERIAL PRIMARY KEY,
    work_order_id BIGINT REFERENCES manufacturing.work_orders(id),
    routing_ops_id BIGINT REFERENCES manufacturing.routing_ops(id),
    sequence INT,
    status VARCHAR(20) DEFAULT 'pending',
    assigned_equipment_id BIGINT,
    hours_taken NUMERIC(10,2),
    inspected_by BIGINT REFERENCES auth.users(id)
);

CREATE TABLE manufacturing.routing_ops (
    id BIGSERIAL,
    operation_name VARCHAR(200),   -- 切管, 攻螺纹, 热处理, 防腐...
    workstation_type VARCHAR(100), -- pipe_cutting, threading, heat_treatment,,, 等等
    standard_hours_each NUMERIC(10,2),
    requires_qc BOOLEAN DEFAULT true
);

CREATE TABLE manufacturing.quality_inspections (
    id BIGSERIAL PRIMARY KEY,
    work_order_step_id BIGINT REFERENCES manufacturing.work_order_steps(id),
    test_type VARCHAR(100),        -- dimension, hardness, hydrostatic, non_destructive, visual
    value NUMERIC(18,4),
    tolerance_range VARCHAR(100), -- 上限下限
    result VARCHAR(20) DEFAULT 'pass', -- pass/fail
    ncr_id BIGINT REFERENCES manufacturing.ncr_outputs(id),
    inspector_id BIGINT REFERENCES auth.users(id),
    created_at TIMESTAMPTZ
);

CREATE TABLE manufacturing.ncr_outputs (
    id BIGSERIAL PRIMARY KEY,
    ncr_no VARCHAR(100) UNIQUE,
    defected_product_id,  -- 什么管件
    defect_description TEXT,
    corrective_action TEXT,
    root_cause TEXT,
    status VARCHAR(20) DEFAULT 'open'
);

CREATE TABLE manufacturing.equipment_register (
    id BIGSERIAL PRIMARY KEY,
    eq_name VARCHAR(300),
    maintenance_interval_days INT
);
```

## 4. API

| Method | Path | Description |
|--------|------|-------------|
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
| POST | `/api/manufacturing/ncr` | 创建不合规报告 |

## 5. 前端

- `features/manufacturing/pages/BomListPage.tsx`
- `features/manufacturing/pages/BomDetailPage.tsx` → 树状 BOM viewer
- `features/manufacturing/pages/WorkOrderListPage.tsx`
- `features/manufacturing/pages/WorkOrderDetailPage.tsx` → 工单 + 进度
- `features/manufacturing/pages/QualityInspectionsListPage.tsx`