# Phase 2 — Frontend: Data Import/Export Module (P1)

> Based on: `docs/frontend-design.en.md` §4.1

## Tasks

### 1.1 Import Page
- [ ] Implement `ImportPage`:
  - Tab switch: Import Seamless Pipes / Import Screen Pipes
  - Download import template button (.xlsx template, with header descriptions + required field markers)
  - File upload area (drag-and-drop or click to upload, supports .xlsx / .xls / .csv)
  - Import config: duplicate number handling strategy (skip / overwrite / auto-number)
  - Preview after upload: table showing first N rows of parsed data
  - Confirm import button → calls backend API
- [ ] Implement import results display:
  - On success: success count + failure count + per-row failure reason table
  - Failure report available for download
  - Import history (recent import record list)

### 1.2 Export Page
- [ ] Implement `ExportPage`:
  - Select export type: inventory report / inbound detail / outbound detail / pipe list
  - Dynamic filter display based on type (date range, pipe type, grade etc.)
  - Field selector: check which fields to export
  - File format: Excel / CSV
  - Export button → triggers download

### 1.3 Shared Components
- [ ] Implement `FileUploader`: drag-and-drop + click upload, file type/size limits, progress bar
- [ ] Implement `ImportResultTable`: import error result table (row number, reason, raw data)

> **Deps**: Pipe management frontend module
