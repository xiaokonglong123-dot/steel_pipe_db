# 后端详细设计 (Detailed Design)

> 对应 PRD。重点是修正 v2 的两类问题：①领域模型不干净 ②repo 臃肿。技术栈不变。

## 1. 分层与责任

```
main.rs        启动：tracing + pool + migrate + bootstrap_admin + serve
config.rs      环境变量
error.rs       AppError + IntoResponse (不泄露 SQL) + ErrorCode
response.rs    ApiResponse<T> / PaginatedResponse<T> / Meta
db.rs          SqlitePool + sqlx::migrate!
middleware/    auth.rs(JWT→AuthUser Extension)  rbac.rs(查库校验 require_permission)
http/          每业务域一个模块：handler(thin, 只做 bind/validate/调用/包装)
services/      业务规则 + 事务边界 + 领域编排（唯一业务逻辑所在地）
repos/         纯 SQL，一聚合一文件，单文件 ≤ 300 行
domain/        真领域模型：值对象 + 各聚合状态机 + 领域错误
```

### 1.1 分层约束（写进 review 清单）
- http 不写 if 业务分支（只做参数解析与响应包装）。
- service 不写 SQL（调 repo）。
- repo 不含业务判断，只做参数化查询 + 行→结构体映射。
- 事务由 service 用 `repo.begin()`/`pool.begin()` 开启并贯穿多个 repo 调用。
- 金额/数量比较计算一律 Decimal，禁止 f64。

---

## 2. 领域模型 (domain/)

### 2.1 值对象
- `domain/money.rs`: `Money(Decimal)`；parse/serialize(JSON string)；金额比较、round_dp(4)。
- `domain/quantity.rs`: `Quantity(Decimal)`；**改为 serialize 为 JSON string**（与金额一致，去掉 to_f64）。
- 前端: TS 以 `string` 接收，再用轻量 util 计算；禁止 `parseFloat` 直接参与加法累计。

### 2.2 状态机 —— 按聚合拆分（关键修正）
不再共用一个 `OrderStatus`。定义两个独立状态机，各自带合法迁移表：

```rust
// domain/purchasing.rs
enum PurchaseOrderStatus { Draft, Submitted, Approved, Rejected, Cancelled, Ordered, PartiallyReceived, Received }
impl PurchaseOrderStatus {
  fn can_transition(&self, action: PoAction) -> Option<PurchaseOrderStatus> { ... } // 唯一权威迁移表
  fn allowed_actions(&self) -> &'static [&'static str]  // 给前端用
}
// doc_status: 0草稿 1已提交 2已取消 (保持不变)
```
```rust
// domain/sales.rs
enum SalesOrderStatus { Draft, Submitted, Approved, Rejected, Cancelled, AwaitingShipment, PartiallyShipped, Shipped }
```
迁移规则完全由这两个枚举的 `can_transition/allowed_actions` 表达；service 只调用它，不再散落字符串比较。

### 2.3 聚集边界
- Inventory 聚合：`Inventory(物化余额)` + 事件 `InventoryLog`；写余额必同事务写 log。
- Reservation：面向 SalesOrder 行的 ATP 预留。
- workflow 实例是与单据解耦的平台聚合（pk = business_type + business_id）。


## 3. Database schema

结构沿用 v2（migrations 001–013 不变更不删改），新增一律 `014+` 追加。
仅列关键表与本次注意的列。所有金额/数量列 = TEXT（存 `Decimal::to_string()`）。软删除 = `deleted_at`。

（示例）purchase_orders:
```
id INTEGER PK, order_no TEXT UNIQUE, supplier_id INTEGER NOT NULL REFERENCES suppliers,
status TEXT NOT NULL, doc_status INTEGER NOT NULL DEFAULT 0,
expected_date TEXT NULL, total_amount TEXT NOT NULL, remark TEXT,
created_by INTEGER, created_at, updated_at, deleted_at
purchase_order_items(id, po_id REF, item_id REF, quantity TEXT, unit_price TEXT, line_total TEXT)
```
- total_amount = Σ line_total，service 用 Decimal 计算后写库。
- 列表/详情 API join `suppliers` 返回 `supplier_name` 等（仅查询投影，不冗余存储）。

完整性约束/索引（保持 v2 已建）: sku/code/order_no UNIQUE；常用筛选列索引；外键启用 `PRAGMA foreign_keys=ON`。

SQLite 单事务写收支: update stock + insert log 在一个 `sqlx::Transaction`。

---

## 4. 事务边界（service）

| 操作 | 事务内动作 |
|---|---|
| 入库过账 | update inventory balance + insert inbound_items/records + insert inventory_logs |
| 出库过账 | check stock + update balance + insert outbound + logs |
| 采购收货 | update PO(状态) + 走"入库过账"事务 |
| 销售发货 | 释放对应 reservation + 走"出库过账"事务 + update SO |
| 日记账 | 校验借贷平衡(Decimal) → insert header+lines |
| 盘点过账 | 差异 → 生成 adjust 出入库 + 写 logs |

---

## 5. API 契约

### 5.1 统一响应
```json
成功(单条): { "success": true, "request_id": "...", "data": { ... } }
成功(分页): { "success": true, "request_id": "...", "data": { "items": [...] }, "meta": { "total": N, "page": P, "page_size": S, "total_pages": N } }
失败:      { "success": false, "code": 11001, "request_id": "...", "message": "...", "details": null }
```
- 201 创建 / 204 删除 / 200 其余
- **Decimal 字段一律序列化为 JSON string**（`"unit_price": "12.5000"`）

### 5.2 名称投影（新）
列表/详情中所有外键以 “id + name” 双字段返回：
```json
{ "id": 12, "supplier_id": 3, "supplier_name": "钢材厂A", "items":[{"item_id":7,"item_name":"钢板","sku":"ST-001","quantity":"10","unit_price":"100"}], ... }
```
实现：repo 列表查询 LEFT JOIN 相应主数据表。

### 5.3 allowed_actions（新）
单据 GET 详情/列表行额外返回：
```json
"allowed_actions": ["submit"]   // 当前状态+操作者权限下合法动作
```
计算：`status.allowed_actions() ∩ 当前用户权限映射`。前端按钮仅据此渲染。

### 5.4 错误码(分域)
| 域 | 范围 |
|---|---|
| 通用 | 1000x |
| Auth | 110xx |
| 商品 | 120xx |
| 库存 | 130xx |
| 订单 | 140xx |
| 往来单位 | 150xx |
| 财务 | 160xx |
| 审批 | 170xx |
| 报表 | 180xx |
| DB | 50001 |
`From<sqlx::Error>` 一律转 50001，不回显原始错误。

### 5.5 端点骨架（对齐 v2，保持路径稳定）
- POST /auth/login /refresh /logout; GET/POST /users ...
- CRUD `/items /suppliers /customers /locations /warehouses`
- 库存：`/inbound` `/outbound` `/inventory` `/inventory/logs` `/inventory/available` `/checks`
- `/purchase-orders`(+`/:id submit|approve|reject|cancel|receive`)；`/sales-orders`(+`ship`)
- `/accounts` `/journal-entries` `/invoices` `/payments` `/trial-balance`
- `/workflow-definitions` `/workflow-instances` `/workflow-tasks`(+`/:id/complete`)
- `/reports/*` 与 `?format=csv`

---

## 6. Repo 拆分（治理臃肿）

```
repos/
  auth_repo.rs            (user/role/perm/token)
  catalog_repo.rs         (item)
  parties_repo.rs         (supplier/customer)
  inventory/
    mod.rs
    balance_repo.rs       (inventory 物化余额)
    log_repo.rs           (inventory_logs、追溯)
    inbound_repo.rs  outbound_repo.rs
    check_repo.rs         (盘点)
    reservation_repo.rs   (ATP)
  purchase_repo.rs
  sales_repo.rs
  finance/
    account_repo.rs  journal_repo.rs  invoice_repo.rs  payment_repo.rs
  workflow/
    definition_repo.rs (workflows/states/transitions)
    instance_repo.rs   (workflow_instances)
    task_repo.rs       (workflow_tasks)
  reports_repo.rs
```
规则：文件 > 300 行即提出拆分；同一 service 事务可调用多 repo 并共享 `&mut Transaction`。

---

## 7. 关键服务伪代码（示例：采购收货+入库存）
```
service::receive_purchase(po_id, lines, user):
  tx = pool.begin()
  po = purchase_repo.get_for_update(tx, po_id)
  assert po.status in {Approved, Ordered, PartiallyReceived}
  purchase_repo.apply_receipt(tx, po, lines)          // 更新 ordered/received qty + 状态过渡
  for line in lines:
     balance_repo.upsert_add(tx, item_id, loc, qty)
     log_repo.insert(tx, {ref: po, type: inbound, qty})
  tx.commit()
```

---

## 8. 测试策略（继承并强化）
- service 单测: 状态机迁移、借贷平衡、ATP 预留释放分支。
- repo/integration: 事务原子性（过账失败回滚）、订单状态联动。
- e2e: PO→approve→receive；SO→approve→ship；盘点调整。
- 重写期间保持现有 124 测试绿色（允许随搬移改内部访问路径，不允许放松断言语义）。
