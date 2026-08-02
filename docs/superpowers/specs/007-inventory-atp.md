# 007 — 库存管理 & ATP (Phase 2)

> **版本**: v1.0
> **日期**: 2026-08-02
> **依赖**: 001-auth-identity
> **状态**: Draft

---

## 1. 目标

深化库存管理：增强 ATP (Available-to-Promise)、多仓库、预售及到货预期、库存日志、盘点流程深化。

## 2. 功能

| 功能 | 说明 |
|------|------|
| Stock aggregate views | 实时汇总 stock  quantités (按各管道 + 仓库，加预留) |
| Full ATP 系统 | 基于约束的 ATP 模型 (总计库存 + purchase_inflight 预计 - sale_ordered 未交付 - manufacture_planned 未制造) |
| Stock reservation (预售) | SO 创建预留库存 |
| Expected arrival | 系统预测 PO到货的自动生成 inbounds |
| Internal transfer | 仓库间调拨 |
| Cycle counting | 系统驱动的盘点计划 |

## 3. 数据模型 (extend inventory schema)

```sql
CREATE TABLE inventory.atp_slots (
    id BIGSERIAL PRIMARY KEY,
    pipe_number TEXT NOT NULL,
    warehouse_id INTEGER,
    stock_qty_onhand NUMERIC(12,4),         -- 现货数量
    reserved_qty NUMERIC(12,4) DEFAULT 0,
    expected_qty NUMERIC(12,4) DEFAULT 0,   -- 预期入库(来源 PO、生产工单)
    atp_qty NUMERIC(12,4),                  -- = onhand+expected - reserved
    last_updated TIMESTAMPTZ
);

CREATE TABLE inventory.internal_transfers (
    id BIGSERIAL PRIMARY KEY,
    transfer_no VARCHAR(100) UNIQUE,
    source_warehouse_id INTEGER,
    target_warehouse_id INTEGER,
    status VARCHAR(20) DEFAULT 'pending',
    shipped_at, received_at TIMESTAMPTZ
);

CREATE TABLE inventory.count_templates (
    id BIGSERIAL PRIMARY KEY,
    template_name VARCHAR(200),
    frequency VARCHAR(20),                  -- daily/weekly/quarterly/annual?
    created_at TIMESTAMPTZ
);

-- 原 inbound_records / outbound_records / inventory_logs / locations 表已在 inventory schema
```

## 4. API

| Method | Path | Description |
|--------|------|-------------|
| ATP 查询 | | |
| GET | `/api/inventory/atp/pipe/:pipeNumber` | 查询特定管号的ATP |
| GET | `/api/inventory/atp/overage` | 全局可承诺库存概览 |
| POST | `/api/inventory/transfers` | 创建伪库移送记录 |
| POST | `/api/inventory/reservations` | 为 SO 预留库存 |
| GET | `/api/inventory/logs` | 普查日志 |

## 5. 前端

- `features/inventory/pages/AtpQueryPage.tsx` → 实时 ATP 计算展示
- `features/inventory/pages/InternalTransferPage.tsx`
- `features/inventory/pages/StockCountTemplatePage.tsx`