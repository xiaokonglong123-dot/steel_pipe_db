# 011 — 固定资产管理 (Phase 4)

> **版本**: v1.0 | **日期**: 2026-08-02 | **依赖**: 004-finance | **状态**: Draft

---

## 1. 目标

管理公司的固定资产：购买登记、折旧摊销、转移、报废、盘存。

## 2. 功能

| 子模块 | 描述 |
|--------|------|
| 资产登记 | 编号、名称、采购、折旧类、年限、生产日期 |
| 折旧计算 | 直线法 / 双倍余额递减、按会计周期生成摊款分录 |
| 资产调拨 | 公司/部门/项目之间的资产移动 |
| 资产处置 | 报废或卖 —  计提 同时关闭 |

## 3. 数据模型

```sql
CREATE TABLE assets.fixed_assets (
    id BIGSERIAL PRIMARY KEY,
    asset_no VARCHAR(100) UNIQUE,
    name VARCHAR(400),
    asset_category VARCHAR(50),
    acquisition_cost NUMERIC(18,2),
    useful_life_years INT,
    residual_value NUMERIC(18,2),
    ials INT DEFAULT 0, -- 当前累计天数
     current_depreciation_method VARCHAR(20) DEFAULT 'straight_line',
    location_id BIGINT,
    created_at TIMESTAMPTZ
);

CREATE TABLE assets.depreciation_entries (
    id BIGSERIAL PRIMARY KEY,
    asset_id BIGINT REFERENCES assets.fixed_assets(id),
    period DATE,   -- month
    depreciated_amount NUMERIC(18,2),
    journal_entry_rel_id BIGINT,       --> finance.journal_entries.id
    created_at TIMESTAMPTZ
);
```

## 4. API

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/assets` | List |
| POST | `/api/assets` | Create |
| PUT | `/api/assets/:id` | Update |
| POST | `/api/assets/:id/depreciate` | Compute period depreciation |