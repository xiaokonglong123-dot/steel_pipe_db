# Phase 3 — 后端：合同管理模块 (P2)

> 基于：`docs/需求文档.md` §3.4；`docs/详细设计文档.md` §5.3.13

## 任务清单

### 1.1 数据库迁移
- [ ] 创建 `contracts` 表迁移（合同主表）
- [ ] 创建 `contract_items` 表迁移（合同明细）
- [ ] 创建 `contract_payments` 表迁移（合同付款计划）

### 1.2 领域层
- [ ] 定义 `Contract`、`ContractItem`、`ContractPayment` 结构体
- [ ] 定义枚举：`ContractStatus` (Draft / Active / Completed / Terminated)、`ContractType` (Sales / Purchase)
- [ ] 定义 DTO：`CreateContractDto`、`UpdateContractDto`

### 1.3 仓库层
- [ ] 实现 `ContractRepo`：CRUD + list（含筛选：类型、状态、日期范围、关联客户/供应商）
- [ ] 实现 `ContractItemRepo`：batch create/update，按 contract_id 查询
- [ ] 实现 `ContractPaymentRepo`：CRUD，按 contract_id 查询

### 1.4 服务层
- [ ] 实现 `ContractService`：
  - `create(dto)`：创建合同（含合同编号自动生成）
  - `update(id, dto)`：修改合同基本信息
  - `update_status(id, status)`：状态流转（draft→active→completed/terminated）
  - `get_with_items(id)`：合同 + 明细 + 付款计划
  - `list(filter)`：条件列表分页查询
  - `add_payment(id, dto)` / `update_payment(id, dto)` / `delete_payment(id)`

### 1.5 处理器层
- [ ] `GET /api/v1/contracts` — 合同列表
- [ ] `POST /api/v1/contracts` — 新建合同
- [ ] `GET /api/v1/contracts/{id}` — 合同详情（含明细 + 付款计划）
- [ ] `PUT /api/v1/contracts/{id}` — 更新合同
- [ ] `PUT /api/v1/contracts/{id}/status` — 更新状态
- [ ] `DELETE /api/v1/contracts/{id}` — 删除合同
- [ ] `POST /api/v1/contracts/{id}/payments` — 添加付款计划
- [ ] `PUT /api/v1/contracts/{id}/payments/{payment_id}` — 更新付款计划
- [ ] `DELETE /api/v1/contracts/{id}/payments/{payment_id}` — 删除付款计划

### 1.6 测试
- [ ] 测试合同 CRUD
- [ ] 测试状态流转逻辑
- [ ] 测试合同编号生成

> **依赖**: 采购/销售管理模块（引用客户/供应商数据）
