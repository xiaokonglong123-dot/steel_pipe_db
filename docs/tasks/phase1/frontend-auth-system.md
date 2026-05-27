# Phase 1 — Frontend: Auth & System Management (P0 MVP)

> Based on: `docs/frontend-design.en.md` §3, §5, §7, §9

## Tasks

### 1.1 Auth State Management
- [ ] Implement `src/stores/authStore.ts` (Zustand):
  - State: user, token, refreshToken, isAuthenticated
  - Actions: setUser, setToken, logout (pure state ops, no API calls)
  - Persist/restore token from localStorage
- [ ] Implement `src/stores/appStore.ts`: siderCollapsed, theme
- [ ] Implement `features/auth/api/authApi.ts`:
  - `login(credentials)` → POST `/auth/login`
  - `refreshToken(token)` → POST `/auth/refresh`
  - `getMe()` → GET `/auth/me`
- [ ] Implement `features/auth/hooks/useAuth.ts`:
  - `useLogin()` mutation: call API → authStore.setToken → redirect to dashboard
  - `useLogout()` mutation: authStore.logout → redirect to /login
  - `useCurrentUser()` query
- [ ] Implement `features/auth/types.ts`

### 1.2 Login Page
- [ ] Implement `LoginPage`:
  - Centered login form (username + password inputs)
  - "Login" button
  - Language switcher (zh/en) + copyright info at bottom
  - Redirect to /dashboard on success
  - Show error toast on failure
  - Auto-redirect if already logged in

### 1.3 Routing & Layout
- [ ] Implement `src/routes/routes.ts`: route config definitions (with permissions field)
- [ ] Implement `src/routes/ProtectedRoute.tsx`: unauthenticated → redirect to /login; no permission → 403 page
- [ ] Implement `src/routes/index.tsx`: route config aggregation + lazy loading
- [ ] Implement `src/App.tsx`: Provider assembly (QueryClientProvider + BrowserRouter + ConfigProvider + I18nextProvider)
- [ ] Implement `layouts/MainLayout.tsx`: Ant Design Layout (Sider + Header + Content)
- [ ] Implement `layouts/Sidebar.tsx`:
  - Logo + system name
  - Ant Design Menu (dynamically filter items by role)
  - Collapse/expand toggle
  - Dark theme (#0F1A2E deep navy)
- [ ] Implement `layouts/Header.tsx`:
  - Left: collapse button + breadcrumb nav
  - Right: language switch + unit switch + user avatar dropdown (profile settings, logout)
- [ ] Implement `layouts/components/Logo.tsx`
- [ ] Implement `layouts/components/UserDropdown.tsx`
- [ ] Implement `layouts/components/LanguageSwitcher.tsx`
- [ ] Implement `layouts/components/UnitSwitch.tsx`

### 1.4 User Management Page (admin role)
- [ ] Implement `system/pages/UserListPage.tsx`:
  - User table (username, display_name, role, email, last_login, status)
  - Add/edit/enable/disable user
- [ ] Implement `system/pages/UserFormPage.tsx`:
  - Form: username, display_name, password, email, role select, language pref, unit system
- [ ] Implement `system/api/userApi.ts`
- [ ] Implement `system/types.ts`
- [ ] Implement `system/components/RoleTag.tsx` (colored role badges)

### 1.5 Operation Log Page (admin role)
- [ ] Implement `system/pages/OperationLogPage.tsx`:
  - Log table (time, user, action type, target, summary, IP)
  - Filters (date range, action type, target type, user)
  - Click to expand change JSON

### 1.6 Profile Settings Page
- [ ] Implement `system/pages/ProfilePage.tsx`:
  - Change password form (old password + new password + confirm)
  - Language preference
  - Unit system preference

### 1.7 403 / 404 Pages
- [ ] Implement 403 Forbidden page
- [ ] Implement 404 Not Found page

### 1.8 i18n
- [ ] Create `src/i18n/resources/zh/system.json` and `en/system.json`

> **Deps**: Infrastructure (Axios interceptor, i18n)
