# Phase 2 — 后端：数据导入导出模块 (P1)

> 基于：`docs/需求文档.md` §3.5；`docs/详细设计文档.md` §11

## 任务清单

### 1.1 依赖
- [ ] 添加 Cargo 依赖：`calamine`（读 Excel）、`xlsxwriter` 或 `rust_xlsxwriter`（写 Excel）、`csv`（CSV 读写）

### 1.2 导入功能
- [ ] 实现 Excel 解析器：
  - 支持 `.xlsx` / `.xls` 格式读取
  - 自动检测表头行并映射到结构体字段
  - 支持中英文表头自动匹配
  - 逐行校验数据格式和必填字段
- [ ] 实现 CSV 解析器：
  - 支持 UTF-8 / GBK 编码自动检测
  - 逗号/制表符分隔自动识别
- [ ] 实现导入 Service：
  - `import_seamless_pipes(file)`：批量导入无缝钢管
  - `import_screen_pipes(file)`：批量导入筛管
  - 事务：全部成功才提交，部分失败回滚并返回错误行
  - 生成导入结果报告（成功行数 / 失败行数 + 每行错误原因）
  - 管材编号重复检测与处理策略（跳过/覆盖/自动生成新编号）
- [ ] 实现导入端点：
  - `POST /api/v1/import/seamless-pipes` — 上传文件导入无缝钢管
  - `POST /api/v1/import/screen-pipes` — 上传文件导入筛管
  - `GET /api/v1/import/template/seamless-pipes` — 下载导入模板
  - `GET /api/v1/import/template/screen-pipes` — 下载导入模板

### 1.3 导出功能
- [ ] 实现 Excel 生成器：
  - 支持多 Sheet 导出
  - 自动设置列宽、表头样式（工业蓝主题）
  - 大数据量分 Sheet 写入
- [ ] 实现 CSV 生成器
- [ ] 实现导出 Service：
  - `export_inventory_report(filter)`：导出库存报表
  - `export_inbound_detail(filter)`：导出入库明细
  - `export_outbound_detail(filter)`：导出入库明细
  - `export_pipe_list(filter)`：导出管材列表
  - 支持选择导出字段
  - 支持分页导出（全部数据流式写入）
- [ ] 实现导出端点：
  - `POST /api/v1/export/inventory` — 导出库存报表
  - `POST /api/v1/export/inbound` — 导出入库明细
  - `POST /api/v1/export/outbound` — 导出入库明细
  - `POST /api/v1/export/pipes` — 导出管材列表
  - （请求体指定文件格式：excel/csv + 筛选条件）

### 1.4 测试
- [ ] 测试导入模板生成（格式正确性）
- [ ] 测试 1 万条数据导入性能（应 ≤ 10 秒）
- [ ] 测试导入数据校验（必填缺失、格式错误、重复编号）
- [ ] 测试导出文件可读性（用 calamine 回读验证）

> **依赖**: 管材管理模块（引用 pipe 类型/字段映射）
