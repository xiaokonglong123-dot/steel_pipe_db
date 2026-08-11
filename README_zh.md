# Ikari_Shinji — ERP

通用 ERP 系统（企业资源计划）。

- **后端**: Rust + Axum + SQLx + SQLite（Decimal 金额、JWT/Argon2）
- **前端**: Vue 3 + Pinia + Element Plus + TanStack Vue Query
- **包管理**: bun（前端）、cargo（后端）

详见 [AGENTS.md](./AGENTS.md) 项目索引、架构与开发约定。设计文档在 [docs/](./docs/)；重写前（React 栈）的设计文档归档于 [docs/legacy/](./docs/legacy/)。

## 快速开始

```bash
# 后端（Rust Axum :3000）
cd backend
cp .env.example .env
cargo run

# 前端（Vue 3 + Vite :5173）
cd frontend
bun install
bun run dev
```

打开 `http://localhost:5173`，用 `admin` / `admin123` 登录。

## 构建与校验

| 内容 | 命令 |
| ---- | ---- |
| 后端类型检查 | `cd backend && cargo check --all-targets` |
| 后端测试 | `cd backend && cargo test --all` |
| 前端类型检查 | `cd frontend && bunx tsc --noEmit` |
| 前端构建 | `cd frontend && bun run build` |

数据库采用 SQLite3（单文件 `backend/data/erp.db`）。121 个后端测试全绿，前端 tsc + build 全绿。

## 历史

重写前（React 19 + Ant Design）的代码在 `legacy/steel-pipe-react` 分支；当前 `main` 是 erp-v2 重写时代。
