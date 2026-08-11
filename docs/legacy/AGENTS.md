# `docs/` — Design Docs & Architecture Decisions

This is where the design rationale lives. Not everything is written down, but the important stuff is.

## Structure

```
docs/
├── AGENTS.md              ← This file
├── AGENTS_zh.md           ← Chinese version
├── 需求文档.md             ← PRD (originally Chinese, rewritten to English)
├── 详细设计文档.md          ← Architecture & design (originally Chinese, rewritten to English)
├── 前端设计文档.md           ← Frontend design (originally Chinese, rewritten to English)
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

- No external DB server to set up or maintain.
- Single-file database — easy to backup, move, or deploy.
- SQLx checks queries at compile time, so SQL errors get caught early.
- Plenty fast for single-warehouse or multi-warehouse scale.

### Why Rust + React?

- **Rust**: Type safety, solid perf for reports and inventory math, memory safety without GC. Axum's handlers are pleasant to work with.
- **React 19**: Mature ecosystem. Ant Design gives us enterprise-grade components out of the box. TanStack Query handles the server state mess.

### Why Feature-based Frontend?

- Each feature (pipes, inventory, purchases, etc.) is self-contained.
- Clear boundaries = no cross-module spaghetti.
- Multiple devs (or agents) can work on different features at the same time.
- Adding or removing features doesn't touch unrelated code.

### Monorepo vs Separate Repos

- Single repo keeps versions in sync.
- Backend and frontend each have their own build commands — no monorepo tooling overhead.
- Backend serves the built frontend from embedded static files. Dev mode uses Vite proxy.

## Decision Records

| Decision | Choice | Alternative | Why |
|----------|--------|-------------|-----|
| Database | SQLite | PostgreSQL | Simpler deployment, adequate scale |
| HTTP framework | Axum 0.8 | Actix, Rocket | Good ergonomics, tower middleware ecosystem |
| ORM | SQLx | Diesel, SeaORM | Compile-time SQL checking, no ORM overhead |
| UI library | Ant Design 5 | MUI, ShadCN | Enterprise focus, solid table component, Chinese ecosystem |
| State management | TanStack Query | Redux, Zustand | Purpose-built for server state — caching, dedup, refetch |
| i18n | i18next | react-intl, Lingui | Mature, namespace support, lazy loading |
| Auth | JWT + RBAC | Session-based | Stateless, works fine with mobile clients |

## Key Design Docs

- `需求文档.md` — Product requirements (orig. Chinese, rewritten to English)
- `详细设计文档.md` — Architecture & database design (orig. Chinese, rewritten to English)
- `前端设计文档.md` — Frontend component tree & routing (orig. Chinese, rewritten to English)
- `requirements.en.md` — Product requirements (English)
- `detailed-design.en.md` — Architecture & design (English)
- `frontend-design.en.md` — Frontend design (English)
- `tasks/progress.md` — Master task tracking

## Process Notes

- These docs are living — update them when implementation reveals something the design got wrong.
- AGENTS.md files are the canonical source of truth for AI-assisted work.
- Task breakdown in `docs/tasks/` tracks what's been done and what hasn't.
