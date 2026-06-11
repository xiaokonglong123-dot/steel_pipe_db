# Steel Pipe DB — 代码逻辑审计报告

**审计日期**: 2026-05-28
**审计范围**: 后端 (Rust Axum) + 前端 (React/TypeScript)
**方法**: 逐文件静态审查，聚焦逻辑错误、类型不匹配、数据一致性、资源管理

---

## 汇总

| 严重程度 | 后端 | 前端 | 合计 |
|----------|------|------|------|
| 🔴 Critical | 0 | 2 | 2 |
| 🔴 High | 2 | 5 | 7 |
| 🟡 Medium | 4 | 5 | 9 |
| 🔵 Low | 6 | 8 | 14 |
| **总计** | **14** | **25** | **39** |

---

## 后端发现 (Rust)

### 🔴 HIGH

#### B1 — 采购/销售订单创建缺少事务
- **文件**: `backend/src/repositories/purchase_order_repo.rs:18-54`, `sales_order_repo.rs:17-53`
- **问题**: `create_with_items` 方法包含三步操作（插入订单头 → 插入明细 → 更新总金额），**未包裹在事务中**。若中间步骤崩溃，数据库留下不一致的半完成记录。
- **对比**: `InboundRepo::create_with_items` 和 `OutboundRepo::create_with_items` 正确使用了 `pool.begin()` / `tx.commit()`。
- **修复**: 将 `purchase_order_repo.rs:18-54` 和 `sales_order_repo.rs:17-53` 改为使用 `let mut tx = pool.begin().await?; ... tx.commit().await?;`。
- **影响**: 数据完整性

#### B2 — 合同 payment_type CHECK 约束与代码不匹配
- **文件**: `backend/migrations/009_create_ref_data.sql:53`, `backend/src/dto/contract_dto.rs:136-138`
- **问题**: SQL 约束 `CHECK (payment_type IN ('deposit', 'milestone', 'final'))` 只接受 3 个值，但 DTO 注释写的是 `deposit / progress / final / retention`。若 handler 放行 `"progress"` 或 `"retention"`，DB 层报 500 错误。
- **修复**: 要么更新 SQL 约束增加 `'progress', 'retention'`，要么更新 DTO 只允许 `'deposit', 'milestone', 'final'`。需确认业务意图后决定。
- **影响**: 数据库拒绝合法请求 → 500 错误

### 🟡 MEDIUM

#### B3 — 合同 status CHECK 不包含 `cancelled`
- **文件**: `backend/migrations/009_create_ref_data.sql:55-59`
- **问题**: 约束 `CHECK (status IN ('draft', 'pending_approval', 'approved', 'active', 'completed', 'expired'))` 无 `'cancelled'`，但代码多处支持该值。提交 `cancelled` 状态会报 500。
- **修复**: 添加 `'cancelled'` 到 CHECK 约束。

#### B4 — 合同 update_total_amount 未检查 soft-delete
- **文件**: `backend/src/repositories/contract_repo.rs:176-186`
- **问题**: `UPDATE contracts SET total_amount = ? WHERE id = ?` 缺少 `AND deleted_at IS NULL`。可更新已软删除的合同。
- **修复**: 添加 `AND deleted_at IS NULL` 条件。

#### B5 — 金额以 SQLite REAL (f64) 存储导致精度丢失
- **文件**: `backend/migrations/006_create_orders.sql`, `backend/migrations/009_create_ref_data.sql`, 多处 model/repo
- **问题**: 金额字段 (`total_amount`, `unit_price` 等) 使用 `DOUBLE` (f64) 存储。Rust 端 `Decimal` → `f64` → `Decimal` 的来回转换导致精度丢失（`0.1 + 0.2 ≠ 0.3`）。
- **建议**: 这是架构层面问题，短期可通过确保计算在 Decimal 域内完成来缓解。长期应迁移到 TEXT 存储 Decimal 字符串。
- **影响**: 财务计算可能出现分分差

#### B6 — 库存盘点 is_match 逻辑过于简单
- **文件**: `backend/src/repositories/inventory_repo.rs:1070`
- **问题**: `is_match` 仅检查 `found_status == "in_stock"`，而非比较 `found_status == expected_status`。这忽略了 expected_status 为其他值（如 `reserved`）时的正确性检查。
- **修复**: 改为 `found_status == expected_status` 比较。

### 🔵 LOW

#### B7 — 动态筛选器将数值绑定为字符串
- **文件**: `backend/src/repositories/pipe_repo.rs:157-165`, `inventory_repo.rs:544-545`
- **问题**: `push_bind(ParamValue::Float(v))` 最终作为 TEXT 绑定到 SQLite。范围比较变成 ASCII 字符串比较：`'9.7' >= '10.0'` 返回 `true`。
- **修复**: 对浮点字段使用 `CAST(? AS REAL)` 或改用参数化数值绑定。

#### B8 — Pipe model 注释含 DB 无的状态值
- **文件**: `backend/src/models/seamless_pipe.rs:45`, `screen_pipe.rs:43`
- **问题**: 注释列出了 `in_transit` / `reserved` 等状态，但数据库 CHECK 约束不包含。注释与实际不一致。
- **修复**: 更新注释使其与 DB 约束一致。

#### B9 — ContractPayment::is_paid 类型为 i64 而非 bool
- **文件**: `backend/src/models/contract.rs:104`
- **问题**: `is_paid: i64` 与其他布尔字段（`use_status: bool`）风格不一致。SQLite 以 0/1 存储 bool，i64 可以工作但语义不清晰。
- **修复**: 改为 `bool` 类型。

#### B10 — Cargo.toml 含 rust_decimal 但 AGENTS.md 声称不存在
- **文件**: `backend/Cargo.toml:13-14`
- **问题**: `rust_decimal = "1.36"` 和 `rust_decimal_macros = "1.36"` 已在依赖中，但项目文档声称"不存在 rust_decimal"。
- **修复**: 更新 AGENTS.md 反映实际依赖。

#### B11 — .env.example 缺少 REFRESH_TOKEN_EXPIRY_DAYS
- **文件**: `backend/.env.example`
- **问题**: `config.rs:69` 读取 `REFRESH_TOKEN_EXPIRY_DAYS` 但 .env.example 未列出此项。
- **修复**: 添加到 .env.example。

#### B12 — 服务层直接写原始 SQL
- **文件**: `backend/src/services/purchase_sales_service.rs:721`, `outbound_service.rs:118`, `inventory_service.rs:108`, `quality_service.rs:59`, `contract_service.rs:38`
- **问题**: 服务方法直接执行 SQL 而不通过 repository 层。虽在事务中但破坏了分层架构。
- **建议**: 将 SQL 移入对应的 repository 方法。

### ✅ 已验证无问题

| 类别 | 结论 |
|------|------|
| SQL 注入 | 所有 QueryBuilder 使用 `push_bind()`，无注入风险 |
| Soft-delete 筛选 | 除 B4 外所有查询均正确含 `deleted_at IS NULL` |
| Handler 验证 | 所有 51 个 POST/PUT handler 均调用 `.validate()` |
| 入库/出库/盘点事务 | 正确使用 `pool.begin()` / `tx.commit()` |
| 中间件 | 认证和 RBAC 正确配置 |
| 错误处理 | `AppError::into_response()` 自动包含 `request_id` |

---

## 前端发现 (React/TypeScript)

### 🔴 CRITICAL

#### F1 — 会话恢复失败后应用永久卡在加载
- **文件**: `frontend/src/features/auth/hooks/useAuth.ts:34-58`
- **问题**: `useRestoreSession()` 中，`.catch()` 只包裹了 `refresh()` 调用。若 `refresh()` 成功但 `getMe()` 失败，rejection 未被捕获。`setRestoring(false)` 永不执行，`ProtectedRoute` 永久渲染 `<Spin>`，**整个应用无法使用**。
- **修复**: 用 `.then(() => setRestoring(false)).catch(...)` 包裹整个链，或用 `try/catch` + `finally`。

#### F2 — i18n 命名空间加载模式与 useTranslation 不匹配
- **文件**: `frontend/src/i18n/index.ts:~70-90`
- **问题**: Feature 翻译文件 (`profile.json`, `search.json` 等) 被加载到 `translation` 命名空间下，结构为 `{ [moduleName]: data }`。但页面使用 `useTranslation('profile')` 查找 `resources.en.profile`，而数据实际在 `resources.en.translation.profile["profile.title"]`。所有 feature 页面的翻译可能失效。
- **修复**: 统一加载模式——要么所有 feature 以独立命名空间加载（`useTranslation('profile')` → `resources.en.profile`），要么保持当前模式但更新所有 `useTranslation` 调用。

### 🔴 HIGH

#### F3 — App.tsx 重复创建 QueryClient，丢失全局配置
- **文件**: `frontend/src/App.tsx:12-21`
- **问题**: 内联创建 `new QueryClient({})`，未导入共享的 `@/api/queryClient.ts`。所有全局配置丢失：`staleTime: 2min`, `gcTime: 5min`, `retry: 1`, `refetchOnWindowFocus: false`, 全局 `onError` 均不生效。
- **修复**: 改为 `import { queryClient } from '@/api/queryClient'`。

#### F4 — 供应商/客户点击行导航到不存在的路由
- **文件**: `frontend/src/features/suppliers/pages/SupplierListPage.tsx:~76`, `customers/pages/CustomerListPage.tsx:~68`
- **问题**: `navigate(\`/suppliers/${record.id}\`)` 期望详情页路由，但 `src/routes/index.tsx` 仅有 `/suppliers/new` 和 `/suppliers/:id/edit`，无 `/suppliers/:id`。点击行跳转空白页。
- **修复**: 将 `navigate` 改为 `navigate(\`/suppliers/${record.id}/edit\`)`，或添加详情路由。

#### F5 — 入库/出库编辑模式实际创建新记录
- **文件**: `frontend/src/features/inventory/pages/InboundFormPage.tsx:36`, `OutboundFormPage.tsx:36`
- **问题**: 页面支持 `isEdit` 模式，但只有 `useCreateInbound()` / `useCreateOutbound()`，无 update mutation。提交编辑时创建新记录而非更新旧记录。
- **修复**: 添加 `useUpdateInbound()` / `useUpdateOutbound()` hooks，编辑模式调用 update 而非 create。

#### F6 — 管材选择模态框无法传回选中结果
- **文件**: `frontend/src/features/sales/pages/SalesOrderFormPage.tsx`
- **问题**: 使用 `Modal.confirm` 打开管材选择，但该回调无法将选中管材传回表单 → 无法为销售订单添加明细项。
- **修复**: 改为受控的 `<Modal>` 组件，在 `onOk` 回调中收集选中值。

#### F7 — 搜索页面中文列标题写死
- **文件**: `frontend/src/features/search/pages/SearchPage.tsx:10-44`
- **问题**: 列标题（编号、钢级、外径、壁厚、长度、标准、库存、位置、炉号）为硬编码中文字符串，未使用 `useTranslation`。英文模式下标题仍为中文。
- **修复**: 使用 `t('columns.pipeNumber')` 等 i18n 调用。

### 🟡 MEDIUM

#### F8 — purchaseApi.get / salesApi.get TypeScript 返回类型与 Zod schema 不匹配
- **文件**: `frontend/src/features/purchases/api/purchaseApi.ts:24-26`, `salesApi.ts:24-27`
- **问题**: TypeScript 泛型返回 `ApiResponse<PurchaseOrder>`，但 Zod schema `purchaseOrderDetailSchema` 验证的是 `{ order, items }` 结构。类型声明与运行时数据形状不一致。
- **修复**: 更新返回类型为 `ApiResponse<{ order: PurchaseOrder; items: PurchaseOrderItem[] }>`。

#### F9 — 库存查询 rowKey 包含 page 导致翻页时所有行重新挂载
- **文件**: `frontend/src/features/inventory/pages/StockQueryPage.tsx:~131`
- **问题**: `rowKey={(record) => \`${record.id}-${page}\`}` 将 `page` 包含在 key 中。翻页时所有行获得新 key → 完全重挂载 → 选择/排序状态丢失。
- **修复**: 改为 `rowKey={(record) => record.id}`。

#### F10 — 入库/出库列表审批按钮共享 loading 状态
- **文件**: `frontend/src/features/inventory/pages/InboundListPage.tsx`, `OutboundListPage.tsx`
- **问题**: 审批 mutation 的 `isPending` 被所有行共享 → 点击某行的审批按钮，**所有行的审批按钮都转圈**。
- **修复**: 将 loading 状态绑定到具体行 ID（如 `approvingId` state）。

#### F11 — 销售订单表单 order_date 类型不匹配
- **文件**: `frontend/src/features/sales/pages/SalesOrderFormPage.tsx:36-37`
- **问题**: `form.setFieldsValue({ order_date: order.order_date })` 传 String，但 Ant Design DatePicker 需要 Dayjs 对象。导致表单初始化警告/错误。
- **修复**: `order_date: dayjs(order.order_date)`。

#### F12 — unitStore.ts 缺失
- **文件**: `frontend/src/stores/unitStore.ts`（不存在）
- **问题**: AGENTS.md 记载 `unitStore` 用于度量/英制单位切换，但文件不存在。
- **修复**: 创建 unitStore 或从文档中移除引用。

### 🔵 LOW

#### F13 — 无 404 兜底路由
- **文件**: `frontend/src/routes/index.tsx`
- **问题**: 未知路径渲染空白页。

#### F14 — useChangePassword 无效化 userQueryKeys.all
- **文件**: `frontend/src/features/auth/hooks/useUsers.ts:39-40`
- **问题**: 改密后不必要地刷新用户列表。

#### F15 — LoginPage 冗余 loading 状态
- **文件**: `frontend/src/features/auth/pages/LoginPage.tsx:13, 17-24`
- **问题**: 单独维护 `loading`，与 `loginMutation.isPending` 冗余。

#### F16 — i18n 可能重复注册 purchase/purchases
- **文件**: `frontend/src/i18n/index.ts:~75-85`

#### F17 — useCreateAttachment 未无效化证书详情查询
- **文件**: `frontend/src/features/quality/hooks/useQuality.ts:67-75`

#### F18 — 搜索/提交错误被吞掉
- **文件**: 多个文件的 `catch` 块

#### F19 — useLogout 缺少 useCallback
- **文件**: `frontend/src/features/auth/hooks/useAuth.ts:22-32`

#### F20 — 登录后 getMe 被重复调用
- **文件**: `frontend/src/features/auth/hooks/useAuth.ts:14-15, 41-46`

---

## 修复优先级建议

### Phase 1 — 立即修复（阻塞功能）
1. **F1** — useRestoreSession 错误捕获（应用不可用）
2. **F3** — QueryClient 重复（全局配置丢失）
3. **F5** — 入库/出库编辑模式（创建而非更新）
4. **B1** — PO/SO 事务缺失（数据不一致）
5. **B2** — 合同 payment_type 约束（500 错误）

### Phase 2 — 高优先级
6. **F2** — i18n 命名空间（翻译失效）
7. **F4** — 供应商/客户导航（空白页）
8. **F6** — 管材选择模态框（无法添加明细）
9. **F7** — 搜索页面中文（国际化失败）
10. **B4** — 合同 update_total_amount soft-delete
11. **B6** — 库存盘点 is_match 逻辑

### Phase 3 — 中优先级
12. **F8** — API 类型不匹配
13. **F9** — rowKey 翻页问题
14. **F10** — 审批按钮共享 loading
15. **F11** — 日期类型
16. **B3** — 合同 cancelled 状态
17. **B5** — 金额精度

### Phase 4 — 低优先级
其余 14 个 Low 发现
