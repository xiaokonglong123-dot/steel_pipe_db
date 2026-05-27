# Phase 2 — Backend: Quality Management Module (P1)

> Based on: `docs/requirements.en.md` §3.3; `docs/detailed-design.en.md` §4.4, §5.3.14, §5.3.18, §6.4

## Tasks

### 1.1 DB Migration
- [ ] Create `quality_certs` table migration (inspection certificates)
- [ ] Create `api_5ct_grade_ref` table migration (API 5CT grade reference data)
- [ ] Create `pipe_attachments` table migration (pipe attachments)

### 1.2 Domain Layer
- [ ] Define `QualityCert` struct
- [ ] Define `Api5ctGradeRef` struct
- [ ] Define `PipeAttachment` struct
- [ ] Define DTOs: `CreateQualityCertDto`, `UpdateQualityCertDto`
- [ ] Define enum: `CertResult` (Pass / Fail / Pending)

### 1.3 Repository Layer
- [ ] Implement `QualityCertRepo`: create, update, find_by_id, list, find_by_pipe(pipe_type, pipe_id)
- [ ] Implement `GradeRefRepo`: get_by_grade(grade), list_all
- [ ] Implement `AttachmentRepo`: create, delete, find_by_pipe(pipe_type, pipe_id)

### 1.4 Service Layer
- [ ] Implement `QualityService`:
  - `create_cert(dto)`: create quality cert, link to pipe
  - `update_cert(id, dto)`: update quality record
  - `list_certs(filter)`: query quality cert list with filters
  - `trace_by_heat_number(heat_no)`: trace quality records by heat number
  - `trace_by_pipe_number(pipe_no)`: trace by pipe number
  - `get_api_5ct_reference(grade)`: get mechanical/chemical reference data for a grade
  - `attach_file(pipe_type, pipe_id, file)`: upload attachment

### 1.5 Handler Layer
- [ ] `GET /api/v1/quality/certs` — list quality certs
- [ ] `POST /api/v1/quality/certs` — create quality cert
- [ ] `PUT /api/v1/quality/certs/{id}` — update quality cert
- [ ] `GET /api/v1/quality/certs/{id}` — quality cert detail
- [ ] `GET /api/v1/quality/trace/heat-number/{heat_no}` — trace by heat number
- [ ] `GET /api/v1/quality/trace/pipe-number/{pipe_no}` — trace by pipe number
- [ ] `GET /api/v1/quality/api5ct/grades` — list API 5CT grade reference data
- [ ] `GET /api/v1/quality/api5ct/grades/{grade}` — single grade reference detail
- [ ] `POST /api/v1/quality/attachments` — upload attachment
- [ ] `DELETE /api/v1/quality/attachments/{id}` — delete attachment

### 1.6 Data Seeding
- [ ] Insert API 5CT common grade mechanical property reference data in migration (see PRD §5.1)

### 1.7 Tests
- [ ] Test quality cert CRUD
- [ ] Test traceability endpoint accuracy
- [ ] Test attachment upload/delete

> **Deps**: Pipe management module (references pipe IDs)
