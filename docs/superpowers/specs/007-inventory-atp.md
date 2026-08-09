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
| ------ | ------ |
| Stock aggregate views | 实时汇总库存数量（按商品 + 仓库，加预留） |
| Full ATP 系统 | 基于约束的 ATP 模型 (在库库存 + 采购在途预计 - 销售未交付 - 制造未生产) |
| Stock reservation (预售) | 销售订单创建预留库存 |
| Expected arrival | 系统预测采购订单到货的自动生成 inbounds |
| Internal transfer | 仓库间调拨 |
| Cycle counting | 系统驱动的盘点计划 |

## 3. 数据模型

> 数据库为 SQLite3（`sqlite://data/erp.db?mode=rwc`），商品用 `items.id` / SKU 标识。

```sql
CREATE TABLE atp_slots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id INTEGER NOT NULL REFERENCES items(id),  -- 商品
    sku TEXT,                                       -- 商品 SKU
    warehouse_id INTEGER,
    stock_qty_onhand REAL,         -- 现货数量
    reserved_qty REAL DEFAULT 0,
    expected_qty REAL DEFAULT 0,   -- 预期入库(来源 采购订单、生产工单)
    atp_qty REAL,                  -- = onhand + expected - reserved
    last_updated TEXT DEFAULT (datetime('now'))
);

CREATE TABLE internal_transfers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transfer_no TEXT UNIQUE,
    source_warehouse_id INTEGER,
    target_warehouse_id INTEGER,
    status TEXT DEFAULT 'pending',
    shipped_at TEXT,
    received_at TEXT
);

CREATE TABLE count_templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    template_name TEXT,
    frequency TEXT,                  -- daily/weekly/quarterly/annual
    created_at TEXT DEFAULT (datetime('now'))
);

-- 原 inbound_records / outbound_records / inventory_logs / locations 表保留（商品化改造后引用 item_id）
```

## 4. API

| Method | Path | Description |
| -------- | ------ | ------------- |
| ATP 查询 | | |
| GET | `/api/inventory/atp/items/:sku` | 查询特定 SKU 的 ATP |
| GET | `/api/inventory/atp/overview` | 全局可承诺库存概览 |
| POST | `/api/inventory/transfers` | 创建调拨记录 |
| POST | `/api/inventory/reservations` | 为销售订单预留库存 |
| GET | `/api/inventory/logs` | 库存日志 |

## 5. 前端

- `features/inventory/pages/AtpQueryPage.tsx` → 实时 ATP 计算展示
- `features/inventory/pages/InternalTransferPage.tsx`
- `features/inventory/pages/StockCountTemplatePage.tsx`
