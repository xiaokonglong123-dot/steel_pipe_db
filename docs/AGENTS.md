# `docs/` — Design Documents & Architecture Decisions

## Structure

```
docs/
├── AGENTS.md              ← This file
├── AGENTS_zh.md           ← Chinese version
├── 需求文档.md             ← PRD (Chinese)
├── 详细设计文档.md          ← Architecture & design (Chinese)
├── 前端设计文档.md           ← Frontend design (Chinese)
├── requirements.en.md     ← PRD (English)
├── detailed-design.en.md  ← Detailed design (English)
├── frontend-design.en.md  ← Frontend design (English)
├── tasks/                 ← Task breakdown
│   ├── progress.md
│   ├── phase1/            ← Auth, pipes, inventory
│   ├── phase2/            ← Business features
│   └── phase3/            ← Enterprise features
└── superpowers/           ← Architecture specs
    └── specs/
```

## Architecture Decisions

### Why SQLite?
- No external database server needed in production
- Single-file database, easy to backup and deploy
- SQLx compile-time query checking catches SQL errors at build time
- Adequate for single-warehouse/multi-warehouse scale

### Why Rust + React?
- **Rust**: Type safety, performance for report generation and inventory calculations, memory safety without GC overhead. Axum provides ergonomic async handlers.
- **React 19**: Mature ecosystem, Ant Design provides enterprise-grade UI components out of the box, TanStack Query simplifies server state management.

### Why Feature-based Frontend?
- Each feature (pipes, inventory, purchases, etc.) is self-contained
- Clear boundaries prevent cross-module coupling
- Parallel development possible (different agents work on different features)
- Easy to add/remove features without touching unrelated code

### Monorepo vs Separate Repos
- Single repository for coordinated versioning
- Direct cargo/npm commands, each package independent
- Backend serves frontend dist from embedded static files; frontend dev uses Vite proxy to backend

## Decision Records

| Decision | Choice | Alternative | Rationale |
|----------|--------|-------------|-----------|
| Database | SQLite | PostgreSQL | Simpler deployment, adequate scale |
| HTTP framework | Axum 0.8 | Actix, Rocket | Ecosystem, ergonomics, tower ecosystem |
| ORM | SQLx | Diesel, SeaORM | Compile-time SQL checking, no ORM overhead |
| UI library | Ant Design 5 | MUI, ShadCN | Enterprise focus, Chinese ecosystem, table quality |
| State management | TanStack Query | Redux, Zustand | Server state focus, caching, deduplication |
| i18n | i18next | react-intl, Lingui | Mature ecosystem, namespace support, lazy loading |
| Auth | JWT + RBAC | Session-based | Stateless, mobile-friendly |

## Key Design Docs
- `需求文档.md` — Product requirements (Chinese)
- `详细设计文档.md` — Architecture & database design (Chinese)
- `前端设计文档.md` — Frontend component tree & routing (Chinese)
- `requirements.en.md` — Product requirements (English)
- `detailed-design.en.md` — Architecture & design (English)
- `frontend-design.en.md` — Frontend design (English)
- `tasks/progress.md` — Master task tracking across phases

## Process Notes
- Docs are living documents — update when implementation reveals design gaps
- AGENTS.md files are the canonical reference for AI-assisted development
- Task breakdown in `docs/tasks/` tracks implementation status
