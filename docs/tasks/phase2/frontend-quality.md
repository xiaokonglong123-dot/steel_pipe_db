# Phase 2 — Frontend: Quality Management Module (P1)

> Based on: `docs/frontend-design.en.md` §4.1, §4.2

## Tasks

### 1.1 Shared Types & API
- [ ] Define `features/quality/types.ts`: QualityCert, Api5ctGradeRef, PipeAttachment etc.
- [ ] Define `features/quality/api/qualityApi.ts`:
  - `getQualityCerts(filter, page, pageSize)` / `getQualityCert(id)` / `createQualityCert(data)` / `updateQualityCert(id, data)`
  - `traceByHeatNumber(heatNo)` / `traceByPipeNumber(pipeNo)`
  - `getApi5ctGrades()` / `getApi5ctGrade(grade)`
  - `uploadAttachment(pipeType, pipeId, file)` / `deleteAttachment(id)`

### 1.2 Quality Cert Pages
- [ ] Implement `QualityCertListPage`:
  - Filters: pipe number, grade, test result, test date range
  - Cert table (cert number, pipe number, test date, test lab, result, actions)
  - Row actions: view detail, edit, delete
  - +New quality cert button
- [ ] Implement `QualityCertFormPage`:
  - Select linked pipe (pipe search selector)
  - Fill test info (cert number, test date, test lab, inspector)
  - Test result select (Pass / Fail / Pending)
  - Dynamic test item add component (name + result + standard value)
  - File upload (inspection report PDF / image)
- [ ] Implement `QualityCertDetailPage` (could be Modal or standalone page):
  - Show all quality cert fields
  - Linked pipe info card
  - Attachment list (download / preview)

### 1.3 Quality Traceability Page
- [ ] Implement `QualityTracePage`:
  - Choose input method: by heat number OR by pipe number
  - Input search criteria → show trace results
  - Results: pipe info + timeline (all quality records + inbound/outbound records)
  - Use Ant Design Timeline for display

### 1.4 API 5CT Standard Reference Page
- [ ] Implement `Api5ctRefPage`:
  - Grade list table (grade, group, min yield strength, max yield strength, min tensile strength, hardness, description)
  - Grade filter / search
  - Click row to expand grade detail info

### 1.5 Shared Components
- [ ] Implement `CertFileUploader`: quality file upload component (preview + drag-and-drop)
- [ ] Implement `TraceTimeline`: trace timeline component (different event types get different icons/colors)
- [ ] Implement `GradeCompareTable`: comparison table for test results vs API 5CT standard values

### 1.6 i18n
- [ ] Create `src/i18n/resources/zh/quality.json` and `en/quality.json`

> **Deps**: Pipe management frontend module (pipe search selector)
