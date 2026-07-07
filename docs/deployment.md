# 部署指南 / Deployment Guide

## 生产环境配置

### 后端环境变量

复制 `.env.example` 为 `.env`，并**务必修改**以下配置：

```bash
cp .env.example .env
```

| 变量 | 生产环境要求 | 说明 |
|------|-------------|------|
| `DATABASE_URL` | 确保路径可写 | SQLite 文件路径，`mode=rwc` 自动创建 |
| `JWT_SECRET` | **必须修改** | 使用 `openssl rand -base64 48` 生成 |
| `JWT_EXPIRY_HOURS` | 按需调整 | 默认 24 小时 |
| `SERVER_HOST` | 建议 `127.0.0.1` | 仅本地监听，配合反向代理 |
| `SERVER_PORT` | 按需调整 | 默认 3000 |

### 生成安全的 JWT 密钥

```bash
openssl rand -base64 48
```

将输出结果填入 `JWT_SECRET=...`。

---

## 构建与运行

### 后端（Rust）

```bash
cd backend

# 发布构建（优化编译，性能显著优于 debug）
cargo build --release

# 运行
./target/release/steel-pipe-db
```

构建产物位于 `backend/target/release/steel-pipe-db`。

### 前端（React）

```bash
cd frontend

# 安装依赖
npm ci

# 生产构建
npm run build
```

构建产物位于 `frontend/dist/` 目录。

---

## Nginx 反向代理配置

推荐使用 Nginx 作为反向代理，将前端静态文件和后端 API 统一入口：

```nginx
server {
    listen 80;
    server_name your-domain.com;

    # 前端静态文件
    location / {
        root /var/www/steel-pipe-db/frontend/dist;
        index index.html;
        try_files $uri $uri/ /index.html;  # SPA 路由回退
    }

    # 后端 API 代理
    location /api/ {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # 文件上传大小限制（Excel 导入）
        client_max_body_size 50m;
    }

    # 静态资源缓存
    location /assets/ {
        root /var/www/steel-pipe-db/frontend/dist;
        expires 30d;
        add_header Cache-Control "public, immutable";
    }
}
```

---

## Systemd 服务配置

创建 `/etc/systemd/system/steel-pipe-db.service`：

```ini
[Unit]
Description=Steel Pipe DB API Server
After=network.target

[Service]
Type=simple
User=steel-pipe
WorkingDirectory=/opt/steel-pipe-db/backend
EnvironmentFile=/opt/steel-pipe-db/backend/.env
ExecStart=/opt/steel-pipe-db/backend/target/release/steel-pipe-db
Restart=on-failure
RestartSec=5

# 安全限制
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/steel-pipe-db/backend/data

[Install]
WantedBy=multi-user.target
```

启动服务：

```bash
sudo systemctl daemon-reload
sudo systemctl enable steel-pipe-db
sudo systemctl start steel-pipe-db
sudo systemctl status steel-pipe-db
```

---

## SQLite 数据库备份

### 手动备份

```bash
# 使用 sqlite3 进行一致性备份（推荐，不会锁定数据库）
sqlite3 data/steel_pipe.db ".backup data/steel_pipe_backup_$(date +%Y%m%d_%H%M%S).db"
```

### 定时备份（crontab）

```bash
# 每天凌晨 2 点自动备份，保留最近 30 天
0 2 * * * sqlite3 /opt/steel-pipe-db/backend/data/steel_pipe.db ".backup /opt/steel-pipe-db/backups/steel_pipe_$(date +\%Y\%m\%d).db" && find /opt/steel-pipe-db/backups/ -name "steel_pipe_*.db" -mtime +30 -delete
```

### 恢复备份

```bash
# 停止服务
sudo systemctl stop steel-pipe-db

# 替换数据库文件
cp data/steel_pipe_backup_YYYYMMDD.db data/steel_pipe.db

# 重启服务
sudo systemctl start steel-pipe-db
```

---

## Docker 部署（可选）

### Dockerfile

```dockerfile
# ── Stage 1: Build backend ──
FROM rust:1.78-slim AS backend-builder
WORKDIR /app/backend
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY backend/ .
RUN cargo build --release

# ── Stage 2: Build frontend ──
FROM node:20-slim AS frontend-builder
WORKDIR /app/frontend
COPY frontend/ .
RUN npm ci && npm run build

# ── Stage 3: Runtime ──
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Backend binary
COPY --from=backend-builder /app/backend/target/release/steel-pipe-db /app/steel-pipe-db
# Backend .env and migrations
COPY backend/.env /app/.env
COPY backend/migrations/ /app/migrations/

# Frontend dist
COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist

# Data directory for SQLite
RUN mkdir -p /app/data

EXPOSE 3000

CMD ["/app/steel-pipe-db"]
```

### docker-compose.yml

```yaml
version: "3.8"
services:
  app:
    build: .
    ports:
      - "3000:3000"
    volumes:
      - ./data:/app/data        # SQLite 数据持久化
      - ./.env:/app/.env:ro     # 环境变量（只读）
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/api/v1/auth/login"]
      interval: 30s
      timeout: 10s
      retries: 3
```

---

## 安全检查清单

- [ ] `JWT_SECRET` 已替换为强随机密钥（非默认值）
- [ ] `SERVER_HOST` 设为 `127.0.0.1`（不直接暴露到公网）
- [ ] Nginx 已配置 HTTPS（使用 Let's Encrypt 或其他证书）
- [ ] 数据库文件定期备份
- [ ] 默认 `admin/admin123` 密码已在首次登录后修改
- [ ] CORS 仅允许前端域名（非 `*`）
- [ ] 文件上传大小限制已配置
