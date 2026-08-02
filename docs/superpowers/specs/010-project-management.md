# 010 — 项目管理 (Phase 3)

> **版本**: v1.0 | **日期**: 2026-08-02 | **依赖**: 005-procurement, 006-sales-crm | **状态**: Draft

---

## 1. 目标

对每个工程/接头/服务项目做全生命周期追踪：立项、预算、WBS、日程、里程碑、与该项目的 PO/SO 映射。

## 2. 功能

- **Project Registry** — 立项、名称、项目经理、基金详情
- **WBS 分解** — 工作分解结构 + 进度计划
- **预算控制** — project budget + 实时消金比对
- **Schedule** — 里程碑
- **N対1账户 mapping** — 将 PO/SO/合同 挂接到项目

## 3. 数据模型

```sql
CREATE TABLE projects.projects (
    id BIGSERIAL PRIMARY KEY,
    project_no VARCHAR(100) UNIQUE,
    name VARCHAR(400),
    customer_id BIGINT,
    start_date DATE, end_date DATE,
    budget_amount NUMERIC(18,2) DEFAULT 0,
    actual_spent NUMERIC(18,2) DEFAULT 0,
    status VARCHAR(20) DEFAULT 'draft'
);

CREATE TABLE projects.wbs_elements (
    id BIGSERIAL, project_id BIGINT, parent_id BIGINT,
    name VARCHAR(200), wbs_code VARCHAR(50),
    budget_amount, actual_amount
);

CREATE TABLE projects.project_transactions (
    id BIGSERIAL,
    project_id BIGINT, wbs_element_id BIGINT,
    source_type VARCHAR(50), source_id BIGINT,
    amount NUMERIC(18,2), transaction_date DATE
);
```

## 4. API

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/projects` | List projects |
| POST | `/api/projects` | Create project |
| GET | `/api/projects/:id/wbs` | WBS tree |
| GET | `/api/projects/:id/financials` | 预算 + 实际 vs 预算 |

## 5. 前端

- `features/projects/pages/ProjectListPage.tsx`
- `features/projects/pages/ProjectDetailPage.tsx` with WBS + financial widgets