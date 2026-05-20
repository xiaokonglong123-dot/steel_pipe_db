# 任务拆解 — 总体进度

> 更新日期：2026-05-19
> 技术栈：Rust + Axum + SQLx + SQLite (后端) / Vite + React 19 + Ant Design 5 + TanStack Query + Zustand (前端)

---

## 已生成的任务文件

| 模块 | Phase | 后端 | 前端 | 任务项数 |
|------|-------|------|------|----------|
| 无缝钢管与筛管管理 | P0 | ✅ | ✅ | 18+15 |
| 库存管理 | P0 | ✅ | ✅ | 21+18 |
| 系统管理与认证 | P0 | ✅ | ✅ | 20+19 |
| 历史追溯 | P0 | ✅ | ✅ | 12+4 |
| 质量管理 | P1 | ✅ | ✅ | 18+15 |
| 采购管理 | P1 | ✅ | ✅ | 16+11 |
| 销售管理 | P1 | ✅ | ✅ | 16+12 |
| 数据导入导出 | P1 | ✅ | ✅ | 14+8 |
| 合同管理 | P2 | ✅ | ✅ | 14+12 |
| 报表与统计 | P2 | ✅ | ✅ | 14+15 |
| 标签打印 | P2 | ✅ | ✅ | 12+10 |
| 国际化与单位切换 | P2 | — | ✅ | —+10 |
| **合计** | | **12 个后端模块** | **12 个前端模块** | ~320 项 |

---

## Phase 1 — MVP / P0 最高优先级

> 目标：系统核心骨架，可运行的管材管理+库存管理+用户认证

### 后端模块
- [x] **管材管理** `phase1/backend-pipe-management.md` — 移值初始化→数据库→领域→仓库→服务→处理器→测试
- [x] **库存管理** `phase1/backend-inventory.md` — 库位、入库、出库、盘点、库存查询、盘点
- [x] **系统管理认证** `phase1/backend-auth-system.md` — JWT 认证、RBAC、用户管理、安全配置
- [x] **历史追溯** `phase1/backend-tracing.md` — 日志基础设施 + 追溯 API + 横切集成

### 前端模块
- [x] **管材管理** `phase1/frontend-pipe-management.md` — 列表/表单/详情页、筛选、搜索
- [x] **库存管理** `phase1/frontend-inventory.md` — 入库/出库/盘点/库位页面
- [x] **系统管理认证** `phase1/frontend-auth-system.md` — 登录、布局、用户管理、路由权限
- [x] **历史追溯** `phase1/frontend-tracing.md` — 详情页追溯 Tab

---

## Phase 2 — P1 重要功能

> 目标：核心业务闭环，采购管理 + 销售管理 + 质检 + 数据导入导出

### 后端模块
- [x] **质量管理** `phase2/backend-quality.md`
- [x] **采购管理** `phase2/backend-purchase.md`
- [x] **销售管理** `phase2/backend-sales.md`
- [x] **数据导入导出** `phase2/backend-data-io.md`

### 前端模块
- [x] **质量管理** `phase2/frontend-quality.md`
- [x] **采购管理** `phase2/frontend-purchase.md`
- [x] **销售管理** `phase2/frontend-sales.md`
- [x] **数据导入导出** `phase2/frontend-data-io.md`

---

## Phase 3 — P2 增强功能

> 目标：合同管理 + 报表统计 + 标签打印 + 国际化

### 后端模块
- [x] **合同管理** `phase3/backend-contracts.md`
- [x] **报表与统计** `phase3/backend-reports.md`
- [x] **标签打印** `phase3/backend-labels.md`

### 前端模块
- [x] **合同管理** `phase3/frontend-contracts.md`
- [x] **报表与统计** `phase3/frontend-reports.md`
- [x] **标签打印** `phase3/frontend-labels.md`
- [x] **国际化与单位切换** `phase3/frontend-i18n-units.md`

---

## 输出文件清单

```
docs/tasks/
├── progress.md                    ← 你在这里
├── phase1/
│   ├── backend-pipe-management.md
│   ├── backend-inventory.md
│   ├── backend-auth-system.md
│   ├── backend-tracing.md
│   ├── frontend-pipe-management.md
│   ├── frontend-inventory.md
│   ├── frontend-auth-system.md
│   └── frontend-tracing.md
├── phase2/
│   ├── backend-quality.md
│   ├── backend-purchase.md
│   ├── backend-sales.md
│   ├── backend-data-io.md
│   ├── frontend-quality.md
│   ├── frontend-purchase.md
│   ├── frontend-sales.md
│   └── frontend-data-io.md
└── phase3/
    ├── backend-contracts.md
    ├── backend-reports.md
    ├── backend-labels.md
    ├── frontend-contracts.md
    ├── frontend-reports.md
    ├── frontend-labels.md
    └── frontend-i18n-units.md
```
