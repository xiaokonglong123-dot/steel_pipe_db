# 钢管原料进出入库管理系统

基于 Rust + egui + SQLite 的跨平台桌面应用，用于管理钢管原料的入库、出库、库存查询、数据统计及导入导出。

## 功能特性

### 核心功能
- **钢管入库** - 录入钢管信息（编号、直径、壁厚、长度、材质、数量、存放位置、供应商等），支持重复编号自动累加库存
- **钢管出库** - 根据钢管编号进行出库操作，自动校验库存数量，库存不足时提示
- **库存查询** - 支持关键词搜索、高级筛选（材质、直径范围、长度范围、状态），分页显示
- **出入库记录** - 查看所有操作记录，支持按操作类型、钢管编号、日期范围筛选
- **数据统计** - 总种类、总数量、入库/出库总数统计，按材质分类统计及占比分析
- **库存预警** - 自定义阈值，自动标识库存不足的钢管项
- **首页概览** - 统计卡片、最近操作记录、材质统计、库存预警提示

### 数据管理
- **CSV 导入导出** - 支持粘贴 CSV 内容批量导入，导出库存/记录为 CSV 文件
- **Excel 导入导出** - 支持 .xlsx / .xls 文件批量导入，导出为带格式的 Excel 文件（表头样式、边框、数字格式）
- **操作撤回** - 记录所有操作日志（入库/出库/编辑/删除），支持逐条撤回或一键撤回最近操作
- **编辑/删除** - 在库存查询页面直接编辑钢管信息或删除记录（带确认对话框）

## 技术栈

| 组件 | 技术 |
|------|------|
| GUI 框架 | [egui](https://github.com/emilk/egui) + [eframe](https://github.com/emilk/egui) |
| 数据库 | [SQLite](https://www.sqlite.org/) (via [rusqlite](https://crates.io/crates/rusqlite)) |
| Excel 读取 | [calamine](https://crates.io/crates/calamine) |
| Excel 写入 | [rust_xlsxwriter](https://crates.io/crates/rust_xlsxwriter) |
| 序列化 | [serde](https://crates.io/crates/serde) + [serde_json](https://crates.io/crates/serde_json) |
| 日期时间 | [chrono](https://crates.io/crates/chrono) |
| 错误处理 | [thiserror](https://crates.io/crates/thiserror) |
| 日志 | [tracing](https://crates.io/crates/tracing) + [env_logger](https://crates.io/crates/env_logger) |

## 项目结构

```
rust_version/
├── Cargo.toml          # 项目配置与依赖
├── config.toml         # 应用配置文件
├── src/
│   ├── main.rs         # 程序入口，初始化配置、数据库、启动 GUI
│   ├── app.rs          # GUI 界面逻辑，所有视图与交互
│   ├── database.rs     # 数据库操作层，CRUD、导入导出、撤回、统计
│   └── config.rs       # 配置结构体定义、加载、验证
└── assets/             # 静态资源（图标等）
```

## 快速开始

### 环境要求

- Rust 1.70+ (推荐最新稳定版)
- Linux: `libxcb`, `libssl-dev`, `libsqlite3-dev`
- Windows: MSVC 工具链
- macOS: Xcode Command Line Tools

### 编译运行

```bash
# 克隆项目后进入目录
cd rust_version

# 编译并运行
cargo run

# 或仅编译
cargo build --release
```

### 配置文件

首次运行会自动生成 `config.toml`，可按需修改：

```toml
[database]
path = "pipes.db"

[ui]
window_title = "钢管原料进出入库管理系统"
window_width = 1200
window_height = 800
```

## 使用说明

### 入库操作
1. 点击左侧导航栏「钢管入库」
2. 填写钢管信息（带 * 为必填项）
3. 选择材质，填写数量和存放位置
4. 点击「确认入库」完成操作

### 出库操作
1. 点击「钢管出库」
2. 输入钢管编号和出库数量
3. 系统自动校验库存，库存不足时提示
4. 点击「确认出库」完成操作

### 库存查询
- 顶部搜索框支持关键词搜索
- 「高级筛选」可按材质、直径范围、长度范围、状态筛选
- 表格支持分页，每页 20 条记录
- 每行提供「编辑」和「删除」按钮

### 数据导入
1. 点击左侧「数据导入」
2. 选择导入方式：
   - **CSV 粘贴导入**：粘贴 CSV 内容到文本框
   - **Excel 文件导入**：填写 Excel 文件路径
3. 填写操作员姓名，点击「开始导入」

CSV/Excel 格式要求：
```
钢管编号, 直径(mm), 壁厚(mm), 长度(m), 材质, 数量, 存放位置(可选), 供应商(可选)
```

### 数据导出
1. 点击左侧「数据导出」
2. 选择格式：CSV 或 Excel (.xlsx)
3. 可选设置日期范围
4. 点击「导出库存」或「导出记录」

### 撤回操作
1. 点击左侧「撤回操作」
2. 查看所有操作记录
3. 点击单条记录的「撤回」按钮，或点击「撤回最近一次操作」

## 数据库说明

应用使用 SQLite 作为本地数据库，数据文件默认为 `pipes.db`（位于程序运行目录）。

### 数据表结构

**pipes** - 钢管库存表
| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER | 自增主键 |
| pipe_id | TEXT | 钢管编号（唯一） |
| diameter | REAL | 直径(mm) |
| thickness | REAL | 壁厚(mm) |
| length | REAL | 长度(m) |
| material | TEXT | 材质 |
| quantity | INTEGER | 数量 |
| location | TEXT | 存放位置 |
| supplier | TEXT | 供应商 |
| entry_date | TEXT | 入库日期 |
| last_update | TEXT | 最后更新时间 |
| status | TEXT | 状态（在库/已出库） |

**inventory_records** - 出入库记录表
| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER | 自增主键 |
| pipe_id | TEXT | 钢管编号 |
| operation_type | TEXT | 操作类型（入库/出库） |
| quantity | INTEGER | 数量 |
| operation_date | TEXT | 操作日期 |
| operator | TEXT | 操作员 |
| remarks | TEXT | 备注 |

**operation_logs** - 操作日志表（用于撤回功能）
| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER | 自增主键 |
| operation_type | TEXT | 操作类型 |
| target_type | TEXT | 目标类型 |
| target_id | TEXT | 目标ID |
| snapshot_before | TEXT | 操作前快照(JSON) |
| snapshot_after | TEXT | 操作后快照(JSON) |
| operator | TEXT | 操作员 |
| timestamp | TEXT | 时间戳 |
| remarks | TEXT | 备注 |

## 开发

```bash
# 开发模式运行
cargo run

# 发布模式编译（优化）
cargo build --release

# 运行测试
cargo test

# 代码检查
cargo clippy

# 格式化代码
cargo fmt
```

## 许可证

MIT
