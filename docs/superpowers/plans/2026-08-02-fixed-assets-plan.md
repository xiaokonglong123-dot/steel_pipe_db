# Fixed Assets Implementation Plan

**Goal:** Manage fixed assets, depreciation, transfer, disposal.

**Architecture:** New `backend/src/assets/` module (repos/services/handlers).

**Database:** SQLite3 (`sqlite://data/erp.db?mode=rwc`)

---

### Task 1: Tables — fixed_assets, depreciation_entries

### Task 2: Straight-line depreciation computation on each period end

### Task 3: Journal entry integration (新修 journal entry for 折旧)

### Task 4: Transfer between departments, disposal

### Task 5: Frontend — AssetRegisterPage, DepreciationPage
