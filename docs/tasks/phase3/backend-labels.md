# Phase 3 — 后端：标签打印模块 (P2)

> 基于：`docs/需求文档.md` §3.8；`docs/详细设计文档.md` §10.3

## 任务清单

### 1.1 标签模板管理
- [ ] 创建 `label_templates` 表迁移（模板名称、内容配置）
- [ ] 实现 `GET /api/v1/label-templates` — 模板列表
- [ ] 实现 `POST /api/v1/label-templates` — 创建模板
- [ ] 实现 `PUT /api/v1/label-templates/{id}` — 更新模板
- [ ] 实现 `DELETE /api/v1/label-templates/{id}` — 删除模板
- [ ] 每个模板包含：标签宽度/高度、字体大小、字段位置信息（JSON 配置）

### 1.2 标签生成与打印
- [ ] 实现 `POST /api/v1/labels/generate`：
  - 请求：模板 ID + 管材编号列表
  - 校验管材存在性 + 模板存在性
  - 返回：生成的标签图像 PDF（A4 纸排版，多标签网格排列）
- [ ] 使用 PDF 库（`printpdf`）生成包含条码或二维码的标签
- [ ] 裸管/成品管/接箍分类标签差异化输出
- [ ] 标签内容：管材编号、钢级、规格、长度、重量、生产日期、炉批号
- [ ] `GET /api/v1/labels/print-history` — 打印历史记录

### 1.3 条码/二维码
- [ ] 生成二维码：内容为管材编号 URL（指向详情页）
- [ ] 基于管材编号生成 Code 128 条码（使用 `barcoder` crate 或手动生成）

> **依赖**: 管材管理模块（引用 pipe 数据）
