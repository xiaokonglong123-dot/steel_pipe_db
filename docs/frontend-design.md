# 前端设计 (Frontend Design)

> 对应 PRD/detailed-design。核心原则：把“录单 + 审批”做真；消灭 CrudList 万能组件与原 ID 裸显。

## 1. 技术栈
Vue3 + TS + Vite + Element Plus + Pinia + TanStack Query + vue-router。

## 2. 项目结构
```
src/
  api/        client.ts(拦截401/错误归一/request_id)  +  每域一个: auth.ts catalog.ts parties.ts inventory.ts purchase.ts sales.ts finance.ts workflow.ts reports.ts
  types/      每域一个文件，严格类型; common.ts(Page<T>/ApiResponse<T>)
  stores/     auth.ts(含 permissions) 等
  router/     index.ts + guard.ts(登录+权限路由守卫)
  components/
    common/  PageHeader DataTable PaginationBar EmptyState
    domain/  EntitySelect OrderLineEditor MoneyText OrderStatusTag (ActionBar 由 allowed_actions 渲染)
  layouts/    MainLayout(侧边栏+顶栏)
  views/      按域目录
  utils/      money.ts(Decimal string 计算/格式化) format.ts
styles/       全局样式
```

## 3. 类型层（消灭 any-ish）
- `types/common.ts`: `ApiResponse<T>` `Paginated<T>`。
- 金额/数量字段 **全部 `string`**（后端 JSON string）。
- 例：
```ts
interface PurchaseOrderItem { item_id: number; item_name: string; sku: string; quantity: string; unit_price: string; line_total: string }
interface PurchaseOrder {
  id: number; order_no: string; supplier_id: number; supplier_name: string;
  status: PurchaseOrderStatus; doc_status: number; expected_date: string|null;
  total_amount: string; items: PurchaseOrderItem[]; allowed_actions: PoAction[];
}
type PoAction = 'submit'|'approve'|'reject'|'cancel'|'receive';
```

## 4. 组件规范

### 4.1 EntitySelect（新增，核心）
- props: `modelValue:number|null`, `api`（搜索函数）、`placeholder`
- 远程搜索 + 下拉显示 `名称 (code)`；选中回填 id。所有外键（供应商/客户/商品/库位/科目）一律用它。

### 4.2 OrderLineEditor（新增，核心）
- 表格行：商品(EntitySelect) | 数量 | 单价 | 行小计(自动) | 操作(删除)
- 行可增删；底部显示合计金额(Σ 行小计)；保存前校验：至少一行、数量>0、单价≥0。
- 数量/单价输入以 string 保存；行小计用 utils/money 相加。

### 4.3 OrderStatusTag / ActionBar
- 状态→颜色/文案映射集中定义；ActionBar 仅按 `order.allowed_actions` 渲染按钮（无按钮 hardcode）。

### 4.4 MasterDataCrud
取代 CrudList，但**仅**用于纯主数据列表页（商品/供应商/客户/科目/用户/库位），配置化生成列/表单；单据不经过它。

---

## 5. 页面规范（关键页面）

### 5.1 采购订单列表/详情/表单
- 列表列：订单号 / 供应商**名称** / 状态(tag) / 预计日期 / 总金额 / 操作(按行内 allowed_actions 或由详情承担，列表简单提供 查看)
- 详情：头信息(descriptions，外键名称) + 明细表(列同 OrderLineEditor 只读) + allowed_actions 按钮 + 时间线(提交/审批/收货记录可选)
- 表单：供应商 EntitySelect + 预计日期 + OrderLineEditor + 备注；编辑 draft 时可改、其它态只读跳转详情
- 操作结果统一 message；操作后自动刷新详情与列表。

### 5.2 销售订单 同构（ATP 提示：表单内存量校验提示，提交前若服务端报 ATP 失败要提示具体差额）。

### 5.3 库存
- 库存余额页：筛选 + 分页 + 显示“商品名称/SKU/库位名/现有量/预留量/可用量”
- 入库/出库单：创建页 = 库位 + 明细(OrderLineEditor 复用，去掉价格)；列表+详情+“过账”按钮（详情按状态显示）
- 盘点：生成单（选择库位）→ 录入实盘数 → 差异表 → 过账调整
- 库存流水: 按 SKU / ref 过滤，时间倒序
- ATP 查询页: EntitySelect 选商品 + 库位 → 显示可用（不再手输数字 id）

### 5.4 财务
- 科目树 + 增删子科目；日记账录入（多行借贷 + 自动平衡校验提示）；发票/付款列表；试算平衡表格
- 所有金额用 MoneyText 渲染

### 5.5 审批流
- 定义管理（workflows/states/transitions 维护（管理员）、threshold）、实例列表、我的待办（完成任务弹窗）
### 5.6 报表
- 4 报表页（库存汇总/出入库明细/销售趋势/财务摘要）+ 时间范围筛选 + ECharts + CSV 导出按钮

---

## 6. 交互与质量约定
- 所有列表分页/搜索/加载态/空态统一；错误统一 message（取自后端 `code/message`），401 → 跳登录。
- 数字输入：数量/金额经 InputNumber，但表单模型用 string 表达（提交前不做 f64 累计）。
- 权限：路由 guard + 按钮级 v-if（permissionStore.has('order.approve')）+ allowed_actions 双保险。
- request_id 显示在错误提示中，便于排障。

## 7. 非目标
- 不做多语言 UI、不做移动端、不做打印/复杂导出（已有 CSV 即可）。
