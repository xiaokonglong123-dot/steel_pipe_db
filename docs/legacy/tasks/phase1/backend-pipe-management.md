# Phase 1 — Backend: Pipe Management Module (P0 MVP)

> Based on: `docs/requirements.en.md` §3.1, §3.6, §6.2; `docs/detailed-design.en.md` §4.2, §5.3.1-2, §6.2

## Tasks

### 1.1 Project Setup
- [ ] Init Rust project (`cargo init`), configure `Cargo.toml` deps:
  - axum 0.8+, tokio, serde, serde_json, sqlx (sqlite feature), jsonwebtoken, argon2, validator, tracing, tower-http (cors)
- [ ] Create dir structure: `src/{domain,handler,service,repository,config,middleware,error}`
- [ ] Implement `src/main.rs`: Axum router assembly, SQLite pool init (WAL mode), start tracing
- [ ] Implement `src/config/mod.rs`: read config from env / `.env` (DB path, JWT secret, port)

### 1.2 DB Migration
- [ ] Create `migrations/` dir, write initial SQL migrations:
  - `seamless_pipes` table (all fields + indexes)
  - `screen_pipes` table (all fields + indexes)
  - Include `PRAGMA journal_mode = WAL` etc.
- [ ] Auto-run migrations on startup

### 1.3 Domain Layer
- [ ] Define `SeamlessPipe` struct (maps to `seamless_pipes` table)
- [ ] Define `ScreenPipe` struct (maps to `screen_pipes` table)
- [ ] Define enums: `PipeType` (Casing/Tubing), `PipeStatus` (InStock/Outbound/Scrapped), `EndType` (SC/LC/BC/X), `ScreenType` (WireWrapped/Slotted/Punched/MetalFelt)
- [ ] Define DTOs: `CreateSeamlessPipeDto`, `UpdateSeamlessPipeDto`, `CreateScreenPipeDto`, `UpdateScreenPipeDto`
- [ ] Define filter params: `SeamlessPipeFilter`, `ScreenPipeFilter` (optional query fields + pagination + sorting)
- [ ] Define unified response types: `ApiResponse<T>`, `PaginatedResponse<T>`, `ApiError`
- [ ] Add `Validator` derive annotations (required fields, range checks)

### 1.4 Repository Layer
- [ ] Implement `SeamlessPipeRepo`:
  - `create(dto) -> SeamlessPipe`
  - `update(id, dto) -> SeamlessPipe`
  - `delete(id)` (soft delete)
  - `find_by_id(id) -> Option<SeamlessPipe>`
  - `find_by_pipe_number(number) -> Option<SeamlessPipe>` (uniqueness check)
  - `list(filter) -> PaginatedResult<SeamlessPipe>` (multi-condition filtering + sort + pagination)
- [ ] Implement `ScreenPipeRepo` (same deal, for screen pipes)

### 1.5 Service Layer
- [ ] Implement `PipeService`:
  - `create_seamless_pipe(dto)`: validate + check uniqueness + call Repo + log operation
  - `update_seamless_pipe(id, dto)`: check existence + update + log
  - `delete_seamless_pipe(id)`: check inventory status (only if in stock) + soft delete + log
  - `get_seamless_pipe(id)`: get details (with location JOIN)
  - `list_seamless_pipes(filter)`: filtered query + pagination
  - `create_screen_pipe(dto)` / `update_screen_pipe` / `delete_screen_pipe` / `get_screen_pipe` / `list_screen_pipes`
  - `generate_pipe_number(pipe_type, grade, od, wt)`: auto-generate unique pipe number by format
  - `validate_pipe_number_unique(number)`: check uniqueness
  - `search_pipes(query)`: cross-table fuzzy search (pipe_number / heat_number / serial_number)

### 1.6 Handler Layer
- [ ] Implement seamless pipe REST endpoints:
  - `GET /api/v1/seamless-pipes` — list (query params: q, grade, pipe_type, od_min, od_max, wt_min, wt_max, status, location_id, manufacturer, sort_by, sort_order, page, page_size)
  - `POST /api/v1/seamless-pipes` — create
  - `GET /api/v1/seamless-pipes/{id}` — detail
  - `PUT /api/v1/seamless-pipes/{id}` — full update
  - `DELETE /api/v1/seamless-pipes/{id}` — delete
- [ ] Implement screen pipe REST endpoints (same structure, add screen_type / slot_size etc.):
  - `GET /api/v1/screen-pipes`
  - `POST /api/v1/screen-pipes`
  - `GET /api/v1/screen-pipes/{id}`
  - `PUT /api/v1/screen-pipes/{id}`
  - `DELETE /api/v1/screen-pipes/{id}`
- [ ] Implement unified search endpoint:
  - `GET /api/v1/pipes/search?q={query}`
- [ ] Implement unified error handling (404/400/409/500 mapping)
- [ ] Implement pagination params extractor

### 1.7 Tests
- [ ] Write Repo layer unit tests (using SQLite :memory:)
- [ ] Write Service layer unit tests (mock Repo or integration)
- [ ] Write Handler layer integration tests (using `axum::test`)
- [ ] Test pipe number uniqueness validation
- [ ] Test combined filtering + pagination + sorting

> **Deps**: None (self-contained module)
