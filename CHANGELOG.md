# CHANGELOG

本项目版本号跟随 `pyproject.toml` 的 `version`，设计权威见 `docs/2026-07-13-api5ct-inventory-design.md`。

## v0.8.0 — 2026-10-24

### 前端重写

全部 20 个模板（19 个页面 + `base.html`）从零重写，废弃旧页面代码。后端路由与业务语义未变，唯一后端改动是新增静态资源挂载。

- 新增 `src/static/`：`app.css`（token 层 + 组件）、`app.js`（条件编辑器 / 密度 / 批量选择）、`vendor/`（Bootstrap 5.3.3 + Bootstrap Icons 1.11.3 本地拷贝）
- 新增 `src/templates/_macros.html`：状态 / 类型 / 质量 / 合规 / 角色 / 盘点 / 审计动作徽章 + 分页宏。所有状态展示统一走宏，**图标 + 文字 + 颜色**三重编码（色盲模式下颜色不是唯一信息载体）
- 设计方向「工业数据网格」：高密度表格、粘性表头、行左状态轨（形状 + 颜色双重编码）、条件芯片筛选、显式批量模式与操作坞；深色优先
- `main.py` 挂载 `/static`，**不再依赖 jsdelivr CDN**（应用部署在无外网的局域网）
- 模板内不再写 `<style>`，样式只进 `app.css`；`standards/` 拆为独立模板目录
- 修复 `standards/list.html` 分页链接丢失 `grade` 筛选参数
- 清理 `app.css` 中迁移期的兼容别名（`.surface` / `.card` / `.badge.bg-*` / `.btn-secondary` / `.form-text` / `.input-group-text` / `.form-check-label` / `.form-section` / `.chip` / `.text-accent` / `.page-sub`），模板已零引用
- 新增 820px / 560px 响应式断点，390px 视口无横向溢出

### 修复

- **未认证浏览器导航返回裸 JSON**：直接打开 `/` 或 `/tubes` 得到 `{"detail":"not authenticated"}`。新增 `StarletteHTTPException` 处理器，**仅当 `Accept` 含 `text/html`** 时 303 重定向到 `/login`；API / XHR（`fetch` 的 `*/*`、curl）仍返回 401 原文，路由契约与既有测试不变。新增 2 项测试。

### 校验

- `pytest -m "not slow"`：167 passed, 3 deselected；`pytest -m slow`：3 passed
- `ruff check`：零报错
- 端到端冒烟（真实浏览器走完整表单）：未登录重定向 → 登录 → 建标准 J55 → 入库 → 列表 / 详情 → 审计页 → 哈希链校验（完整，2 条）→ 出库，20 项断言全通过，零 console 错误

## v0.7.0 及更早

见 `docs/2026-07-13-api5ct-inventory-design.md` §0.2–§0.10 的历史修订记录。
