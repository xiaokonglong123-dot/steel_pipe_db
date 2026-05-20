# Phase 1 — 后端：管材管理模块 (P0 MVP)

> 基于：`docs/需求文档.md` §3.1, §3.6, §6.2；`docs/详细设计文档.md` §4.2, §5.3.1-2, §6.2

## 任务清单

### 1.1 项目初始化
- [ ] 初始化 Rust 项目 (`cargo init`)，配置 `Cargo.toml` 依赖：
  - axum 0.8+, tokio, serde, serde_json, sqlx (sqlite feature), jsonwebtoken, argon2, validator, tracing, tower-http (cors)
- [ ] 创建目录结构：`src/{domain,handler,service,repository,config,middleware,error}`
- [ ] 实现 `src/main.rs`：Axum router 组装、SQLite 连接池初始化（WAL 模式）、启动 tracing 日志
- [ ] 实现 `src/config/mod.rs`：从环境变量 / `.env` 读取配置（DB 路径、JWT 密钥、服务端口）

### 1.2 数据库迁移
- [ ] 创建 `migrations/` 目录，编写初始 SQL 迁移：
  - `seamless_pipes` 表（含所有字段、索引）
  - `screen_pipes` 表（含所有字段、索引）
  - 初始化 SQL 含 `PRAGMA journal_mode = WAL` 等配置
- [ ] 实现迁移自动执行逻辑（启动时自动运行）

### 1.3 领域层 (Domain)
- [ ] 定义 `SeamlessPipe` 结构体（对应 `seamless_pipes` 表字段）
- [ ] 定义 `ScreenPipe` 结构体（对应 `screen_pipes` 表字段）
- [ ] 定义枚举：`PipeType` (Casing/Tubing), `PipeStatus` (InStock/Outbound/Scrapped), `EndType` (SC/LC/BC/X), `ScreenType` (WireWrapped/Slotted/Punched/MetalFelt)
- [ ] 定义 DTO 结构体：`CreateSeamlessPipeDto`, `UpdateSeamlessPipeDto`, `CreateScreenPipeDto`, `UpdateScreenPipeDto`
- [ ] 定义筛选参数结构体：`SeamlessPipeFilter`, `ScreenPipeFilter`（含可选查询字段 + 分页 + 排序）
- [ ] 定义统一响应类型：`ApiResponse<T>`, `PaginatedResponse<T>`, `ApiError`
- [ ] 实现 `Validator` derive 校验注解（必填字段、范围校验）

### 1.4 仓库层 (Repository)
- [ ] 实现 `SeamlessPipeRepo`：
  - `create(dto) -> SeamlessPipe`
  - `update(id, dto) -> SeamlessPipe`
  - `delete(id)`（软删除）
  - `find_by_id(id) -> Option<SeamlessPipe>`
  - `find_by_pipe_number(number) -> Option<SeamlessPipe>`（唯一性校验用）
  - `list(filter) -> PaginatedResult<SeamlessPipe>`（多条件组合筛选 + 排序 + 分页）
- [ ] 实现 `ScreenPipeRepo`（同上，针对筛管表操作）

### 1.5 服务层 (Service)
- [ ] 实现 `PipeService`：
  - `create_seamless_pipe(dto)`：校验 + 唯一性检查 + 调用 Repo + 记录操作日志
  - `update_seamless_pipe(id, dto)`：检查存在性 + 更新 + 日志
  - `delete_seamless_pipe(id)`：检查库存状态（在库才可删）+ 软删除 + 日志
  - `get_seamless_pipe(id)`：获取详情（含库位信息 JOIN）
  - `list_seamless_pipes(filter)`：条件查询 + 分页
  - `create_screen_pipe(dto)` / `update_screen_pipe` / `delete_screen_pipe` / `get_screen_pipe` / `list_screen_pipes`
  - `generate_pipe_number(pipe_type, grade, od, wt)`：按格式自动生成唯一编号
  - `validate_pipe_number_unique(number)`：校验编号唯一性
  - `search_pipes(query)`：跨表模糊搜索（pipe_number / heat_number / serial_number）

### 1.6 处理器层 (Handler)
- [ ] 实现无缝钢管 REST 端点：
  - `GET /api/v1/seamless-pipes` — 列表（query params: q, grade, pipe_type, od_min, od_max, wt_min, wt_max, status, location_id, manufacturer, sort_by, sort_order, page, page_size）
  - `POST /api/v1/seamless-pipes` — 新增
  - `GET /api/v1/seamless-pipes/{id}` — 详情
  - `PUT /api/v1/seamless-pipes/{id}` — 全量更新
  - `DELETE /api/v1/seamless-pipes/{id}` — 删除
- [ ] 实现筛管 REST 端点（同上，增加 screen_type / slot_size 等筛管参数）
  - `GET /api/v1/screen-pipes`
  - `POST /api/v1/screen-pipes`
  - `GET /api/v1/screen-pipes/{id}`
  - `PUT /api/v1/screen-pipes/{id}`
  - `DELETE /api/v1/screen-pipes/{id}`
- [ ] 实现统一搜索端点：
  - `GET /api/v1/pipes/search?q={query}`
- [ ] 实现统一的错误处理（404/400/409/500 映射）
- [ ] 实现分页参数提取器

### 1.7 测试
- [ ] 编写 Repo 层单元测试（使用 SQLite :memory: 数据库）
- [ ] 编写 Service 层单元测试（Mock Repo 或集成测试）
- [ ] 编写 Handler 层集成测试（使用 `axum::test`）
- [ ] 测试管材编号唯一性校验
- [ ] 测试组合筛选 + 分页 + 排序

> **依赖**: 无（独立模块）
