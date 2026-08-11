# Phase 3 — Frontend: Internationalization & Unit Switching (P2)

> Based on: `docs/frontend-design.en.md` §3.2, §6

## Task List

### 1.1 Full i18n Coverage
- [ ] Audit every module for hardcoded Chinese text, extract into i18n keys:
  - `zh/common.json`: generic fields (actions, save, cancel, search, reset, status, date, etc.)
  - `en/common.json`: English translations
  - `zh/pipes.json` / `en/pipes.json`: pipe-related fields
  - `zh/inventory.json` / `en/inventory.json`: inventory fields
  - `zh/system.json` / `en/system.json`: system management fields
  - `zh/quality.json` / `en/quality.json`: quality fields
  - `zh/orders.json` / `en/orders.json`: order fields
  - `zh/contracts.json` / `en/contracts.json`: contract fields
  - `zh/reports.json` / `en/reports.json`: report fields
  - `zh/labels.json` / `en/labels.json`: label fields
  - `zh/validation.json` / `en/validation.json`: validation messages
- [ ] Review English translations for accuracy: API 5CT grade names and pipe type names stay in English, always
- [ ] Replace all hardcoded Chinese strings with `useTranslation()` hook

### 1.2 Unit System Switching
- [ ] Implement `src/stores/unitStore.ts` (Zustand):
  - State: `unitSystem: 'metric' | 'imperial'`
  - Actions: `toggleUnitSystem`, `setUnitSystem`
- [ ] Implement `src/shared/utils/unit-convert.ts`:
  - `formatLength(mm, unitSystem)`: mm ↔ inch
  - `formatWeight(kg, unitSystem)`: kg ↔ lb
  - `formatDiameter(mm, unitSystem)`: mm ↔ inch (2 decimal places)
  - `formatPressure(MPa, unitSystem)`: MPa ↔ psi
- [ ] Implement `hooks/useUnit.ts`:
  - Read current unit system from unitStore
  - Return formatted values (auto-converts based on current setting)
  - Append unit suffix next to numbers
- [ ] Integrate unit conversion across all pipe spec displays / tables / details
- [ ] Keep the unit toggle button in the header

### 1.3 Date Formatting
- [ ] Use dayjs locale switching (zh-cn / en)
- [ ] Consistent global date format: Chinese `YYYY-MM-DD HH:mm` / English `MM/DD/YYYY HH:mm`

### 1.4 Testing
- [ ] Verify all page text switches correctly after toggling language
- [ ] Verify all numbers and units convert correctly after toggling unit system (check rounding precision)

> **Dependencies**: All frontend modules (touches every single one)
