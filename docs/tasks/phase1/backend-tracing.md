# Phase 1 — 后端：历史追溯模块 (P0 MVP)

> 基于：`docs/需求文档.md` §3.9；`docs/详细设计文档.md` §5.3.6 (inventory_logs), §5.3.16 (operation_logs)

## 任务清单

### 1.1 模块说明
历史追溯不是独立模块，而是贯穿其他所有模块的**横切关注点**。分为两个层面：
- **库存变动追溯**（`inventory_logs`）：每根管材的出入库历史
- **操作审计追溯**（`operation_logs`）：用户操作的审计日志

### 1.2 日志记录基础设施
- [ ] 在 `AuthService` 登录/登出时调用 `OperationLogRepo::create()`
- [ ] 在 `PipeService` 创建/更新/删除管材时记录操作日志（记录变更前后字段 JSON）
- [ ] 在 `InventoryService` 入库/出库/盘点/调拨时记录 inventory_log + operation_log
- [ ] 实现 `OperationLogger` trait 或辅助函数，统一日志记录接口
- [ ] 日志内容包含：user_id, username, action, target_type, target_id, target_summary, detail (JSON diff), ip_address

### 1.3 追溯查询端点
- [ ] `GET /api/v1/trace/pipe/{pipe_type}/{pipe_id}` — 单根管材全生命周期追溯
  - 返回该管材的所有 inventory_logs（入库→在库→出库→报废等）
  - 关联显示对应的入库单/出库单编号
- [ ] `GET /api/v1/trace/heat-number/{heat_number}` — 按炉批号追溯
  - 查找同炉批号的所有管材 + 每根的当前状态
- [ ] `GET /api/v1/trace/order/{order_type}/{order_id}` — 按订单追溯
  - 查询该采购/销售订单关联的所有管材及状态

### 1.4 测试
- [ ] 验证管材创建/修改/删除时 operation_log 正确写入
- [ ] 验证入库/出库时 inventory_log 正确写入
- [ ] 验证追溯接口返回完整生命周期数据

> **依赖**: 管材管理模块、库存管理模块、系统管理模块（用户+日志表）
