# Contributing

See [AGENTS.md](./AGENTS.md) for the project index, architecture, and developer conventions.

## Local development

```bash
# Backend
cd backend && cargo check --all-targets && cargo test --all

# Frontend
cd frontend && bun install && bunx tsc --noEmit && bun run build
```

## Conventions

- Atomic commits, descriptive messages (see repo log for style).
- No force-push to `main` without coordination.
- The `legacy/steel-pipe-react` branch is read-only history — do not add new commits there.
- Design docs in `docs/` are the source of truth; keep them in sync when changing architecture.
- `AGENTS.md` is the authoritative project index — update it when module structure or status changes.

## CI

`.github/workflows/ci.yml` runs on every push / PR to `main`:
- Backend: `cargo check --all-targets` + `cargo test --all`
- Frontend: `bun install --frozen-lockfile` + `bunx tsc --noEmit` + `bun run build`
