# Phase 1 — 前端：管材管理模块 (P0 MVP)

> 基于：`docs/前端设计文档.md` §2, §3, §4.2, §6, §8

## 任务清单

### 1.1 项目初始化
- [ ] 初始化 Vite + React 19 + TypeScript 项目 (`npm create vite`)
- [ ] 安装核心依赖：antd 5, @ant-design/icons, react-router-dom 7, @tanstack/react-query 5, zustand 5, axios, react-i18next, i18next, dayjs, zod
- [ ] 配置 `vite.config.ts`（API 代理到 `/api/v1`）
- [ ] 配置 TypeScript 严格模式 + 路径别名 `@/`
- [ ] 配置 ESLint + Prettier

### 1.2 基础设施
- [ ] 创建 `src/api/client.ts`：Axios 实例 + 请求拦截器（注入 JWT）+ 响应拦截器（401 刷新 token）
- [ ] 创建 `src/api/queryClient.ts`：TanStack Query Client 配置（staleTime: 2min, gcTime: 5min）
- [ ] 创建 `src/i18n/index.ts`：i18next 初始化（中英文资源文件骨架）
- [ ] 创建国际化资源文件：`zh/common.json`、`en/common.json`、`zh/pipes.json`、`en/pipes.json`
- [ ] 创建 `src/styles/theme.ts`：Ant Design 5 工业蓝主题配置
- [ ] 创建 `src/styles/global.less`：全局样式覆盖

### 1.3 共享组件
- [ ] 实现 `PageContainer`：白色卡片容器组件，统一内边距
- [ ] 实现 `PageHeader`：页面标题 + 面包屑 + 右侧操作按钮
- [ ] 实现 `ErrorBoundary`：React Error Boundary
- [ ] 实现 `ConfirmModal`：二次确认弹窗（删除/取消操作）
- [ ] 实现 `LoadingSpin`：全局/区域加载状态
- [ ] 实现 `EmptyState`：空数据占位组件

### 1.4 管材共享类型与 API
- [ ] 定义 `features/pipes/types.ts`：SeamlessPipe, ScreenPipe, PipeFilter, PaginatedResponse 等类型
- [ ] 定义 `features/pipes/api/pipeApi.ts`：
  - `getSeamlessPipes(filters, page, pageSize)`
  - `getSeamlessPipe(id)`
  - `createSeamlessPipe(data)`
  - `updateSeamlessPipe(id, data)`
  - `deleteSeamlessPipe(id)`
  - `getScreenPipes(...)`、`getScreenPipe(...)`、`createScreenPipe(...)`、`updateScreenPipe(...)`、`deleteScreenPipe(...)`
  - `searchPipes(query)`
- [ ] 实现 `hooks/useSeamlessPipes.ts`（React Query hooks：useSeamlessPipes, useSeamlessPipe, useCreateSeamlessPipe, useUpdateSeamlessPipe, useDeleteSeamlessPipe）
- [ ] 实现 `hooks/useScreenPipes.ts`（同上，针对筛管）

### 1.5 管材共享组件
- [ ] 实现 `PipeFilterBar`：钢级下拉、外径/壁厚范围输入、状态下拉、库位下拉、搜索框、查询/重置按钮
- [ ] 实现 `PipeTable`：Ant Design Table 封装（含分页、排序、操作列）
- [ ] 实现 `GradeTag`：钢级标签组件（按钢级分组显示不同颜色）
- [ ] 实现 `PipeStatusBadge`：库存状态徽标（在库🟢/已出🔵/报废🔴）
- [ ] 实现 `PipeDetailCard`：管材详情信息卡片

### 1.6 无缝钢管页面
- [ ] 实现 `SeamlessPipeListPage`：
  - 顶部操作栏（+新增、导入、导出按钮）
  - FilterBar + PipeTable 组合
  - 行操作：查看详情、编辑、删除
  - 批量选择 + 批量操作
- [ ] 实现 `SeamlessPipeFormPage`：
  - Ant Design Form，分区：基本信息、接箍信息、生产信息
  - 字段联动选择（钢级级联、规格自动带出参考重量）
  - Zod 表单校验
  - 提交 + 取消按钮
- [ ] 实现 `SeamlessPipeDetailPage`：
  - PipeDetailCard 展示所有字段
  - 关联出入库记录 Tab
  - 操作日志 Tab
  - 编辑/删除按钮

### 1.7 筛管页面
- [ ] 实现 `ScreenPipeListPage`（同无缝钢管列表结构，增加筛管特有筛选条件）
- [ ] 实现 `ScreenPipeFormPage`（含筛管特有字段：筛管类型、缝宽、过滤精度、基管参数）
- [ ] 实现 `ScreenPipeDetailPage`（展示筛管所有字段 + 关联记录）

### 1.8 统一搜索页面
- [ ] 实现 `UnifiedPipeSearchPage`：
  - 全局搜索框（支持管材编号/钢级/炉批号模糊搜索）
  - 搜索结果分 Tab 展示（无缝钢管 / 筛管）
  - 点击结果跳转到对应详情页

### 1.9 常量与工具函数
- [ ] 实现 `shared/utils/constants.ts`：钢级列表、端部类型列表、筛管类型列表
- [ ] 实现 `shared/utils/format.ts`：日期格式化、数字格式化、枚举值映射
- [ ] 实现 `shared/utils/pipe-number.ts`：管材编号生成/解析工具

> **依赖**: 基础设施（Axios, QueryClient, i18n, theme）
