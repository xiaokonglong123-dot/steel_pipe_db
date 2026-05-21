# `frontend/src/` — 应用结构与共享基础设施

## 入口点

### `main.tsx`
```tsx
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { BrowserRouter } from 'react-router-dom'

// 初始化 i18n
// 创建 QueryClient
// 渲染：<QueryClientProvider> → <BrowserRouter> → <App />
```
- 初始化 i18next（检测语言、加载资源）
- 创建 QueryClient，设置默认 staleTime
- 将 React 应用渲染到 `#root`

### `App.tsx`
```tsx
function App() {
  return (
    <ConfigProvider theme={theme}>
      <AppLayout>
        <AuthGuard>
          <AppRoutes />
        </AuthGuard>
      </AppLayout>
    </ConfigProvider>
  )
}
```
- 使用自定义主题的 Ant Design `ConfigProvider` 包裹应用
- `AppLayout` — 侧边栏 + 顶部栏 + 内容区
- `AuthGuard` — 检查 JWT token，过期时重定向到登录页
- `AppRoutes` — 渲染 `routes/` 中匹配的路由

## 共享基础设施

### `api/` — Axios 实例
```ts
const api = axios.create({ baseURL: '/api/v1' })
// 拦截器：从 localStorage 附加 JWT token
// 拦截器：处理 401 → 重定向到登录
```
- 一个 axios 实例用于所有 API 调用
- 自动附加 `Authorization: Bearer <token>` 头部
- 401 时自动重定向

### `components/` — 共享组件
- `AppLayout.tsx` — 侧边栏 + 顶部栏 + 内容区外壳
- `Sidebar.tsx` — 导航菜单
- `PrivateRoute.tsx` — 认证守卫包装器
- `Loading.tsx` — 加载动画
- `ErrorBoundary.tsx` — 错误回退界面

### `lib/` — 运行时验证
- `validateResponse.ts` — 封装 Zod 模式用于 API 响应验证
- 使用 `zod.response()` 模式：在运行时验证 API 响应
- 被特性 API 模块导入，用于类型安全的数据获取

### `hooks/` — 共享 Hooks
- `useAuth.ts` — 认证上下文（登录/登出/当前用户）
- `usePagination.ts` — 分页状态管理

### `i18n/` — 翻译
```
i18n/
├── index.ts        ← i18next 初始化
├── zh/             ← 中文翻译
│   ├── common.json
│   ├── pipes.json
│   ├── inventory.json
│   └── ...
└── en/             ← 英文翻译（相同结构）
```
- 按特性划分命名空间：`'common'`、`'pipes'`、`'inventory'` 等
- 组件中使用 `useTranslation('feature_name')`

### `routes/` — 路由配置（react-router-dom v7）
```
/login                     ← 公开
/                          ← ProtectedRoute → MainLayout → Outlet
  /pipes/seamless          ← SeamlessPipeListPage
  /pipes/seamless/new      ← SeamlessPipeFormPage
  /pipes/seamless/:id      ← SeamlessPipeDetailPage
  /pipes/seamless/:id/edit ← SeamlessPipeFormPage
  /pipes/screen/*          ← 相同模式
  /inventory/inbound       ← InboundListPage
  /inventory/outbound      ← OutboundListPage
  /inventory/stock         ← StockQueryPage
  /inventory/locations     ← LocationListPage
  /inventory/check         ← InventoryCheckListPage
  /suppliers               ← SupplierListPage (+ /new, /:id/edit)
  /customers               ← CustomerListPage (+ /new, /:id/edit)
  /purchases               ← (+ /new, /:id, /:id/edit)
  /sales                   ← (+ /new, /:id, /:id/edit)
  /quality/certs           ← (+ /new, /:id, /:id/edit)
  /contracts               ← (+ /new, /:id, /:id/edit)
  /reports                 ← ReportListPage
  /reports/dashboard       ← DashboardPage
  /labels                  ← LabelPrintPage
```
- 使用 `createBrowserRouter`（非扁平路由数组）
- `ProtectedRoute` 包装器在渲染 `MainLayout` 前检查认证
- 嵌套布局使用 `Outlet` 模式
- 当前未使用懒加载（所有页面即时加载）

### `theme/` — Ant Design 主题
```ts
const theme: ThemeConfig = {
  token: {
    colorPrimary: '#1677ff',
    borderRadius: 6,
    // Ant Design 5 主题令牌
  }
}
```
- 一致的品牌配色和间距
- 在 `vite.config.ts` 中通过 Less 变量覆盖 Ant Design CSS

### `zod-schemas/` — Zod 验证模式
```
zod-schemas/
├── core.ts        ← 通用类型（PaginatedResponse、ApiResponse 包装器）
├── orders.ts      ← 采购/销售订单模式
├── inventory.ts   ← 库存、入库、出库模式
├── quality.ts     ← 质量证书模式
├── reports.ts     ← 报表参数模式
├── labels.ts      ← 标签数据模式
```
- 每个模式文件导出用于 API 请求/响应验证的 Zod 类型
- 被 `lib/validateResponse.ts` 用于运行时 API 响应检查
- 以运行时验证补充 TypeScript 静态类型

### `utils/` — 工具函数
- `formatters.ts` — 日期、货币、十进制格式化
- `validators.ts` — 遗留表单验证辅助（可能引用 zod-schemas/）
- 主要验证模式位于 `zod-schemas/`
- `constants.ts` — API 端点路径、状态枚举

## 如何添加新特性页面
1. 在 `src/features/{feature}/` 中创建特性文件（参见 features/AGENTS_zh.md）
2. 在 `src/routes/index.tsx` 中添加路由
3. 在 `src/i18n/zh/{feature}.json` 和 `src/i18n/en/{feature}.json` 中添加 i18n 命名空间
4. 从 `src/api/` 导入 api 实例用于数据获取
