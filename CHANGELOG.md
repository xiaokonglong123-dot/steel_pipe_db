# CHANGELOG

当前版本线为 Rust 单二进制实现。旧 Python（FastAPI + PyO3）版本记录已随旧栈资料移除，
可在 git 历史中追溯。

## v0.1.0 — 2026-11-17

### Rust 重写

- 全部业务迁入 Rust workspace（domain / compliance / audit / auth / infra / service / api / server）
- 单二进制（内嵌前端）+ 单 SQLite 文件；无 Python、无 Node 运行时
- 审计链、API 5CT 合规、argon2id + HS256 与旧实现行为一致（黄金向量逐字节校验）
- 118 项测试；CI 覆盖 fmt / clippy / 分层断言 / 契约漂移

### 前端

- SPA：Vite 8 + React 19 + TanStack Query + react-router
- UI v2：高密度暗色数据网格、三档密度、后端白名单排序、审计虚拟滚动
- UI v3：石墨指挥台（68px 图标轨 + 命令条、主次 KPI、确认对话框与 Toast、全站状态三重编码）
  - 见 [`docs/2026-11-17-ui-redesign-v3.md`](docs/2026-11-17-ui-redesign-v3.md)

### 仓库整理

- 移除旧 Python 栈文档与阶段计划
- 移除旧构建产物（egg-info、rust-core/target）与本地缓存
- 补充 `docs/screenshots/` 界面截图
