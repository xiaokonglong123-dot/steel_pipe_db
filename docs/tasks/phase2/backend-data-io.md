# Phase 2 — Backend: Data Import/Export Module (P1)

> Based on: `docs/requirements.en.md` §3.5; `docs/detailed-design.en.md` §11

## Tasks

### 1.1 Dependencies
- [ ] Add Cargo deps: `calamine` (read Excel), `rust_xlsxwriter` (write Excel), `csv` (CSV read/write)

### 1.2 Import
- [ ] Implement Excel parser:
  - Read `.xlsx` / `.xls` formats
  - Auto-detect header row and map to struct fields
  - Support zh/en header matching
  - Validate data format and required fields row by row
- [ ] Implement CSV parser:
  - Auto-detect UTF-8 / GBK encoding
  - Auto-detect comma / tab delimiter
- [ ] Implement import Service:
  - `import_seamless_pipes(file)`: batch import seamless pipes
  - `import_screen_pipes(file)`: batch import screen pipes
  - Tx: commit only if all succeed, rollback partial failures and return error rows
  - Generate import result report (success count / failure count + per-row error reason)
  - Pipe number duplicate handling strategy (skip / overwrite / auto-generate new number)
- [ ] Implement import endpoints:
  - `POST /api/v1/import/seamless-pipes` — upload file to import seamless pipes
  - `POST /api/v1/import/screen-pipes` — upload file to import screen pipes
  - `GET /api/v1/import/template/seamless-pipes` — download import template
  - `GET /api/v1/import/template/screen-pipes` — download import template

### 1.3 Export
- [ ] Implement Excel generator:
  - Multi-sheet export support
  - Auto-set column widths, header styles (industrial blue theme)
  - Split large data across sheets
- [ ] Implement CSV generator
- [ ] Implement export Service:
  - `export_inventory_report(filter)`: export inventory report
  - `export_inbound_detail(filter)`: export inbound detail
  - `export_outbound_detail(filter)`: export outbound detail
  - `export_pipe_list(filter)`: export pipe list
  - Support selectable export fields
  - Support paginated export (stream all data)
- [ ] Implement export endpoints:
  - `POST /api/v1/export/inventory` — export inventory report
  - `POST /api/v1/export/inbound` — export inbound detail
  - `POST /api/v1/export/outbound` — export outbound detail
  - `POST /api/v1/export/pipes` — export pipe list
  - (Request body specifies format: excel/csv + filter criteria)

### 1.4 Tests
- [ ] Test import template generation (correct format)
- [ ] Test 10k row import performance (should be ≤ 10s)
- [ ] Test import data validation (missing required fields, format errors, duplicate numbers)
- [ ] Test export file readability (verify by reading back with calamine)

> **Deps**: Pipe management module (references pipe types/field mappings)
