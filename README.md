# 钢管原料进出入库管理系统

本项目包含多个独立实现的钢管库存管理系统，按技术栈分类存放于 `projects/` 目录下。

## 项目结构

```
steel_pipe_db/
├── projects/
│   ├── python-tkinter/        # Python Tkinter 桌面应用
│   │   ├── main.py            #   主程序
│   │   ├── config.ini         #   配置文件
│   │   ├── requirements.txt   #   依赖
│   │   └── 钢管原料进出入库系统.bat  # Windows 启动脚本
│   │
│   ├── rust-axum-vue3/        # Rust (axum) + Vue 3 Web 应用 (推荐)
│   │   ├── server/            #   Rust axum 后端
│   │   │   ├── src/
│   │   │   │   ├── main.rs    #     服务器入口和 API 路由
│   │   │   │   └── db.rs      #     SQLite 数据库层
│   │   │   └── Cargo.toml
│   │   └── web/               #   Vue 3 前端 (苹果风格 UI)
│   │       ├── src/
│   │       └── vite.config.js
│   │
│   ├── flask-web/             # Flask + Vanilla JS Web 应用
│   │   ├── backend/           #   Flask 后端
│   │   │   ├── app.py
│   │   │   └── requirements.txt
│   │   ├── frontend/          #   原生 JS 前端
│   │   │   └── index.html
│   │   └── windows/           #   Windows 一键启动脚本
│   │
│   └── rust-egui/             # Rust egui 桌面应用 (旧版)
│       ├── src/
│       └── Cargo.toml
│
├── testdata/                  # 测试数据 (CSV 格式)
└── test.txt                   # 测试文件
```

## 各项目说明

### 1. Rust axum + Vue 3 (推荐)

现代化 Web 应用，功能最完整。

| 技术 | 说明 |
|------|------|
| 后端 | Rust + axum + tokio + rusqlite |
| 前端 | Vue 3 + Vue Router + Axios |
| 数据库 | SQLite3 |
| UI | Apple Design 风格 |

**启动方式:**
```bash
# 启动后端
cd projects/rust-axum-vue3/server
cargo run --release

# 启动前端开发服务器
cd projects/rust-axum-vue3/web
npm install && npm run dev
```

### 2. Python Tkinter 桌面应用

轻量级桌面应用，无需浏览器即可使用。

| 技术 | 说明 |
|------|------|
| 语言 | Python 3 |
| GUI | Tkinter |
| 数据库 | SQLite3 |
| 配置 | config.ini |

**启动方式:**
```bash
cd projects/python-tkinter
python main.py
```

### 3. Flask + Vanilla JS Web 应用

简易 Web 应用，适合快速部署。

| 技术 | 说明 |
|------|------|
| 后端 | Flask + Flask-CORS |
| 前端 | 原生 JavaScript |
| 数据库 | SQLite3 |

**启动方式:**
```bash
cd projects/flask-web
pip install -r backend/requirements.txt
python backend/app.py
```

### 4. Rust egui 桌面应用 (旧版)

基于 egui 的 Rust 桌面应用，已不再维护。

## 测试数据

测试用 CSV 文件位于 `testdata/` 目录，包含:
- `pipes_test.csv` - 正常导入数据
- `pipes_error.csv` - 错误测试数据
