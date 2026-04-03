# 钢管原料进出入库管理系统

基于 Rust (axum) + Vue 3 + SQLite3 的现代化库存管理系统。

## 项目结构

```
steel_pipe_db/
├── server/          # Rust axum 后端
│   ├── src/
│   │   ├── main.rs  # 服务器入口和API路由
│   │   └── db.rs    # 数据库层 (SQLite3)
│   └── Cargo.toml
├── web/             # Vue 3 前端 (苹果风格UI)
│   ├── src/
│   │   ├── api/     # API请求封装
│   │   ├── router/  # 路由配置
│   │   ├── views/   # 页面组件
│   │   └── components/  # 公共组件
│   └── vite.config.js
└── rust_version/    # 旧版 egui 桌面应用 (保留)
```

## 启动方式

### 1. 启动后端

```bash
cd server
cargo run --release
```

后端运行在 `http://localhost:3000`，同时提供API和静态文件服务。

### 2. 启动前端（开发模式）

```bash
cd web
npm install
npm run dev
```

前端开发服务器运行在 `http://localhost:5173`，API请求自动代理到后端。

### 3. 生产部署

```bash
# 构建前端
cd web && npm run build

# 启动后端（直接服务前端静态文件）
cd ../server && cargo run --release
```

访问 `http://localhost:3000` 即可使用完整应用。

## 功能特性

- **首页概览**: 统计卡片、最近操作、材质统计、库存预警
- **钢管入库**: 表单录入，支持自动更新已有库存
- **钢管出库**: 库存校验，不足时自动提示
- **库存查询**: 搜索、筛选、分页、编辑、删除
- **出入库记录**: 按类型/编号/日期筛选，支持CSV导出
- **数据统计**: 材质分类柱状图、入库出库对比、库存预警阈值
- **数据导入**: CSV批量导入
- **数据导出**: 库存/记录CSV导出

## 技术栈

| 层 | 技术 |
|---|---|
| 后端 | Rust + axum + tokio |
| 数据库 | SQLite3 (rusqlite) |
| 前端 | Vue 3 + Vue Router + Axios |
| UI风格 | Apple Design (SF字体、毛玻璃、圆角卡片) |
