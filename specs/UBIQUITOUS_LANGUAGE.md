# Ubiquitous Language — ERP 通用术语表

> 来源：erp-v2 重写（2026-08）。本表为本仓库的**规范术语**，文档、代码、命名必须使用「术语」列词汇，禁止使用「Aliases to avoid」列中的词。
> 适用范围：erp-v2 P0 + P1 + P2 已实现模块。未实现模块（HR、制造、合同、门户、BI、固定资产）暂未纳入本表，待实现后追加。

## 商品与库存 (Catalog & Inventory)

| Term | Definition | Aliases to avoid |
| ---- | ---------- | ---------------- |
| **商品 (Item)** | 可交易的基本业务对象，全系统唯一实体 | 管材、产品、物料、货品 |
| **SKU** | 商品的唯一业务编码，由系统或人工分配 | 管号 |
| **规格 (Spec)** | 商品的描述性属性（可选），自由文本 | 钢级、API 5CT 等级 |
| **库存 (Inventory)** | 商品在库位上的存量记录 | 存货、现货 |
| **仓库 (Warehouse)** | 库存的顶级物理/逻辑容器；`locations.warehouse_id` 指向它 | 库区 |
| **库位 (Location)** | 仓库内的具体存放位置，库存归属的最小单位 | 货架号 |
| **入库 (Inbound)** | 商品进入库存的业务动作 | 收货（仅限采购语境） |
| **出库 (Outbound)** | 商品离开库存的业务动作 | 发货（仅限销售语境） |
| **库存预留 (Reservation)** | 为销售订单预先锁定可用库存（ATP） | 占用、预占 |
| **盘点 (Inventory Check)** | 周期性核对账面与实物库存的过程 | 盘点单、Count Session |
| **库存日志 (Inventory Log)** | 物料移动的不可变事件日志（审计轨迹） | 流水 |
| **ATP 可用量 (Available-To-Promise)** | 当前库存 − 已预留 = 可对客户承诺的数量 | 可售库存 |

## 采购与供应 (Procurement)

| Term | Definition | Aliases to avoid |
| ---- | ---------- | ---------------- |
| **供应商 (Supplier)** | 向本企业供应商品的当事人 | 卖方、供方 |
| **采购订单 (Purchase Order)** | 向供应商发出的正式采购单据 | 订货单、PO |
| **采购收货 (Receipt)** | 供应商到货并入库的确认动作 | 收货单、入库单 |
| **订单号 (Order No)** | 系统生成的唯一单号，格式 `PO{YYYYMMDD}-{rand4hex}`（采购）/ `SO{YYYYMMDD}-{rand4hex}`（销售） | 单号 |

## 销售 (Sales)

| Term | Definition | Aliases to avoid |
| ---- | ---------- | ---------------- |
| **客户 (Customer)** | 从本企业购买商品的当事人 | 买方、顾客、客户单位 |
| **销售订单 (Sales Order)** | 客户下达的正式销售单据 | 订单（无前缀时禁止使用）、SO |
| **发货 (Shipment)** | 销售出库并发往客户的确认动作 | 出库单 |

## 财务 (Finance)

| Term | Definition | Aliases to avoid |
| ---- | ---------- | ---------------- |
| **会计科目 (Account)** | 记账的分类科目（GL 科目） | 账户、科目代码 |
| **日记账 (Journal Entry)** | 一笔业务的分录记录 | 流水、凭证（与凭证文件区分） |
| **发票 (Invoice)** | 财务意义上的开票/收票单据 | 账单 |
| **付款 (Payment)** | 对外支付的资金动作 | 支出 |
| **试算平衡 (Trial Balance)** | 校验借贷平衡的财务报表 | 试算表 |

## 审批流 (Workflow)

| Term | Definition | Aliases to avoid |
| ---- | ---------- | ---------------- |
| **审批流 (Workflow)** | 一组状态 + 转换构成的业务流转定义 | 流程（无前缀时禁止使用） |
| **审批流状态 (Workflow State)** | 流转中的一个节点；映射到业务单据的 `doc_status` 整数 | 节点 |
| **审批流转换 (Workflow Transition)** | 状态间的有向弧；带 `action` 标识触发动作，可选 `amount_threshold` 做条件路由 | 边 |
| **审批流实例 (Workflow Instance)** | 一次具体业务单据的流转执行记录 | 实例（无前缀时禁止使用） |
| **审批任务 (Workflow Task)** | 实例流转中分派给具体用户的待办 | 任务（无前缀时禁止使用） |
| **金额阈值路由 (Amount Threshold Routing)** | 满足 `business_amount ≥ threshold` 的 transition 优先选中；否则 fallback 走无阈值 transition | 条件分支 |

## 鉴权与权限 (Auth & RBAC)

| Term | Definition | Aliases to avoid |
| ---- | ---------- | ---------------- |
| **用户 (User)** | 系统登录身份 | 账号 |
| **角色 (Role)** | 权限集合的命名分组 | 用户组 |
| **权限 (Permission)** | 形如 `item.read` / `order.approve` / `finance.read` 的能力标识；中间件每个请求查库注入 | 权限点 |
| **Refresh Token** | 服务端存储（SHA-256 哈希）的长效刷新凭证；轮换 + 登出撤销 | 刷新令牌 |
| **操作日志 (Operation Log)** | 管理员操作审计记录 | 操作流水 |

## 数据契约 (API Conventions)

| Term | Definition | Aliases to avoid |
| ---- | ---------- | ---------------- |
| **ApiResponse** | 统一响应壳：`{ success, request_id, data }` | 返回值 |
| **PaginatedResponse** | 分页响应：`{ ..., data, meta: { total, page, page_size, total_pages } }` | 列表响应 |
| **软删除 (Soft Delete)** | 通过 `deleted_at` 标记删除，记录永不物理销毁 | 假删、标记删除 |

## Relationships

- **商品** 是 **SKU** 描述的对象；每个 **SKU** 唯一标识一个 **商品**
- **库存** 归属于 **库位**；**库位** 归属于 **仓库**
- **采购订单** 从 **供应商** 采购 **商品**；**采购收货** 增加 **库存**
- **销售订单** 向 **客户** 销售 **商品**；**发货** 减少 **库存** 并释放对应 **库存预留**
- **库存预留** 绑定到 **销售订单**，占用 **ATP 可用量**
- **采购订单** / **销售订单** 均可在 **审批流实例** 中流转；**审批任务** 属于一个 **审批流实例**
- **发票**、**付款**、**日记账** 关联到 **采购订单** / **销售订单**
- **用户** 分配 **角色**；**角色** 授予 **权限**

## Flagged ambiguities

- **「订单」**：曾指采购订单与销售订单两种单据——必须带前缀（采购/销售）使用
- **「收货」vs「入库」**：收货是采购确认动作，入库是库存动作；两者常同时发生但不是同一概念
- **「发货」vs「出库」**：同理，发货是销售确认动作，出库是库存动作
- **「库存」**：通用库存与 ATP 预留是不同概念，预留是库存的分配视图
- **「流程」/「实例」/「任务」**：单独出现时禁止使用——必须带「审批流」前缀
- **「金额」**：业务金额一律 `rust_decimal::Decimal`；禁止 f64 用于业务金额
- **「报表」**：当前只有明细 + 聚合两种（P1 Reports），BI 模块未实现
