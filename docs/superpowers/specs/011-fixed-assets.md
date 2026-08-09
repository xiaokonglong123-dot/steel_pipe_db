# 011 — 固定资产管理 (Phase 4)

> **版本**: v1.0 | **日期**: 2026-08-02 | **依赖**: 004-finance | **状态**: Draft

---

## 1. 目标

管理公司的固定资产：购买登记、折旧摊销、转移、报废、盘存。

## 2. 功能

| 子模块 | 描述 |
| -------- | ------ |
| 资产登记 | 编号、名称、采购、折旧类、年限、生产日期 |
| 折旧计算 | 直线法 / 双倍余额递减、按会计周期生成折旧分录 |
| 资产调拨 | 公司/部门/项目之间的资产移动 |
| 资产处置 | 报废或卖出 — 计提折旧同时关闭 |

## 3. 数据模型

> 数据库为 SQLite3（`sqlite://data/erp.db?mode=rwc`）。

```sql
CREATE TABLE fixed_assets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    asset_no TEXT UNIQUE,
    name TEXT,
    asset_category TEXT,
    acquisition_cost REAL,
    useful_life_years INTEGER,
    residual_value REAL,
    periods INTEGER DEFAULT 0, -- 当前累计期间数
    current_depreciation_method TEXT DEFAULT 'straight_line',
    location_id INTEGER,
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE depreciation_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    asset_id INTEGER REFERENCES fixed_assets(id),
    period TEXT,   -- month
    depreciated_amount REAL,
    journal_entry_rel_id INTEGER,       --> journal_entries.id
    created_at TEXT DEFAULT (datetime('now'))
);
```

## 4. API

| Method | Path | Description |
| -------- | ------ | ------------- |
| GET | `/api/assets` | List |
| POST | `/api/assets` | Create |
| PUT | `/api/assets/:id` | Update |
| POST | `/api/assets/:id/depreciate` | Compute period depreciation |
