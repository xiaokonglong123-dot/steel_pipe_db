# Phase 3 — 前端：标签打印模块 (P2)

> 基于：`docs/前端设计文档.md` §4.1

## 任务清单

### 1.1 共享类型与 API
- [ ] 定义 `features/labels/types.ts`：LabelTemplate, LabelConfig 等类型
- [ ] 定义 `features/labels/api/labelApi.ts`：
  - 模板 CRUD
  - 标签生成
  - 打印历史

### 1.2 标签模板管理页面
- [ ] 实现 `LabelTemplateListPage`：
  - 模板列表（名称、宽度/高度、最后修改时间、操作）
  - 新增/编辑/删除模板
- [ ] 实现 `LabelTemplateFormPage`：
  - 基本信息：模板名称、宽度、高度（mm）
  - 可视化编辑（简化版）：选择要显示的字段 + 字体大小 + 对齐方式
  - 预览：模拟显示标签效果

### 1.3 标签生成与打印页面
- [ ] 实现 `LabelGeneratePage`：
  - 选择标签模板
  - 选择管材（批量选择）：管材编号输入框（逗号分隔/批量粘贴） + 从列表选取
  - 预览生成的标签排版
  - 打印按钮 → 调用生成 API → 下载 PDF / 打印

### 1.4 共享组件
- [ ] 实现 `TemplatePreview`：标签模板预览组件（实时显示配置效果）
- [ ] 实现 `PipeSelector`：管材批量选择器（编号搜索 + 列表多选）

> **依赖**: 管材管理前端模块（PipeSelector）
