# ── Stage 1: Build backend ──
FROM rust:1.78-slim AS backend-builder
WORKDIR /app/backend

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Cache dependencies by building a dummy project first
COPY backend/Cargo.toml backend/Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

# Build actual backend
COPY backend/ .
RUN touch src/main.rs && cargo build --release

# ── Stage 2: Build frontend ──
FROM node:20-slim AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ .
RUN npm run build

# ── Stage 3: Runtime ──
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends     ca-certificates     libssl3     curl     && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Backend binary and migrations
COPY --from=backend-builder /app/backend/target/release/steel-pipe-db /app/steel-pipe-db
COPY backend/migrations/ /app/migrations/

# Frontend dist (serve via Nginx or other reverse proxy)
COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist

# Data directory for SQLite
RUN mkdir -p /app/data

# Default env (override via docker-compose or -e flags)
ENV DATABASE_URL=sqlite://./data/steel_pipe.db?mode=rwc
ENV JWT_SECRET=change-this-to-a-long-random-secret
ENV JWT_EXPIRY_HOURS=24
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=3000

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=10s --retries=3     CMD curl -f http://localhost:3000/api/v1/auth/login || exit 1

CMD ["/app/steel-pipe-db"]
