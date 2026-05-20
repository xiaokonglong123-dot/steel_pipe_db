# Phase 2 — 后端：质量管理模块 (P1)

> 基于：`docs/需求文档.md` §3.3；`docs/详细设计文档.md` §4.4, §5.3.14, §5.3.18, §6.4

## 任务清单

### 1.1 数据库迁移
- [ ] 创建 `quality_certs` 表迁移（质检证书）
- [ ] 创建 `api_5ct_grade_ref` 表迁移（API 5CT 钢级参考数据）
- [ ] 创建 `pipe_attachments` 表迁移（管材附件）

### 1.2 领域层
- [ ] 定义 `QualityCert` 结构体
- [ ] 定义 `Api5ctGradeRef` 结构体
- [ ] 定义 `PipeAttachment` 结构体
- [ ] 定义 DTO：`CreateQualityCertDto`、`UpdateQualityCertDto`
- [ ] 定义枚举：`CertResult` (Pass / Fail / Pending)

### 1.3 仓库层
- [ ] 实现 `QualityCertRepo`：create, update, find_by_id, list, find_by_pipe(pipe_type, pipe_id)
- [ ] 实现 `GradeRefRepo`：get_by_grade(grade), list_all
- [ ] 实现 `AttachmentRepo`：create, delete, find_by_pipe(pipe_type, pipe_id)

### 1.4 服务层
- [ ] 实现 `QualityService`：
  - `create_cert(dto)`：创建质检证书，关联管材
  - `update_cert(id, dto)`：更新质检记录
  - `list_certs(filter)`：条件查询质检证书列表
  - `trace_by_heat_number(heat_no)`：通过炉批号追溯质检记录
  - `trace_by_pipe_number(pipe_no)`：通过管材编号追溯
  - `get_api_5ct_reference(grade)`：获取钢级力学/化学参考数据
  - `attach_file(pipe_type, pipe_id, file)`：上传附件

### 1.5 处理器层
- [ ] `GET /api/v1/quality/certs` — 质检证书列表
- [ ] `POST /api/v1/quality/certs` — 新增质检证书
- [ ] `PUT /api/v1/quality/certs/{id}` — 更新质检证书
- [ ] `GET /api/v1/quality/certs/{id}` — 质检证书详情
- [ ] `GET /api/v1/quality/trace/heat-number/{heat_no}` — 炉批号追溯
- [ ] `GET /api/v1/quality/trace/pipe-number/{pipe_no}` — 管材编号追溯
- [ ] `GET /api/v1/quality/api5ct/grades` — API 5CT 钢级参考列表
- [ ] `GET /api/v1/quality/api5ct/grades/{grade}` — 单钢级参考详情
- [ ] `POST /api/v1/quality/attachments` — 上传附件
- [ ] `DELETE /api/v1/quality/attachments/{id}` — 删除附件

### 1.6 数据初始化
- [ ] 在迁移中插入 API 5CT 常用钢级的力学性能参考数据（见需求文档 §5.1）

### 1.7 测试
- [ ] 测试质检证书 CRUD
- [ ] 测试追溯接口准确性
- [ ] 测试附件上传/删除

> **依赖**: 管材管理模块（引用 pipe ID）
