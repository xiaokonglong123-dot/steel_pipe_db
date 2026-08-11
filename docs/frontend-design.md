# ERP v2 — 前端设计文档

> **版本**: v2.0-alpha
> **日期**: 2026-08-09
> **依赖**: [PRD.md](./PRD.md) — 模块定义、用户角色、FR
> **技术栈**: Vue 3 + TypeScript + Vite + Element Plus + Pinia + Vue Router + @tanstack/vue-query

---

## 1. 顶层架构

```
用户浏览器
  → Vue Router (路由守卫 + RBAC)
  → AppLayout.vue (侧边栏 + 顶栏 + Outlet)
  → 各 feature 页面 (ListPage / FormPage / DetailPage)
  → @tanstack/vue-query (服务端状态：缓存/去重/失效 + mutation)
  → api/client.ts (fetch 封装, Bearer token, 30s timeout, 401 → logout)
```

**语言**：v2.0 仅 zh-CN。前端保留轻量 `src/locales/zh-CN/` 目录作为 i18n 框架入口（不引入 vue-i18n），后续加语言只需补 locale 文件 + 切换 hook。

---

## 2. 项目结构

```
frontend/
├── index.html
├── vite.config.ts
├── tsconfig.json
├── package.json
├── src/
│   ├── main.ts                    # createApp + use Router + use Pinia + use VueQuery
│   ├── App.vue                    # <RouterView /> 根组件
│   ├── api/
│   │   ├── client.ts              # fetch 封装 (Bearer token, 401 → logout, 30s timeout)
│   │   └── queryClient.ts         # VueQuery 全局配置 (staleTime: 2min, gcTime: 5min, retry: 1)
│   ├── router/
│   │   └── index.ts               # createRouter + routes 定义 + beforeEach RBAC 守卫
│   ├── stores/
│   │   ├── authStore.ts           # Pinia: auth_token, auth_user, login/logout/refresh
│   │   └── appStore.ts            # Pinia: sidebar collapsed, breadcrumbs, locale
│   ├── layouts/
│   │   └── AppLayout.vue          # 侧边栏 (el-menu) + 顶栏 (用户信息/退出) + <RouterView />
│   ├── features/                  # 按模块划分（与后端 8 模块对齐）
│   │   ├── auth/
│   │   │   ├── LoginPage.vue
│   │   │   ├── UserListPage.vue
│   │   │   ├── UserFormPage.vue
│   │   │   ├── OperationLogPage.vue
│   │   │   ├── api.ts             # VueQuery hooks: useLogin(), useUsers(), ...
│   │   │   └── queryKeys.ts       # 查询 key 工厂
│   │   ├── catalog/
│   │   │   ├── ItemListPage.vue
│   │   │   ├── ItemFormPage.vue
│   │   │   ├── ItemDetailPage.vue
│   │   │   ├── api.ts
│   │   │   └── queryKeys.ts
│   │   ├── parties/
│   │   │   ├── SupplierListPage.vue
│   │   │   ├── SupplierFormPage.vue
│   │   │   ├── CustomerListPage.vue
│   │   │   ├── CustomerFormPage.vue
│   │   │   ├── api.ts
│   │   │   └── queryKeys.ts
│   │   ├── inventory/
│   │   │   ├── StockQueryPage.vue       # 库存余额查询
│   │   │   ├── InboundListPage.vue
│   │   │   ├── InboundFormPage.vue
│   │   │   ├── OutboundListPage.vue
│   │   │   ├── OutboundFormPage.vue
│   │   │   ├── LocationListPage.vue
│   │   │   ├── CheckListPage.vue
│   │   │   ├── CheckFormPage.vue
│   │   │   ├── InventoryLogsPage.vue    # 库存流水追溯
│   │   │   ├── api.ts
│   │   │   └── queryKeys.ts
│   │   ├── purchasing/
│   │   │   ├── PurchaseOrderListPage.vue
│   │   │   ├── PurchaseOrderFormPage.vue
│   │   │   ├── PurchaseOrderDetailPage.vue
│   │   │   ├── api.ts
│   │   │   └── queryKeys.ts
│   │   ├── sales/
│   │   │   ├── SalesOrderListPage.vue
│   │   │   ├── SalesOrderFormPage.vue
│   │   │   ├── SalesOrderDetailPage.vue
│   │   │   ├── api.ts
│   │   │   └── queryKeys.ts
│   │   ├── finance/
│   │   │   ├── AccountListPage.vue
│   │   │   ├── JournalEntryListPage.vue
│   │   │   ├── JournalEntryFormPage.vue
│   │   │   ├── InvoiceListPage.vue
│   │   │   ├── PaymentListPage.vue
│   │   │   ├── TrialBalancePage.vue
│   │   │   ├── api.ts
│   │   │   └── queryKeys.ts
│   │   ├── workflow/
│   │   │   ├── TaskListPage.vue         # 我的待办
│   │   │   ├── WorkflowListPage.vue     # 审批流定义列表（admin）
│   │   │   ├── api.ts
│   │   │   └── queryKeys.ts
│   │   └── reports/
│   │       ├── InventorySummaryPage.vue
│   │       ├── InboundOutboundPage.vue
│   │       ├── SalesTrendPage.vue
│   │       ├── FinanceSummaryPage.vue
│   │       ├── api.ts
│   │       └── queryKeys.ts
│   ├── shared/
│   │   ├── components/
│   │   │   ├── SearchBar.vue            # 通用搜索栏
│   │   │   ├── DataTable.vue            # 通用表格（排序/分页/导出链接）
│   │   │   ├── ItemPicker.vue           # 商品搜索选择器
│   │   │   ├── PartyPicker.vue          # 供应商/客户选择器
│   │   │   ├── StatusTag.vue            # 状态标签（颜色+图标）
│   │   │   ├── ConfirmDialog.vue        # 通用确认框
│   │   │   ├── PageHeader.vue           # 页面标题 + 面包屑 + 操作按钮
│   │   │   └── EmptyState.vue           # 空数据占位
│   │   ├── hooks/
│   │   │   ├── useApprove.ts            # 审批动作 hook（approve/reject）
│   │   │   ├── usePost.ts               # 过账动作 hook（inbound/outbound/check）
│   │   │   └── usePagination.ts         # 分页参数 + 查询联动
│   │   └── utils/
│   │       └── format.ts                # 金额格式化、日期格式化
│   └── styles/
│       ├── element-plus-custom.scss      # Element Plus 主题覆盖（中文字体、间距）
│       └── global.scss
```

---

## 3. 路由设计

```typescript
// src/router/index.ts
const routes = [
  { path: '/login', component: LoginPage, meta: { public: true } },

  { path: '/', component: AppLayout, meta: { requiresAuth: true }, redirect: '/inventory/stock',
    children: [
      // 商品主数据
      { path: '/catalog',          component: ItemListPage,   meta: { permission: 'item.read' } },
      { path: '/catalog/new',      component: ItemFormPage,   meta: { permission: 'item.write' } },
      { path: '/catalog/:id',      component: ItemDetailPage, meta: { permission: 'item.read' } },
      { path: '/catalog/:id/edit', component: ItemFormPage,   meta: { permission: 'item.write' } },

      // 往来单位
      { path: '/parties/suppliers',          component: SupplierListPage },
      { path: '/parties/suppliers/new',      component: SupplierFormPage },
      { path: '/parties/suppliers/:id/edit', component: SupplierFormPage },
      { path: '/parties/customers',          component: CustomerListPage },
      { path: '/parties/customers/new',      component: CustomerFormPage },
      { path: '/parties/customers/:id/edit', component: CustomerFormPage },

      // 库存
      { path: '/inventory/stock',           component: StockQueryPage },
      { path: '/inventory/locations',       component: LocationListPage },
      { path: '/inventory/inbound',         component: InboundListPage },
      { path: '/inventory/inbound/new',     component: InboundFormPage },
      { path: '/inventory/outbound',        component: OutboundListPage },
      { path: '/inventory/outbound/new',    component: OutboundFormPage },
      { path: '/inventory/check',           component: CheckListPage },
      { path: '/inventory/check/new',       component: CheckFormPage },
      { path: '/inventory/logs',            component: InventoryLogsPage },

      // 采购
      { path: '/purchases',          component: PurchaseOrderListPage },
      { path: '/purchases/new',      component: PurchaseOrderFormPage },
      { path: '/purchases/:id',      component: PurchaseOrderDetailPage },

      // 销售
      { path: '/sales',              component: SalesOrderListPage },
      { path: '/sales/new',          component: SalesOrderFormPage },
      { path: '/sales/:id',          component: SalesOrderDetailPage },

      // 财务
      { path: '/finance/accounts',       component: AccountListPage },
      { path: '/finance/journal',        component: JournalEntryListPage },
      { path: '/finance/journal/new',    component: JournalEntryFormPage },
      { path: '/finance/invoices',       component: InvoiceListPage },
      { path: '/finance/payments',       component: PaymentListPage },
      { path: '/finance/trial-balance',  component: TrialBalancePage },

      // 审批
      { path: '/workflow/tasks',     component: TaskListPage },
      { path: '/workflow/definitions', component: WorkflowListPage },

      // 报表
      { path: '/reports/inventory',       component: InventorySummaryPage },
      { path: '/reports/inbound-outbound', component: InboundOutboundPage },
      { path: '/reports/sales-trend',     component: SalesTrendPage },
      { path: '/reports/finance',         component: FinanceSummaryPage },

      // 系统管理
      { path: '/admin/users',        component: UserListPage,   meta: { permission: 'user.manage' } },
      { path: '/admin/users/new',    component: UserFormPage,   meta: { permission: 'user.manage' } },
      { path: '/admin/users/:id/edit', component: UserFormPage, meta: { permission: 'user.manage' } },
      { path: '/admin/operation-logs', component: OperationLogPage, meta: { permission: 'user.manage' } },
    ]
  },
];

const router = createRouter({ history: createWebHistory(), routes });

// RBAC 守卫
router.beforeEach(async (to, from, next) => {
    if (to.meta.public) return next();
    const auth = useAuthStore();
    if (!auth.token) return next('/login');
    if (to.meta.permission && !auth.hasPermission(to.meta.permission as string)) return next('/');
    next();
});
```

---

## 4. 状态管理

### 4.1 Pinia — 客户端状态

```typescript
// stores/authStore.ts
export const useAuthStore = defineStore('auth', () => {
    const token = ref(localStorage.getItem('auth_token') || '');
    const user = ref<AuthUser | null>(null);
    const permissions = ref<string[]>([]);

    async function login(username: string, password: string) { /* POST /auth/login */ }
    async function logout() { /* POST /auth/logout → clear token → router.push('/login') */ }
    async function refresh() { /* POST /auth/refresh → rotate token */ }
    function hasPermission(perm: string): boolean { return permissions.value.includes(perm); }

    return { token, user, permissions, login, logout, refresh, hasPermission };
}, { persist: { storage: localStorage, paths: ['token', 'user'] } });

// stores/appStore.ts
export const useAppStore = defineStore('app', () => {
    const sidebarCollapsed = ref(false);
    const breadcrumbs = ref<Breadcrumb[]>([]);
    return { sidebarCollapsed, breadcrumbs };
});
```

### 4.2 @tanstack/vue-query — 服务端状态

```typescript
// api/queryClient.ts
export const queryClient = new QueryClient({
    defaultOptions: {
        queries: { staleTime: 2 * 60 * 1000, gcTime: 5 * 60 * 1000, retry: 1, refetchOnWindowFocus: false },
        mutations: { onError: (err) => ElMessage.error(err.message) },
    },
});
```

**使用模式**（每个 feature 独立封装）：

```typescript
// features/catalog/api.ts
export function useItems(params: Ref<ItemFilter>) {
    return useQuery({
        queryKey: itemKeys.list(params.value),
        queryFn: () => apiClient.get<PaginatedResponse<Item>>('/items', params.value),
        placeholderData: keepPreviousData,
    });
}

export function useCreateItem() {
    const qc = useQueryClient();
    return useMutation({
        mutationFn: (dto: CreateItemRequest) => apiClient.post('/items', dto),
        onSuccess: () => qc.invalidateQueries({ queryKey: itemKeys.lists() }),
    });
}
```

### 4.3 queryKeys 工厂模式

每个 feature 实现 `queryKeys.ts`（继承 v1 已验证的 key 工厂模式，杜绝字符串散落）：

```typescript
// features/catalog/queryKeys.ts
export const itemKeys = {
    all:    ['items'] as const,
    lists:  ()       => [...itemKeys.all, 'list'] as const,
    list:   (f: ItemFilter) => [...itemKeys.lists(), f] as const,
    details: ()      => [...itemKeys.all, 'detail'] as const,
    detail: (id: number) => [...itemKeys.details(), id] as const,
};
```

---

## 5. API 客户端

```typescript
// api/client.ts
const BASE_URL = '/api/v1';

async function request<T>(method: string, path: string, body?: unknown, params?: Record<string, unknown>): Promise<T> {
    const auth = useAuthStore();
    const url = new URL(path, window.location.origin);
    if (params) Object.entries(params).forEach(([k, v]) => { if (v !== undefined && v !== '') url.searchParams.set(k, String(v)); });

    const res = await fetch(url, {
        method,
        headers: {
            'Content-Type': 'application/json',
            ...(auth.token ? { 'Authorization': `Bearer ${auth.token}` } : {}),
        },
        body: body ? JSON.stringify(body) : undefined,
        signal: AbortSignal.timeout(30_000),
    });

    if (res.status === 401) { auth.logout(); throw new Error('未登录或 token 过期'); }
    if (!res.ok) {
        const err = await res.json().catch(() => ({ message: res.statusText }));
        throw new ApiError(err.code, err.message, res.status);
    }
    if (res.status === 204) return undefined as T;  // No Content
    return res.json();
}

export const apiClient = {
    get:    <T>(path: string, params?: Record<string, unknown>) => request<T>('GET', path, undefined, params),
    post:   <T>(path: string, body?: unknown) => request<T>('POST', path, body),
    put:    <T>(path: string, body?: unknown) => request<T>('PUT', path, body),
    delete: <T>(path: string) => request<T>('DELETE', path),
};
```

---

## 6. 页面设计模式

### 6.1 ListPage 模板

```vue
<!-- features/catalog/ItemListPage.vue -->
<template>
  <div class="list-page">
    <PageHeader title="商品管理">
      <template #actions>
        <el-button type="primary" @click="router.push('/catalog/new')">新商品</el-button>
      </template>
    </PageHeader>

    <!-- 搜索栏 -->
    <SearchBar :fields="searchFields" v-model="filter" @search="handleSearch">
      <el-select v-model="filter.category" placeholder="分类" clearable>...</el-select>
    </SearchBar>

    <!-- 数据表格 -->
    <DataTable
      :columns="columns"
      :data="query.data?.items ?? []"
      :loading="query.isLoading.value"
      :pagination="{ current: page, pageSize: pageSize, total: query.data?.meta?.total ?? 0 }"
      @page-change="handlePageChange"
    >
      <template #actions="{ row }">
        <el-button link @click="router.push(`/catalog/${row.id}`)">详情</el-button>
        <el-button link type="primary" @click="router.push(`/catalog/${row.id}/edit`)">编辑</el-button>
        <el-button link type="danger" @click="handleDelete(row)">删除</el-button>
      </template>
    </DataTable>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useItems, useDeleteItem } from './api';
import { itemKeys } from './queryKeys';
import { useRouter } from 'vue-router';
const router = useRouter();
const filter = ref<ItemFilter>({ sku: '', name: '', category: '', status: '' });
const page = ref(1);
const pageSize = ref(20);

const query = useItems(computed(() => ({ ...filter.value, page: page.value, page_size: pageSize.value })));
const deleteMutation = useDeleteItem();
async function handleDelete(row: Item) {
    await deleteMutation.mutateAsync(row.id);
}
function handleSearch() { page.value = 1; }
function handlePageChange(p: number, ps: number) { page.value = p; pageSize.value = ps; }
</script>
```

### 6.2 FormPage 模板

```vue
<!-- features/catalog/ItemFormPage.vue -->
<template>
  <div class="form-page">
    <PageHeader :title="isEdit ? '编辑商品' : '新商品'" show-back />

    <el-form ref="formRef" :model="form" :rules="rules" label-width="100px" style="max-width: 600px">
      <el-form-item label="SKU" prop="sku">
        <el-input v-model="form.sku" :disabled="isEdit" />
      </el-form-item>
      <el-form-item label="名称" prop="name">
        <el-input v-model="form.name" />
      </el-form-item>
      <el-form-item label="分类">
        <el-select v-model="form.category" placeholder="选择分类" clearable>
          <el-option label="原材料" value="原材料" />
          <el-option label="半成品" value="半成品" />
          <el-option label="成品" value="成品" />
          <el-option label="备品备件" value="备品备件" />
        </el-select>
      </el-form-item>
      <el-form-item label="单位">
        <el-input v-model="form.unit" placeholder="kg, m, pc, 件…" />
      </el-form-item>
      <el-form-item label="规格">
        <el-input v-model="form.spec" type="textarea" />
      </el-form-item>
      <el-form-item>
        <el-button type="primary" :loading="mutation.isLoading.value" @click="handleSubmit">保存</el-button>
        <el-button @click="router.back()">取消</el-button>
      </el-form-item>
    </el-form>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useItem, useCreateItem, useUpdateItem } from './api';

const route = useRoute();
const router = useRouter();
const isEdit = computed(() => !!route.params.id);
const itemId = computed(() => Number(route.params.id));

const form = reactive({ sku: '', name: '', category: '', unit: '', spec: '' });
const rules = { sku: [{ required: true }], name: [{ required: true }] };

// 编辑态加载已有数据
if (isEdit.value) {
    const { data: item } = useItem(itemId);
    watch(item, (val) => { if (val) Object.assign(form, val); }, { immediate: true });
}

const createMutation = useCreateItem();
const updateMutation = useUpdateItem();
const mutation = computed(() => isEdit.value ? updateMutation : createMutation);

async function handleSubmit() {
    await formRef.value.validate();
    const dto = { ...form };
    if (isEdit.value) await updateMutation.mutateAsync({ id: itemId.value, ...dto });
    else await createMutation.mutateAsync(dto);
    router.push('/catalog');
}
</script>
```

### 6.3 DetailPage 模板（采购/销售订单详情）

```vue
<!-- features/purchasing/PurchaseOrderDetailPage.vue -->
<template>
  <div class="detail-page">
    <PageHeader :title="`采购订单 ${order?.order_no}`" show-back>
      <template #actions>
        <!-- 状态驱动的动作按钮 -->
        <template v-if="order?.status === 'draft'">
          <el-button type="primary" @click="handleSubmit">提交审批</el-button>
          <el-button @click="router.push(`/purchases/${order.id}/edit`)">编辑</el-button>
        </template>
        <el-button v-if="order?.status === 'submitted'" type="success" @click="handleApprove('approve')">审批通过</el-button>
        <el-button v-if="order?.status === 'submitted'" type="danger" @click="handleApprove('reject')">驳回</el-button>
        <el-button v-if="canReceive" type="primary" @click="showReceiveDialog = true">收货</el-button>
      </template>
    </PageHeader>

    <!-- 订单头信息 -->
    <el-descriptions :column="2" border>
      <el-descriptions-item label="供应商">{{ order?.supplier_name }}</el-descriptions-item>
      <el-descriptions-item label="订单日期">{{ order?.order_date }}</el-descriptions-item>
      <el-descriptions-item label="状态"><StatusTag :status="order?.status" /></el-descriptions-item>
      <el-descriptions-item label="总金额">{{ formatAmount(order?.total_amount) }}</el-descriptions-item>
    </el-descriptions>

    <!-- 订单明细 -->
    <el-table :data="order?.items ?? []" style="margin-top: 16px">
      <el-table-column prop="sku" label="SKU" />
      <el-table-column prop="item_name" label="商品名称" />
      <el-table-column prop="quantity" label="数量" />
      <el-table-column prop="received_qty" label="已收货" />
      <el-table-column prop="unit_price" label="单价" :formatter="(r: any) => formatAmount(r.unit_price)" />
      <el-table-column prop="total_price" label="小计" :formatter="(r: any) => formatAmount(r.total_price)" />
    </el-table>

    <!-- 收货弹窗 -->
    <el-dialog v-model="showReceiveDialog" title="采购收货">
      <el-form :model="{ items: receiveItems }">
        <el-table :data="receiveItems">
          <el-table-column label="商品">
            <template #default="{ row }">{{ row.item_name }}</template>
          </el-table-column>
          <el-table-column label="订单数量">{{ row.quantity }}</el-table-column>
          <el-table-column label="已收">{{ row.received_qty }}</el-table-column>
          <el-table-column label="本次收货">
            <template #default="{ row }">
              <el-input-number v-model="row.receive_qty" :min="0" :max="row.quantity - row.received_qty" />
            </template>
          </el-table-column>
        </el-table>
      </el-form>
      <template #footer>
        <el-button @click="showReceiveDialog = false">取消</el-button>
        <el-button type="primary" :loading="receiveMutation.isLoading.value" @click="handleReceive">确认收货</el-button>
      </template>
    </el-dialog>
  </div>
</template>
```

---

## 7. 组件设计

### 7.1 共享组件规范

| 组件 | 职责 | Props | Slots |
|------|------|-------|-------|
| `SearchBar` | 表单式筛选栏：fields 配置 → el-form inline，提供搜索/重置按钮 | `fields: SearchField[]`, `modelValue: any` | `#extras`（额外筛选项） |
| `DataTable` | 通用表格：columns 配置 + 加载态 + 空状态 + 分页 + 操作列 | `columns: TableColumn[]`, `data`, `loading`, `pagination` | `#actions`（每行操作） |
| `ItemPicker` | 商品搜索选择器（el-select + 远程搜索，复用 `GET /items?name=…&sku=…` 分页端点作 typeahead） | `modelValue: number`, `disabled?: boolean` | — |
| `PartyPicker` | 供应商/客户选择器（el-select + type prop） | `modelValue: number`, `type: 'supplier'\|'customer'` | — |
| `StatusTag` | 颜色标签（状态→映射 el-tag type） | `status: string`, `statusMap?: Record<string, string>` | — |
| `PageHeader` | 页面标题 + 返回按钮 + 面包屑 | `title: string`, `showBack?: boolean` | `#actions` |
| `EmptyState` | 空数据占位 | `description?: string`, `icon?: string` | — |
| `ConfirmDialog` | 通用确认框 | `visible`, `title`, `message`, `okText?`, `cancelText?` | — |

### 7.2 订单工作流组件

订单详情页内的审批/状态操作统一用共享 hooks：

```typescript
// shared/hooks/useApprove.ts
export function useApprove(orderType: 'purchase' | 'sales') {
    const qc = useQueryClient();
    const approveMt = useMutation({
        mutationFn: ({ id, action }: { id: number; action: 'approve' | 'reject'; comment?: string }) =>
            apiClient.post(`/${orderType === 'purchase' ? 'purchase-orders' : 'sales-orders'}/${id}/${action}`),
        onSuccess: (_, { id }) => {
            qc.invalidateQueries({ queryKey: [orderType === 'purchase' ? 'purchase-orders' : 'sales-orders', 'detail', id] });
            ElMessage.success(action === 'approve' ? '审批通过' : '已驳回');
        },
    });
    return { approveMt, approve: (id: number) => approveMt.mutateAsync({ id, action: 'approve' }), reject: (id: number, comment: string) => approveMt.mutateAsync({ id, action: 'reject', comment }) };
}
```

---

## 8. Element Plus 主题

```scss
// src/styles/element-plus-custom.scss
:root {
    --el-font-family: 'Microsoft YaHei', 'PingFang SC', sans-serif;
    --el-font-size-base: 14px;
    --el-border-radius-base: 4px;

    // 侧边栏
    --el-menu-bg-color: #1f2937;
    --el-menu-text-color: #e5e7eb;
    --el-menu-hover-bg-color: #374151;
    --el-menu-active-color: #60a5fa;
}
```

侧边栏主色调：深灰 `#1f2937`（Tailwind gray-800），蓝高亮，与 ERP 运营工具定位一致。

---

## 9. 构建配置

```typescript
// vite.config.ts (简化)
export default defineConfig({
    plugins: [vue()],
    resolve: { alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) } },
    server: {
        port: 5173,
        proxy: { '/api': { target: 'http://localhost:3000', changeOrigin: true } },
    },
});
```

**前端运行**：

```bash
cd frontend
bun install               # ✅ 使用 bun（node/npm 不可用）
bunx tsc --noEmit         # 类型检查
bun run build             # Vite 构建
bun run dev               # 开发服务器 :5173
```

> **环境记注**：当前环境用 `bun` 替代 `npm`（npm 未安装），脚本用 `bun run`。

---

## 10. 实现顺序（对齐详细设计 P0/P1/P2）

| Phase | 前端范围 | 页面数 |
|-------|---------|--------|
| **P0 (MVP)** | LoginPage, ItemListPage, ItemFormPage, ItemDetailPage, SupplierListPage, CustomerListPage, StockQueryPage, InboundListPage/FormPage, OutboundListPage/FormPage, PurchaseOrderListPage/FormPage/DetailPage, SalesOrderListPage/FormPage/DetailPage, TaskListPage, LocationListPage, UserListPage | ~20 页 |
| **P1 (Finance)** | AccountListPage, JournalEntryListPage/FormPage, InvoiceListPage, PaymentListPage, TrialBalancePage, CheckListPage/FormPage, InventoryLogsPage | ~10 页 |
| **P2 (Polish)** | SalesTrendPage, FinanceSummaryPage, InventorySummaryPage, InboundOutboundPage, OperationLogPage, WorkflowListPage | ~8 页 |

---

> **下一步**：`tasks.md`（P0/P1/P2 逐任务拆分，含验证方式与里程碑）。
