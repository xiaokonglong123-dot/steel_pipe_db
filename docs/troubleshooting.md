# 故障排查指南 / Troubleshooting Guide

## 后端问题

### 数据库锁定：`database is locked`

**症状：** API 请求返回 500 错误，后端日志显示 `database is locked`。

**原因：** SQLite 在并发写入时会锁定整个数据库文件。默认连接池大小为 5，高并发写入可能导致锁竞争。

**解决方案：**

1. **确认 WAL 模式已启用**（默认已启用，可检查数据库）：
   ```bash
   sqlite3 data/steel_pipe.db "PRAGMA journal_mode;"
   # 应输出 "wal"，而非 "delete"
   ```

2. **减少并发写入**：如果业务允许，串行化写入操作。

3. **增加 busy_timeout**：在 `DATABASE_URL` 中添加参数：
   ```
   DATABASE_URL=sqlite://./data/steel_pipe.db?mode=rwc&busy_timeout=5000
   ```

4. **检查是否有长时间运行的事务**：确保没有未提交的事务占用锁。

---

### JWT 令牌过期 / 刷新失败

**症状：** 前端频繁跳转到登录页，或显示"令牌已过期"。

**原因：** JWT 过期时间（默认 24 小时）已到，或刷新令牌无效。

**解决方案：**

1. **调整过期时间**：在 `.env` 中修改 `JWT_EXPIRY_HOURS`：
   ```
   JWT_EXPIRY_HOURS=72    # 3 天
   ```

2. **检查客户端时间**：确保客户端与服务器时间同步，JWT 依赖时间戳。

3. **刷新令牌轮换**：每次刷新会签发新令牌，旧令牌立即失效。如果多个标签页同时刷新，可能出现竞争。前端已处理此场景（401 自动登出）。

4. **重新登录**：最简单的解决方案，重新登录获取新令牌。

---

### 迁移失败：`Failed to run database migrations`

**症状：** 服务器启动失败，日志显示迁移错误。

**原因：** 数据库 schema 与迁移文件不匹配，或迁移文件被修改。

**解决方案：**

1. **查看具体错误**：启动时日志会显示哪个迁移失败。

2. **检查迁移状态**：
   ```bash
   sqlite3 data/steel_pipe.db "SELECT * FROM _sqlx_migrations ORDER BY version;"
   ```

3. **从备份恢复**：如果有备份，恢复数据库文件后重启服务。

4. **重新初始化**（⚠️ 会丢失所有数据）：
   ```bash
   rm data/steel_pipe.db
   cargo run    # 自动重新运行所有迁移
   ```

---

### 端口被占用：`Failed to bind address`

**症状：** 服务器启动失败，提示地址已被使用。

**解决方案：**

1. **查找占用进程**：
   ```bash
   lsof -i :3000
   # 或
   ss -tlnp | grep 3000
   ```

2. **更换端口**：在 `.env` 中修改 `SERVER_PORT`：
   ```
   SERVER_PORT=3001
   ```

3. **终止占用进程**：
   ```bash
   kill <PID>
   ```

---

## 前端问题

### API 请求 404 / 代理不生效

**症状：** 前端页面加载正常，但 API 请求返回 404。

**原因：** Vite 开发代理未正确配置，或后端未启动。

**解决方案：**

1. **确认后端已启动**：
   ```bash
   curl http://localhost:3000/api/v1/auth/login
   ```

2. **检查 Vite 代理配置**（`frontend/vite.config.ts`）：
   ```typescript
   server: {
     proxy: {
       '/api': {
         target: 'http://localhost:3000',
         changeOrigin: true,
       }
     }
   }
   ```

3. **重启 Vite 开发服务器**：修改 `vite.config.ts` 后需要重启。

4. **检查浏览器网络面板**：确认请求是否发送到了正确的地址。

---

### CORS 错误

**症状：** 浏览器控制台显示 `Access-Control-Allow-Origin` 错误。

**原因：** 后端 CORS 配置仅允许 `http://localhost:5173`。

**解决方案：**

1. **开发环境**：确保通过 `http://localhost:5173` 访问（非 `127.0.0.1` 或其他地址）。

2. **生产环境**：在 `router.rs` 的 `CorsLayer` 中添加实际域名：
   ```rust
   .allow_origin([
       "http://localhost:5173".parse::<HeaderValue>().unwrap(),
       "https://your-domain.com".parse::<HeaderValue>().unwrap(),
   ])
   ```

3. **临时调试**：可将 `CorsLayer::permissive()` 替换当前配置（⚠️ 不安全，仅用于调试）。

---

### 页面白屏 / 构建后路由 404

**症状：** 开发环境正常，生产构建后刷新页面出现 404。

**原因：** SPA 路由需要服务器端配置回退到 `index.html`。

**解决方案：** 在 Nginx 配置中添加：
```nginx
location / {
    try_files $uri $uri/ /index.html;
}
```

---

### Excel 导入失败

**症状：** 上传 Excel 文件后返回错误。

**原因：** 文件格式不匹配模板，或数据校验失败。

**解决方案：**

1. **下载最新模板**：通过 `/api/v1/data-io/templates/{entity_type}` 获取标准模板。

2. **检查文件格式**：仅支持 `.xlsx` 和 `.csv`，不支持 `.xls`（旧格式）。

3. **检查数据格式**：确保日期格式为 `YYYY-MM-DD`，数字列不含文本。

4. **查看操作日志**：`/api/v1/data-io/operation-logs` 记录了导入失败的详细信息。

5. **导入频率限制**：10 次/分钟/IP，超出后返回 429。

---

## 数据库问题

### 数据库文件过大

**症状：** `steel_pipe.db` 文件持续增长，即使删除了数据。

**原因：** SQLite 的软删除（`deleted_at`）和 WAL 模式不会自动回收空间。

**解决方案：**

1. **VACUUM 压缩**（需要停服）：
   ```bash
   sqlite3 data/steel_pipe.db "VACUUM;"
   ```

2. **清理 WAL 文件**：
   ```bash
   sqlite3 data/steel_pipe.db "PRAGMA wal_checkpoint(TRUNCATE);"
   ```

3. **定期维护**：建议每月执行一次 VACUUM。

---

### 查询性能慢

**症状：** 列表页面加载缓慢。

**解决方案：**

1. **确认索引存在**：
   ```bash
   sqlite3 data/steel_pipe.db ".indices"
   ```

2. **查看查询计划**：
   ```bash
   sqlite3 data/steel_pipe.db "EXPLAIN QUERY PLAN SELECT * FROM seamless_pipes WHERE status = 'in_stock';"
   ```

3. **运行 ANALYZE**（更新统计信息，帮助查询优化器）：
   ```bash
   sqlite3 data/steel_pipe.db "ANALYZE;"
   ```

4. **检查是否遗漏了 `deleted_at IS NULL`**：所有查询都应排除软删除记录。

---

## 常用诊断命令

```bash
# 检查后端健康状态
curl -s http://localhost:3000/api/v1/auth/login | head

# 查看数据库表列表
sqlite3 data/steel_pipe.db ".tables"

# 查看数据库大小
ls -lh data/steel_pipe.db

# 查看活跃连接
sqlite3 data/steel_pipe.db "PRAGMA database_list;"

# 查看 WAL 状态
sqlite3 data/steel_pipe.db "PRAGMA wal_checkpoint;"

# 查看后端日志（设置 RUST_LOG 环境变量）
RUST_LOG=debug cargo run    # 详细日志
RUST_LOG=sqlx=warn cargo run  # 仅 SQL 警告
RUST_LOG=steel_pipe_db=info cargo run  # 仅应用信息
```
