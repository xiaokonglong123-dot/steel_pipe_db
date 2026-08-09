# 010 — 项目管理 (Phase 3)

> **版本**: v1.0 | **日期**: 2026-08-02 | **依赖**: 005-procurement, 006-sales-crm | **状态**: Draft

---

## 1. 目标

对每个项目/服务项目做全生命周期追踪：立项、预算、WBS、日程、里程碑、与该项目的采购订单/销售订单映射。

## 2. 功能

- **Project Registry** — 立项、名称、项目经理、预算详情
- **WBS 分解** — 工作分解结构 + 进度计划
- **预算控制** — project budget + 实时消耗比对
- **Schedule** — 里程碑
- **单据映射** — 将采购订单/销售订单/合同挂接到项目

## 3. 数据模型

> 数据库为 SQLite3（`sqlite://data/erp.db?mode=rwc`）。

```sql
CREATE TABLE projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_no TEXT UNIQUE,
    name TEXT,
    customer_id INTEGER,
    start_date TEXT,
    end_date TEXT,
    budget_amount REAL DEFAULT 0,
    actual_spent REAL DEFAULT 0,
    status TEXT DEFAULT 'draft'
);

CREATE TABLE wbs_elements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER,
    parent_id INTEGER,
    name TEXT,
    wbs_code TEXT,
    budget_amount REAL,
    actual_amount REAL
);

CREATE TABLE project_transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER,
    wbs_element_id INTEGER,
    source_type TEXT,
    source_id INTEGER,
    amount REAL,
    transaction_date TEXT
);
```

## 4. API

| Method | Path | Description |
| -------- | ------ | ------------- |
| GET | `/api/projects` | List projects |
| POST | `/api/projects` | Create project |
| GET | `/api/projects/:id/wbs` | WBS tree |
| GET | `/api/projects/:id/financials` | 预算 + 实际 vs 预算 |

## 5. 前端

- `features/projects/pages/ProjectListPage.tsx`
- `features/projects/pages/ProjectDetailPage.tsx` with WBS + financial widgets
