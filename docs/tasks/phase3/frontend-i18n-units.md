# Phase 3 — Frontend: Internationalization & Unit Switching (P2)

> Based on: `docs/frontend-design.en.md` §3.2, §6
> Architecture: Vite + React 19 + Ant Design 5 + TanStack Query 5 + Zustand 5. i18n `react-i18next` (zh-CN 优先, per-feature namespaces). 单位切换走 `unitStore` (Zustand) + `useUnit()` hook；后端 crate `erp-server`，DB = SQLite3. 术语见 `specs/UBIQUITOUS_LANGUAGE_LATEST.md` (商品 / SKU / 规格 / 质检 / 工单 / 采购订单 / 销售订单 等).

## Task List

### 1.1 Full i18n Coverage

- [ ] Audit every module for hardcoded Chinese text, extract into i18n keys:
  - `zh/common.json`: generic fields (actions, save, cancel, search, reset, status, date, etc.)
  - `en/common.json`: English translations
  - `zh/items.json` / `en/items.json`: item / SKU / inventory fields
  - `zh/inventory.json` / `en/inventory.json`: inventory fields
  - `zh/system.json` / `en/system.json`: system management fields
  - `zh/manufacturing.json` / `en/manufacturing.json`: 质检 (Inspection) / 工单 (Work Order) / BOM / NCR fields
  - `zh/orders.json` / `en/orders.json`: 采购订单 / 销售订单 fields
  - `zh/contracts.json` / `en/contracts.json`: 销售合同 / 采购合同 fields
  - `zh/reports.json` / `en/reports.json`: report fields
  - `zh/validation.json` / `en/validation.json`: validation messages
  - `zh/hr.json` / `en/hr.json`: 员工 / 考勤 / 薪资 / 劳动合同 fields
  - `zh/finance.json` / `en/finance.json`: 会计科目 / 日记账 / 发票 / 付款 / 试算平衡 fields
  - `zh/procurement.json` / `en/procurement.json`: 采购申请 / 采购订单 / 采购报价 / 采购收货 / 供应商评分 fields
  - `zh/projects.json` / `en/projects.json`: 项目 / WBS / 预算 fields
  - `zh/assets.json` / `en/assets.json`: 固定资产 / 折旧 / 资产处置 fields
  - `zh/notifications.json` / `en/notifications.json`: 通知 fields
  - `zh/portal.json` / `en/portal.json`: 门户 fields
  - `zh/bi.json` / `en/bi.json`: BI 分析 fields
  - `zh/search.json` / `en/search.json`: 通用商品搜索 fields
  - `zh/profile.json` / `en/profile.json`: 用户设置 fields
- [ ] Review English translations for accuracy: 商品分类名 + SKU 编码 + 通用单位名 保留英文 (always)
- [ ] Replace all hardcoded Chinese strings with `useTranslation()` hook

### 1.2 Unit System Switching

- [ ] Implement `src/stores/unitStore.ts` (Zustand):
  - State: `unitSystem: 'metric' | 'imperial'`
  - Actions: `toggleUnitSystem`, `setUnitSystem`
- [ ] Implement `src/shared/utils/unit-convert.ts`:
  - `formatLength(mm, unitSystem)`: mm ↔ inch (长度)
  - `formatWeight(kg, unitSystem)`: kg ↔ lb (重量)
  - `formatDiameter(mm, unitSystem)`: mm ↔ inch (2 decimal places, 直径/规格)
  - `formatPressure(MPa, unitSystem)`: MPa ↔ psi (压力)
  - `formatVolume(l, unitSystem)`: L ↔ gal (体积)
  - `formatArea(m2, unitSystem)`: m² ↔ ft² (面积)
- [ ] Implement `hooks/useUnit.ts`:
  - Read current unit system from unitStore
  - Return formatted values (auto-converts based on current setting)
  - Append unit suffix next to numbers
- [ ] Integrate unit conversion across all item spec displays / tables / details (长度 / 直径 / 重量)
- [ ] Keep the unit toggle button in the header

### 1.3 Date Formatting

- [ ] Use dayjs locale switching (zh-cn / en)
- [ ] Consistent global date format: Chinese `YYYY-MM-DD HH:mm` / English `MM/DD/YYYY HH:mm`

### 1.4 Testing

- [ ] Verify all page text switches correctly after toggling language
- [ ] Verify all numbers and units convert correctly after toggling unit system (check rounding precision)

> **Dependencies**: All frontend modules (touches every single one)
