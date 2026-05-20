# Phase 2 — 前端：质量管理模块 (P1)

> 基于：`docs/前端设计文档.md` §4.1, §4.2

## 任务清单

### 1.1 共享类型与 API
- [ ] 定义 `features/quality/types.ts`：QualityCert, Api5ctGradeRef, PipeAttachment 等类型
- [ ] 定义 `features/quality/api/qualityApi.ts`：
  - `getQualityCerts(filter, page, pageSize)` / `getQualityCert(id)` / `createQualityCert(data)` / `updateQualityCert(id, data)`
  - `traceByHeatNumber(heatNo)` / `traceByPipeNumber(pipeNo)`
  - `getApi5ctGrades()` / `getApi5ctGrade(grade)`
  - `uploadAttachment(pipeType, pipeId, file)` / `deleteAttachment(id)`

### 1.2 质检证书页面
- [ ] 实现 `QualityCertListPage`：
  - 筛选条件：管材编号、钢级、检测结果、检测日期范围
  - 质检证书表格（证书编号、管材编号、检测日期、检测机构、结果、操作）
  - 行操作：查看详情、编辑、删除
  - +新增质检证书按钮
- [ ] 实现 `QualityCertFormPage`：
  - 选择关联管材（管材搜索选择器）
  - 填写检测信息（证书编号、检测日期、检测机构、检测人员）
  - 检测结果选择（通过/不通过/待定）
  - 检测项目动态添加组件（名称 + 结果 + 标准值）
  - 文件上传（质检报告 PDF/图片）
- [ ] 实现 `QualityCertDetailPage`（可选 Modal 或独立页）：
  - 展示质检证书所有字段
  - 关联管材信息卡
  - 附件列表（下载/预览）

### 1.3 质量追溯页面
- [ ] 实现 `QualityTracePage`：
  - 输入方式选择：按炉批号 或 按管材编号
  - 输入查询条件 → 展示追溯结果
  - 结果展示：管材信息 + 时间线（所有质检记录 + 出入库记录）
  - 使用 Ant Design Timeline 展示

### 1.4 API 5CT 标准参考页面
- [ ] 实现 `Api5ctRefPage`：
  - 钢级列表表格（钢级、分组、最小屈服强度、最大屈服强度、最小抗拉强度、硬度、说明）
  - 钢级筛选/搜索
  - 点击行展开该钢级的详细信息

### 1.5 共享组件
- [ ] 实现 `CertFileUploader`：质检文件上传组件（支持预览 + 拖拽上传）
- [ ] 实现 `TraceTimeline`：追溯时间线组件（不同事件类型不同图标/颜色）
- [ ] 实现 `GradeCompareTable`：质检结果与 API 5CT 标准值的对比表格

### 1.6 国际化
- [ ] 创建 `src/i18n/resources/zh/quality.json` 和 `en/quality.json`

> **依赖**: 管材管理前端模块（管材搜索选择器）
