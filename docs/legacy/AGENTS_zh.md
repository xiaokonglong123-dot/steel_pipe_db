# `docs/` — Design Docs & Architecture Decisions

The design rationale, architecture choices, and links to the detailed docs.

## Structure

```
docs/
├── AGENTS.md              ← English version
├── AGENTS_zh.md           ← This file
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

- No database server to install or manage in production.
- Single-file storage — trivial backups, trivial deployment.
- SQLx catches SQL errors at compile time.
- Handles single/multi-warehouse scale just fine.

### Why Rust + React?

- **Rust**: Type safety, fast report generation, memory safe without GC. Axum makes async handlers straightforward.
- **React 19**: Battle-tested ecosystem. Ant Design gives us enterprise UI components. TanStack Query cleans up server state management.

### Why Feature-based Frontend?

- Each feature is isolated (pipes, inventory, purchases, etc.).
- Clean boundaries prevent modules from tangling.
- You can parallelize development across features.
- Adding or removing a feature doesn't ripple through unrelated code.

### Monorepo vs Separate Repos

- Single repo for coordinated versioning.
- No complex monorepo tooling — backend and frontend each have their own build commands.
- Backend serves the built frontend from embedded static files. In dev, Vite proxies API calls to the backend.

## Decision Records

| Decision | Choice | Alternative | Why |
|----------|--------|-------------|-----|
| Database | SQLite | PostgreSQL | Simpler to deploy, fast enough |
| HTTP framework | Axum 0.8 | Actix, Rocket | Good ergonomics, tower ecosystem |
| ORM | SQLx | Diesel, SeaORM | Compile-time SQL checking, minimal overhead |
| UI library | Ant Design 5 | MUI, ShadCN | Enterprise-ready, great tables, Chinese ecosystem |
| State management | TanStack Query | Redux, Zustand | Built for server state — caching, dedup, refetch |
| i18n | i18next | react-intl, Lingui | Mature, namespaces, lazy loading |
| Auth | JWT + RBAC | Session-based | Stateless, mobile-friendly |

## Key Design Docs

- `需求文档.md` — Product requirements (Chinese)
- `详细设计文档.md` — Architecture & database design (Chinese)
- `前端设计文档.md` — Frontend component tree & routing (Chinese)
- `requirements.en.md` — Product requirements (English)
- `detailed-design.en.md` — Architecture & design (English)
- `frontend-design.en.md` — Frontend design (English)
- `tasks/progress.md` — Master task tracking

## Process Notes

- Docs are living — update them when implementation reveals design gaps.
- AGENTS.md files are the canonical reference for AI-assisted development.
- Task breakdown in `docs/tasks/` tracks implementation status across phases.
