# 任务拆分 (Tasks) — ERP v3 重写

> 里程碑见 rewrite-plan §7。每任务验收 = 后端测试相关项 + 前端可真实操作（若涉及页面）。

## M0 设计（本次完成）
- [x] T0.1 rewrite-plan.md / T0.2 PRD / T0.3 detailed-design / T0.4 frontend-design / T0.5 tasks

## M1 后端核心重构（约）✅ 已完成
| # | 任务 | 产出 | 验收 |
|---|---|---|---|
| 1.1 ✅ | 领域状态机拆分 PoStatus/SoStatus + allowed_actions | domain/purchasing.rs sales.rs inventory.rs | 单测覆盖全合法/非法迁移 |
| 1.2 ✅ | Quantity API 序列化改 string | domain/quantity.rs + 相关 DTO | 回归测试全绿 |
| 1.3 ✅ | repo 拆分 inventory→(balance/log/inbound/outbound/stock)、workflow→(definition/instance/task)、finance→(account/journal/invoice/payment)、sales→(order/reservation) | repos/ 子目录 | 文件≤300行规则；测试全绿 |
| 1.4 ✅ | 列表/详情 API 增加 *_name 投影（供应商/客户/商品/库位/科目） | repo JOIN + DTO | 集成断言 name 字段 |
| 1.5 ✅ | 订单 GET 返回 allowed_actions | http handler + 状态机 | 集成断言动作数组正确 |
| 1.6 ✅ | 错误码/响应复核（不泄露 SQL；Decimal 全 string） | error/response | cargo test 140 + vue-tsc + build 全绿 |

## M2 前端核心（可控产品）
| # | 任务 | 产出 | 验收 |
|---|---|---|---|
| 2.1 | types/ 强类型重构 + api/ 封装 + utils/money | src/types src/api src/utils | vue-tsc 通过 |
| 2.2 | EntitySelect + OrderLineEditor + OrderStatusTag + ActionBar | components/domain | 组件单测或页面验证 |
| 2.3 | 采购：列表/表单(带明细)/详情(状态机按钮)/收货 | views/purchase | 手工自验创建→提交→审批→收货 |
| 2.4 | 销售：列表/表单/详情/发货 + ATP 提示 | views/sales | 同上 + ATP 不足提示 |
| 2.5 | 库存: 余额/入库/出库/流水/ATP(下拉选品)/盘点 | views/inventory | 可查可追溯可盘点 |
| 2.6 | 主数据用 MasterDataCrud 重写（商品/供应商/客户/库位） | 组件+页面 | 显示名称、可搜索选择 |

## M3 收尾域
| 3.1 | 财务 5 页（科目/日记账/发票/付款/试算平衡） | views/finance | 借贷平衡校验、试算平衡正确 |
| 3.2 | 审批流 3 页（定义/实例/待办） | views/workflow | 可 approve/reject |
| 3.3 | 报表 4 页 + ECharts + CSV | views/reports | 数据与后端一致 |
| 3.4 | Auth/用户管理/操作日志界面完善 | views | RBAC 生效演示 |

## M4 打磨
| 4.1 | 错误/空态/加载/权限细化 | 全站 | 走查 |
| 4.2 | 构建分块优化(消除 500kB 警告) | vite.config | chunk 大小合理 |
| 4.3 | README/AGENTS 同步 | docs | 与实际一致 |

## DoD 全层（每个页面级任务通用）
1. cargo test 全绿（涉及后端改动时）
2. vue-tsc + build 绿
3. 手工用该页面完成对应业务动作并看到预期数据（名称/状态/金额正确）
