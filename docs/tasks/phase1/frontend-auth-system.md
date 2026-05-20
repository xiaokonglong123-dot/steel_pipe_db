# Phase 1 — 前端：系统管理与认证模块 (P0 MVP)

> 基于：`docs/前端设计文档.md` §3, §5, §7, §9

## 任务清单

### 1.1 认证状态管理
- [ ] 实现 `src/stores/authStore.ts`（Zustand）：
  - 状态：user, token, refreshToken, isAuthenticated
  - 操作：setUser, setToken, logout（纯状态操作，不调 API）
  - 从 localStorage 持久化恢复 token
- [ ] 实现 `src/stores/appStore.ts`：siderCollapsed, theme
- [ ] 实现 `features/auth/api/authApi.ts`：
  - `login(credentials)` → POST `/auth/login`
  - `refreshToken(token)` → POST `/auth/refresh`
  - `getMe()` → GET `/auth/me`
- [ ] 实现 `features/auth/hooks/useAuth.ts`：
  - `useLogin()` mutation：调用 API → authStore.setToken → 跳转 dashboard
  - `useLogout()` mutation：authStore.logout → 跳转 /login
  - `useCurrentUser()` query
- [ ] 实现 `features/auth/types.ts`

### 1.2 登录页面
- [ ] 实现 `LoginPage`：
  - 居中登录表单（用户名 + 密码输入框）
  - "登录" 按钮
  - 底部中英文切换 + 版权信息
  - 登录成功后跳转 /dashboard
  - 登录失败显示错误提示
  - 已登录自动跳转 dashboard

### 1.3 路由与布局
- [ ] 实现 `src/routes/routes.ts`：路由配置定义（含 permissions 字段）
- [ ] 实现 `src/routes/ProtectedRoute.tsx`：未登录 → 跳转 /login；无权限 → 403 页面
- [ ] 实现 `src/routes/index.tsx`：路由配置汇总 + 懒加载
- [ ] 实现 `src/App.tsx`：Provider 组装（QueryClientProvider + BrowserRouter + ConfigProvider + I18nextProvider）
- [ ] 实现 `layouts/MainLayout.tsx`：Ant Design Layout（Sider + Header + Content）
- [ ] 实现 `layouts/Sidebar.tsx`：
  - Logo + 系统名称
  - Ant Design Menu（根据角色动态过滤菜单项）
  - 折叠/展开功能
  - 暗色主题（#0F1A2E 深海军蓝）
- [ ] 实现 `layouts/Header.tsx`：
  - 左侧：折叠按钮 + 面包屑导航
  - 右侧：语言切换 + 单位制切换 + 用户头像下拉（个人设置、退出登录）
- [ ] 实现 `layouts/components/Logo.tsx`
- [ ] 实现 `layouts/components/UserDropdown.tsx`
- [ ] 实现 `layouts/components/LanguageSwitcher.tsx`
- [ ] 实现 `layouts/components/UnitSwitch.tsx`

### 1.4 用户管理页面（admin 角色）
- [ ] 实现 `system/pages/UserListPage.tsx`：
  - 用户表格（用户名、显示名、角色、邮箱、最后登录、状态）
  - 新增/编辑/启用/禁用用户
- [ ] 实现 `system/pages/UserFormPage.tsx`：
  - 表单：用户名、显示名、密码、邮箱、角色选择、语言偏好、单位制
- [ ] 实现 `system/api/userApi.ts`
- [ ] 实现 `system/types.ts`
- [ ] 实现 `system/components/RoleTag.tsx`（角色标签，不同颜色）

### 1.5 操作日志页面（admin 角色）
- [ ] 实现 `system/pages/OperationLogPage.tsx`：
  - 日志表格（时间、用户、操作类型、操作对象、摘要、IP）
  - 筛选（日期范围、操作类型、目标类型、用户）
  - 点击详情展开变更 JSON

### 1.6 个人设置页面
- [ ] 实现 `system/pages/ProfilePage.tsx`：
  - 修改密码表单（旧密码 + 新密码 + 确认新密码）
  - 语言偏好选择
  - 单位制选择

### 1.7 403 / 404 页面
- [ ] 实现 403 Forbidden 页面
- [ ] 实现 404 Not Found 页面

### 1.8 国际化
- [ ] 创建 `src/i18n/resources/zh/system.json` 和 `en/system.json`

> **依赖**: 基础设施（Axios 拦截器、i18n）
