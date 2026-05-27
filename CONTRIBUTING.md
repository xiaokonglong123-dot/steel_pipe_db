# 贡献指南 / Contributing Guide

感谢你对 Steel Pipe DB 项目的关注！本文档介绍如何参与项目贡献。

## 开发环境设置

请参考 [README.md](../README.md) 中的 🚀 Quick Start 章节配置开发环境。

### 必备工具

| 工具 | 版本 | 用途 |
|------|------|------|
| Rust | 1.78+ (edition 2021) | 后端开发 |
| Node.js | 20+ | 前端开发 |
| npm | 10+ | 前端依赖管理 |
| sqlite3 | 3.x+ | 数据库调试 |

---

## Git 工作流

### 分支策略

| 分支 | 用途 |
|------|------|
| `main` | 稳定发布分支，只接受 PR 合入 |
| `feat/<feature-name>` | 新功能开发 |
| `fix/<issue-description>` | Bug 修复 |
| `docs/<topic>` | 文档改进 |

### Commit Message 规范

遵循 [Conventional Commits](https://www.conventionalcommits.org/) 格式：

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**类型（type）：**

| 类型 | 说明 |
|------|------|
| `feat` | 新功能 |
| `fix` | Bug 修复 |
| `docs` | 文档变更 |
| `style` | 代码格式（不影响逻辑） |
| `refactor` | 重构（非新功能、非修复） |
| `perf` | 性能优化 |
| `test` | 测试相关 |
| `chore` | 构建/工具变更 |

**范围（scope）：**

| 范围 | 说明 |
|------|------|
| `backend` | Rust 后端 |
| `frontend` | React 前端 |
| `api` | API 端点变更 |
| `db` | 数据库/迁移变更 |
| `auth` | 认证/权限相关 |
| `i18n` | 国际化 |

**示例：**

```
feat(backend): add batch inbound import endpoint
fix(frontend): resolve inventory filter reset on page change
docs(api): update RBAC permission matrix
refactor(backend): split inventory_service into focused modules
```

---

## 代码规范

### 后端（Rust）

- `snake_case` 用于函数和变量，`PascalCase` 用于类型
- 所有 handler 返回 `Result<Json<...>, AppError>`
- Service 使用 unit struct + static methods：`PipeService::list(...)`
- Repository 接受 `&SqlitePool`，返回 `Result<Vec<T>, sqlx::Error>`
- 使用 `///` 文档注释标注所有 `pub async fn`
- 使用 `#![allow(dead_code)]` 仅在 `lib.rs` 顶部

```bash
# 提交前检查
cargo check           # 类型检查
cargo test            # 运行测试
cargo clippy          # Lint 检查
```

### 前端（TypeScript/React）

- TypeScript 严格模式，禁止 `as any`、`@ts-ignore`、`@ts-expect-error`
- Feature-based 目录结构：每个功能模块自包含
- 所有 API 调用通过 TanStack Query hooks，不在组件中直接 `fetch`
- i18n 命名空间按功能模块划分
- 使用 JSDoc 注释标注复杂函数

```bash
# 提交前检查
npx tsc --noEmit      # 类型检查
npm run lint           # ESLint 检查
npm run build          # 构建验证
```

---

## Pull Request 流程

1. **Fork & Clone** 仓库
2. 从 `main` 创建功能分支：`git checkout -b feat/my-feature`
3. 开发并提交代码（遵循 commit 规范）
4. 确保所有检查通过：
   - 后端：`cargo check` 无错误
   - 前端：`npx tsc --noEmit` 和 `npm run build` 无错误
5. 推送分支并创建 Pull Request
6. PR 标题遵循 commit 规范格式
7. 等待 CI 通过和代码审查

### PR Checklist

- [ ] 代码遵循项目编码规范
- [ ] 新功能有对应的文档更新
- [ ] 无类型错误（`cargo check` / `tsc --noEmit`）
- [ ] 无 lint 警告
- [ ] 数据库变更包含 migration 文件
- [ ] API 变更已更新相关 DTO 和 handler 文档注释

---

## 问题反馈

- 使用 GitHub Issues 提交 Bug 报告或功能建议
- Bug 报告请包含：复现步骤、预期行为、实际行为、环境信息
- 功能建议请说明：使用场景、期望效果、替代方案

---

## 项目架构速览

```
请求流转：HTTP → CORS → TraceLayer → AuthMiddleware → RBAC → Handler → Service → Repository → SQLite
```

详细架构说明请参考：
- [详细设计文档](../docs/详细设计文档.md) 或 [detailed-design.en.md](../docs/detailed-design.en.md)
- [后端 AGENTS.md](../backend/AGENTS.md)
- [前端 AGENTS.md](../frontend/AGENTS.md)
