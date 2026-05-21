# `docs/` — Design Documents & Architecture Decisions

## Structure

```
docs/
├── AGENTS.md           ← This file: index & architecture decisions
├── backend/            ← Backend design docs
│   ├── project-plan.md
│   ├── prd.md
│   └── requirements.md
├── frontend/           ← Frontend design docs
│   └── tech-stack.md
└── tasks/              ← Task breakdown / sprint planning
    ├── backend-tasks.md
    └── frontend-tasks.md
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
- `backend/prd.md` — Product requirements, user stories
- `backend/requirements.md` — Functional requirements, acceptance criteria
- `backend/project-plan.md` — Implementation plan, milestones
- `frontend/tech-stack.md` — Frontend technology decisions and rationale
- `tasks/backend-tasks.md` — Backend task breakdown, status tracking
- `tasks/frontend-tasks.md` — Frontend task breakdown, status tracking

## Process Notes
- Docs are living documents — update when implementation reveals design gaps
- AGENTS.md files are the canonical reference for AI-assisted development
- Task breakdown in `docs/tasks/` tracks implementation status
