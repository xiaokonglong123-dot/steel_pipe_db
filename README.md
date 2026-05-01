# 钢管原料进出入库管理系统

钢材热处理生产全流程管理解决方案，支持热处理工单、取样、写号、质量检测等完整功能。

## 项目结构

```
steel_pipe_db/
├── projects/
│   ├── python-tkinter/        # Python Tkinter 桌面应用
│   │
│   ├── rust-axum-vue3/        # Rust (axum) + Vue 3 Web 应用 (推荐)
│   │   ├── server/            #   Rust axum 后端
│   │   └── web/               #   Vue 3 前端
│   │
│   ├── flask-web/             # Flask + Vanilla JS Web 应用
│   │   ├── backend/           #   Flask 后端
│   │   └── frontend/          #   原生 JS 前端
│   │
│   └── rust-egui/             # Rust egui 桌面应用
│
├── testdata/                  # 测试数据 (CSV 格式)
└── README.md
```

## 技术栈

### Rust + Vue 3 (推荐)

| 层级 | 技术 |
|------|------|
| 后端 | Rust + axum + tokio + rusqlite |
| 前端 | Vue 3 + Vite + Vue Router + Axios |
| 数据库 | SQLite3 (WAL模式) |
| UI风格 | Apple Design |

### 其他实现

- **Python Tkinter**: 轻量级桌面应用，无需浏览器
- **Flask + Vanilla JS**: 简易Web部署方案
- **Rust egui**: 原生桌面应用 (旧版)

## 核心功能

### 库存管理
- 钢管入库/出库
- 批量CSV/Excel导入导出
- 多条件搜索过滤
- 库存统计与报表

### 热处理工单
- 工单创建与管理
- 工艺参数配置
- 温度曲线记录
- 炉况监控

### 取样管理
- 取样记录登记
- 取样位置追踪
- 样品状态管理 (待检测/已完成)

### 写号管理
- 编号登记
- 写号内容记录
- 写号方法选择 (喷印/钢印/标签)
- 状态确认

### 质量检测
- 硬度检测 (HB/HRC)
- 力学性能 (抗拉/屈服/延伸率)
- 金相组织记录
- 检测结果判定

## 快速开始

### Rust + Vue 3 (推荐)

```bash
# 启动后端
cd projects/rust-axum-vue3/server
cargo run --release

# 启动前端 (新终端)
cd projects/rust-axum-vue3/web
npm install
npm run dev
```

访问 http://localhost:5173

### Flask Web

```bash
cd projects/flask-web/backend
pip install -r requirements.txt
python app.py
```

访问 http://localhost:5000

### Python Tkinter

```bash
cd projects/python-tkinter
python main.py
```

## API文档

### 热处理工单

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | /api/heat-treatment-orders | 获取工单列表 |
| POST | /api/heat-treatment-orders | 创建工单 |
| POST | /api/heat-treatment-orders/:id | 更新工单状态 |

### 取样

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | /api/samplings/:order_id | 获取取样记录 |
| POST | /api/samplings | 添加取样 |
| POST | /api/samplings/:id/status | 更新状态 |

### 写号

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | /api/markings/:order_id | 获取写号记录 |
| POST | /api/markings | 添加写号 |
| POST | /api/markings/:id/status | 更新状态 |

### 质量检测

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | /api/quality-inspections/:order_id | 获取检测记录 |
| POST | /api/quality-inspections | 添加检测 |

## 数据库表

### pipes - 钢管库存表
- pipe_id, diameter, thickness, length, material
- quantity, location, supplier, status
- furnace_number, heat_treatment_batch, sample_number

### heat_treatment_orders - 热处理工单
- order_number, pipe_id, furnace_number
- heat_treatment_type, process_parameters
- start_time, end_time, status

### sampling_records - 取样记录
- order_id, sample_number, sampling_position
- sampling_time, sampler, sample_status

### marking_records - 写号记录
- order_id, marking_number, marking_content
- marking_position, marker, marking_method, marking_status

### quality_inspections - 质量检测
- order_id, hardness_hb, hardness_hrc
- tensile_strength, yield_strength, elongation
- result, inspector, inspection_date

## 测试数据

`testdata/` 目录包含:
- `pipes_test.csv` - 标准导入数据
- `pipes_error.csv` - 错误数据测试