# 项目扫描、修复与测试报告

生成时间：2026-05-28 UTC
项目：ERP（通用企业资源计划系统）— 后端 crate `erp-server`（历史沿革：本系统由钢管行业系统重构而来）

## 1. 执行摘要

本轮扫描覆盖了前端构建链路、现有后端测试资产、库存/出入库/采购/销售/制造质检核心业务用例，以及大数据脚本可执行性。已修复前端 P0 类型/构建阻塞，并补齐一个后端测试签名阻塞。后端 `cargo check` / `cargo test` 当前被本机 Cargo 依赖下载环境阻塞，尚未进入 Rust 编译阶段。

## 2. 优先级问题报表

| 优先级 | 问题 | 影响 | 处理状态 | 证据/验证 |
| --- | --- | --- | --- | --- |
| P0 | 前端 API 返回 nullable 字段，但 TS 实体类型仅允许 `undefined` | `npx tsc --noEmit` 与 `npm run build` 失败，影响所有前端交付 | 已修复 | 已通过 `npx tsc --noEmit`、`npm run build` |
| P0 | `package.json` 缺少 `test` 脚本，但项目已有 Vitest 配置与测试 | `npm test -- --run` 失败，CI/本地测试入口不一致 | 已修复 | `npm test -- --run`：3 files / 11 tests passed |
| P0 | 后端服务签名 `InboundService::approve_inbound(..., handled_by)` 已变化，测试仍按旧签名调用 | 依赖环境恢复后，Rust tests 会先出现编译错误 | 已修复 | `backend/tests/inventory_service_test.rs` 三处调用已补 `None` |
| P0 | Cargo 在线下载依赖失败：`/tmp/ca-bundle.crt` CA trust anchor 错误；离线缺 `axum` 缓存 | 后端无法执行 `cargo check` / `cargo test`，阻塞服务端最终验证 | 未修复（环境问题） | `cargo check` 报 `[77] Problem with the SSL CA cert`; `cargo check --offline` 报 no matching package `axum` |
| P1 | 后端库存/出入库已有工作区改动涉及状态机、`handled_by`、库位 `used_count`、出库报废状态 | 业务正确性依赖后端编译和集成测试确认 | 待环境恢复后验证 | 已识别修改文件，无法 cargo 验证 |
| P1 | 前端库存选项、Zod schema、库存页面已有工作区改动 | 改善 API 合约对齐和页面选项，但需联调确认 | 前端已构建验证；后端联调待恢复 | `tsc` / lint / build 通过 |

## 3. 已修复文件清单

### 本轮直接修复

| 文件 | 修复内容 | 预期结果 |
| --- | --- | --- |
| `frontend/package.json` | 新增 `test: vitest` 脚本 | `npm test -- --run` 可执行 |
| `frontend/src/types/index.ts` | `Item`（商品）可空字段改为 `T \| null`，补 `deleted_at` | 全局实体类型匹配后端/Zod nullable 响应 |
| `frontend/src/features/customers/types.ts` | 客户可空字段改为 `T \| null`，补 `deleted_at` | 客户列表/表单类型与 API 响应一致 |
| `frontend/src/features/suppliers/types.ts` | 供应商可空字段改为 `T \| null`，补 `deleted_at` | 供应商查询 hooks 类型恢复 |
| `frontend/src/features/inventory/api/inventoryApi.ts` | 入库、出库、库位、库存日志、盘点、商品搜索结果 nullable 字段对齐 Zod | 库存页面和搜索结果类型恢复 |
| `frontend/src/features/customers/pages/CustomerFormPage.tsx` | 编辑表单 `null` 规范化为 `undefined` | 客户编辑表单类型正确 |
| `frontend/src/features/suppliers/pages/SupplierFormPage.tsx` | 编辑表单 `null` 规范化为 `undefined` | 供应商编辑表单类型正确 |
| `frontend/src/features/inventory/pages/LocationListPage.tsx` | 库位编辑表单 `null` 规范化为 `undefined` | 库位编辑表单类型正确 |
| `backend/tests/inventory_service_test.rs` | `approve_inbound` 测试调用补齐 `handled_by` 参数 | 后端测试源码匹配服务签名 |

### 工作区已有修复（已纳入风险边界）

| 文件 | 观察到的修复方向 | 后续验证 |
| --- | --- | --- |
| `backend/src/dto/item_dto.rs`、`backend/src/services/item_service.rs` | 商品搜索结果从嵌套 JSON 改为扁平 DTO，便于前端选择弹窗使用 | 需 `cargo test`（代码阶段补充商品用例） |
| `backend/src/handlers/inventory_handler.rs`、`backend/src/services/inbound_service.rs`、`backend/src/services/outbound_service.rs` | 审批流程写入 `handled_by`；出库支持 scrapped 状态 | 需 `cargo test inventory_service_test` |
| `backend/src/repositories/inventory_repo.rs`、`backend/src/services/location_service.rs` | 新增/调用库位 `used_count` 刷新 | 需库存出入库/库位转移测试 |
| `frontend/src/zod-schemas/core.ts`、`frontend/src/zod-schemas/inventory.ts` | Zod schema 与后端 nullable 响应对齐 | 已被前端 build 间接验证 |
| `frontend/src/features/inventory/pages/*` | 入库/出库/库存页面选项和状态枚举调整 | 已被前端 build 间接验证，需联调确认 |

## 4. 已执行验证

| 命令 | 结果 | 说明 |
| --- | ---: | --- |
| `cd frontend && npx tsc --noEmit` | 通过 | 无输出，TypeScript P0 清零 |
| `cd frontend && npm test -- --run` | 通过 | 3 个测试文件，11 条用例通过 |
| `cd frontend && npm run lint` | 通过 | ESLint 无报错 |
| `cd frontend && npm run build` | 通过 | Vite production build 成功 |
| `cd backend && python3 -m py_compile seed_data.py seed_data_enhanced.py` | 通过 | 大数据脚本语法可解释执行（代码阶段重写为通用商品、采购/销售订单数据） |
| `cd backend && cargo check` | 阻塞 | Cargo CA 证书错误，未进入代码编译 |
| `cd backend && cargo test` | 阻塞 | 同 Cargo CA 证书错误 |
| `cd backend && cargo check --offline` | 阻塞 | 本机缓存缺 `axum` |
| Rust LSP diagnostics | 阻塞 | `rust-analyzer` 未安装 |

## 5. 核心业务用例覆盖矩阵

| 业务场景 | 覆盖文件/脚本 | 当前状态 | 下一步 |
| --- | --- | --- | --- |
| 商品主数据（Item/SKU）创建 | `backend/tests/inventory_service_test.rs`（代码阶段扩展商品用例）；`seed_data.py` / `seed_data_enhanced.py` | 测试资产存在，语法脚本通过；cargo 阻塞未执行 | 恢复 Cargo 后运行 `cargo test --test inventory_service_test` |
| 仓库入库数量/状态 | `backend/tests/inventory_service_test.rs`，含 auto_approved、pending、approve/reject、生命周期状态 | 测试资产存在；签名阻塞已修复；cargo 阻塞未执行 | 恢复 Cargo 后运行 `cargo test --test inventory_service_test` |
| 仓库出库数量/状态 | `backend/tests/inventory_service_test.rs`，含 outbound、insufficient stock、状态流转 | 测试资产存在；cargo 阻塞未执行 | 同上 |
| 采购订单 | `backend/tests/purchase_sales_service_test.rs` | 覆盖创建、空 items、供应商状态、重复单号、更新等 | 恢复 Cargo 后运行 `cargo test --test purchase_sales_service_test` |
| 销售订单 | `backend/tests/purchase_sales_service_test.rs` | 覆盖销售订单生命周期与 ATP 相关逻辑 | 恢复 Cargo 后运行同一测试文件 |
| 制造质检（Inspection）/不合格品单（NCR） | `backend/tests/manufacturing_service_test.rs` | 覆盖工单质检记录、结果校验（pass/fail）、附件（代码阶段按通用质检口径整理） | 恢复 Cargo 后运行 `cargo test --test manufacturing_service_test` |
| 大量数据测试 | `backend/seed_data.py` + `backend/seed_data_enhanced.py` | 脚本语法通过，包含供应商、客户、库位、商品（Item/SKU）、合同、采购/销售订单等 ERP 数据（代码阶段重写） | 后端可运行后执行脚本并进行 API/页面抽样验证 |
| 前端基础回归 | `frontend/src/**/__tests__` | Vitest 11 条通过；构建通过 | 可继续补 Playwright E2E |

## 6. 后端环境修复建议

当前后端不是代码验证失败，而是依赖解析环境失败。建议按顺序处理：

1. 修复或移除指向坏文件的 CA 配置：检查 `SSL_CERT_FILE`、`CARGO_HTTP_CAINFO`、`CURL_CA_BUNDLE` 是否指向 `/tmp/ca-bundle.crt`。
2. 确保系统 CA 包可用，例如 Debian/Ubuntu 环境安装/刷新 `ca-certificates`。
3. 安装 Rust LSP：`rustup component add rust-analyzer`。
4. 恢复网络依赖后运行：

```bash
cd backend
cargo check
cargo test
cargo test --test inventory_service_test
cargo test --test purchase_sales_service_test
cargo test --test manufacturing_service_test
```

## 7. 大数据测试建议步骤

后端可运行后，建议在隔离数据库执行：

```bash
cd backend
cp .env.example .env
cargo run
python3 seed_data.py
python3 seed_data_enhanced.py
```

然后用前端或 API 验证：

1. 登录 `admin / admin123`。
2. 创建商品（Item），字段包含 SKU、名称、分类、单位、规格、状态；SKU 为唯一业务编码。
3. 新建/选择库位，执行入库，确认商品库存数量正确且库位 `used_count` 增加。
4. 创建采购订单，检查明细数量和总额。
5. 创建销售订单并选择库存商品（SKU），确认 ATP/出库逻辑，审批后状态流转为 `outbound`。
6. 创建工单质检记录（Inspection），分别覆盖 `pass` 和 `fail`，确认非法结果被拒绝。
7. 使用列表筛选/搜索验证 1,000+ 级别数据下分页、筛选和搜索可用。

## 8. 剩余风险

- 后端编译、后端集成测试和 API 联调仍需在 Cargo 环境恢复后完成。
- 当前仓库已有多处工作区改动，不全是本轮直接修改；合并/提交前应按模块拆分 review。
- 前端 `vendor-ui` 生产包约 1.5MB gzip 前 472KB，功能可用但后续可考虑路由懒加载优化首屏。
