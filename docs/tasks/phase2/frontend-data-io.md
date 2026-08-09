# Phase 2 — Frontend: Data Import/Export Module (P1)

> Based on: `docs/frontend-design.en.md` §4.1
> Architecture: Vite + React 19 + Ant Design 5 + TanStack Query 5. 导入导出对象统一为「商品 (Item) + SKU」+ 库存/订单/合同等通用业务实体；无管材专属字段. 后端 crate `erp-server`，DB = SQLite3. 术语见 `specs/UBIQUITOUS_LANGUAGE_LATEST.md`。

## Tasks

### 1.1 Import Page

- [ ] Implement `ImportPage`:
  - Tab switch: Import Items / Import Suppliers / Import Customers
  - Download import template button (.xlsx template, with header descriptions + required field markers)
  - File upload area (drag-and-drop or click to upload, supports .xlsx / .xls / .csv)
  - Import config: SKU 重复处理策略 (skip / overwrite / auto-generate new SKU)
  - Preview after upload: table showing first N rows of parsed data
  - Confirm import button → calls backend API
- [ ] Implement import results display:
  - On success: success count + failure count + per-row failure reason table
  - Failure report available for download
  - Import history (recent import record list)

### 1.2 Export Page

- [ ] Implement `ExportPage`:
  - Select export type: inventory report / inbound detail / outbound detail / item list / purchase orders / sales orders
  - Dynamic filter display based on type (date range, category, location, etc.)
  - Field selector: check which fields to export
  - File format: Excel / CSV
  - Export button → triggers download

### 1.3 Shared Components

- [ ] Implement `FileUploader`: drag-and-drop + click upload, file type/size limits, progress bar
- [ ] Implement `ImportResultTable`: import error result table (row number, reason, raw data)

> **Deps**: 商品主数据 (item) 前端模块、采购前端模块、销售前端模块、库存前端模块
