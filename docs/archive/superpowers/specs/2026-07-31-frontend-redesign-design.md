# 前端重新设计规格 — 深色工业控制台风

**日期**: 2026-07-31
**项目**: Ayanami_Rei (v0.2.2) — API 5CT 油套管库存管理系统
**范围**: 纯前端外观（Jinja2 模板 + CSS），后端零改动

---

## 1. 背景与目标

现状为 Bootstrap 5 + "iCloud Finder 风格"浅色主题（白底、靛蓝 accent、玻璃拟态顶栏）。本次重新设计：

- **目标 1**: 贴合工业/石油管材行业气质 —— 深色"控制室"（SCADA 面板）风格，符合长时间盯屏的仓库操作场景
- **目标 2**: 强化数据可读性 —— 表格是本系统的核心交互载体，深色下要保证高对比、高密度、状态一眼可辨
- **目标 3**: 零后端改动 —— 全部改动落在 `src/templates/`，保持 Bootstrap 5 类名与表单/路由逻辑不变

## 2. 设计方向（已获用户批准）

**深色工业控制台风（Dark Industrial Control Room）**，灵感来自 SCADA / 工控面板：
近黑蓝基底 + 钢蓝主色 + 琥珀警示色 + 1px 边框面板化组件 + 等宽字体数据呈现。

## 3. 设计系统

### 3.1 色板（CSS 变量，全部重定义于 `base.html`）

| 变量 | 浅色旧值 | 深色新值 | 用途 |
|---|---|---|---|
| `--bg-base` | `#f9fafb` | `#0b1220` | 页面基底（近黑蓝） |
| `--bg-elevated` | `#ffffff` | `#111a2c` | 面板/卡片/表单控件 |
| `--bg-subtle` | `#f3f4f6` | `#0e1729` | 表头/次级面 |
| `--bg-hover` | `#f1f5f9` | `#182338` | 行/项 hover |
| `--border` | `#e2e8f0` | `#1e2a45` | 常规边框 |
| `--border-strong` | `#cbd5e1` | `#2c3a5e` | 强调边框/hover 边框 |
| `--border-focus` | `#6366f1` | `#3b82f6` | 输入聚焦描边 |
| `--text-strong` | `#0f172a` | `#e6edf7` | 标题/关键值 |
| `--text` | `#1e293b` | `#c3cfdf` | 正文 |
| `--text-muted` | `#475569` | `#8fa3bf` | 次级文字 |
| `--text-faint` | `#94a3b8` | `#5c7194` | 弱化文字（时间戳、序号） |
| `--text-disabled` | `#cbd5e1` | `#3d4d6e` | 禁用态 |
| `--accent` | `#4f46e5` | `#3b82f6` | 主色（钢蓝） |
| `--accent-hover` | `#4338ca` | `#60a5fa` | 主色 hover（深底上变亮） |
| `--accent-soft` | `#eef2ff` | `#16233f` | 主色软底（选中/徽章底） |
| `--accent-soft-strong` | `#c7d2fe` | `#1e3a6e` | 主色软底加强 |
| `--success` | `#10b981` | `#34d399` | 成功（在库/正常/通过） |
| `--success-soft` | `#ecfdf5` | `#0f2b21` | 成功软底 |
| `--warning` | `#f59e0b` | `#fbbf24` | 警示（待检/调拨中/问题管） |
| `--warning-soft` | `#fffbeb` | `#2b2110` | 警示软底 |
| `--danger` | `#ef4444` | `#f87171` | 危险（报废/未通过/删除） |
| `--danger-soft` | `#fef2f2` | `#2b1216` | 危险软底 |
| `--info` | `#6366f1` | `#38bdf8` | 信息（筛管类型） |

**原则**：
- 软底色（soft）在深色下成为"深色玻璃底色"，徽章文字用亮色系文字（如 `#7dd3fc`、`#86efac`）
- 主色 hover 方向反转：浅色主题 hover 变深，深色主题 hover 变亮
- 所有阴影改为深色下几乎不可见的 `rgba(0,0,0,0.3)` 微弱投影，主要靠边框分层

### 3.2 字体

- 正文：保留系统栈（PingFang SC / Microsoft YaHei 等），字号维持 14px
- **数据**（pipe_id、序号、统计数字、时间戳、长度/壁厚数值）：`"SF Mono", "JetBrains Mono", "Menlo", monospace` 等宽字体 + `font-variant-numeric: tabular-nums` —— 深色下等宽数据是"仪表感"的关键
- 表头沿用大写 + 字距放大 + 弱化色

### 3.3 组件规则（深色适配）

| 组件 | 规则 |
|---|---|
| 面板 `.surface` | `--bg-elevated` 底 + 1px `--border` 边框 + 10-12px 圆角；顶部可用 1px 亮边框（`rgba(255,255,255,0.06)`）模拟顶光 |
| 按钮 `.btn-accent` | 钢蓝实底白字；hover 亮一档。`.btn-subtle` 深色面板底 + 边框。`.btn-ghost` 透明、hover 主色软底。`.btn-danger-subtle` 深红软底 |
| 表格 | 表头 `--bg-subtle` + 弱化大写文字；行 hover `--bg-hover`；`code` 徽标改为主色亮字（`#93c5fd`）深蓝底 |
| 徽章 | 保留 soft 底 + 描边 + 亮色文字模式（语义色 + 30% 亮度的文字色） |
| 表单 | 控件深色底 + 聚焦主色描边 + `rgba(59,130,246,0.2)` 光晕 |
| 滚动条 | 轨道透明，滑块 `--border-strong`，hover 亮化 |
| 选中文本 | `rgba(59,130,246,0.35)` 底 + 亮蓝文字 |
| 顶栏 | 去掉玻璃拟态：纯 `#0d1526` 底 + 底部 1px 边框；active 项主色软底 + 主色文字；品牌 avatar 保留渐变但换钢蓝系 |
| Flash 消息 | danger/success 深色软底 + 亮色文字 |

## 4. 页面级改动（13 个模板）

| 模板 | 改动要点 |
|---|---|
| `base.html` | **核心**：重写全部 CSS 变量 + 组件样式；顶栏深色化；保留全部结构（block 名、导航项、用户 chip、CSRF 表单） |
| `auth/login.html` | 深色背景 + 居中面板；背景加细网格/图纸感技术纹（CSS `linear-gradient` 网格线，`rgba(255,255,255,0.03)`）；品牌 avatar 钢蓝渐变 |
| `dashboard/index.html` | 统计卡改为"仪表读数"感：等宽大数字 + 单位；图标气泡深色软底；快速操作卡 hover 主色描边；审计事件流保留时间线感 |
| `warehouse/tubes_list.html` | 表格核心页：等宽 pipe_id、徽章高对比、筛选条深色化；空状态图标弱化 |
| `warehouse/stock_in.html` / `stock_out.html` / `transfer.html` | 表单页：统一深色控件、前缀组（input-group-text）深色化、只读信息区面板化 |
| `warehouse/import.html` | 上传区深色化（虚线边框拖放区可用 `--border-strong`） |
| `audit/audit_logs.html` | 审计链页：等宽哈希/序号、验证结果徽章、深色表格 |
| `users/user_list.html` / `user_form.html` | 用户表 + 表单深色化，角色徽章复用语义色 |
| `standards/list.html` / `edit.html` | 标准管理深色化，合规状态徽章复用语义色 |

**通用规则**：
- 凡涉及硬编码颜色（如 `#b91c1c`、`#047857`、`#b45309` 等旧浅色语义文字）一律替换为深色版（如 `#fca5a5`、`#86efac`、`#fcd34d`）
- 页头 `h1` 保持粗体深色（`--text-strong`），可保留各页现有图标点缀
- 不要改动任何 Jinja2 逻辑、表单字段名、路由、按钮的 `href`/`name`/`method`/CSRF 隐藏域

## 5. 验收标准

1. `uvicorn src.main:app --port 8000 --workers 1` 启动正常，所有页面 200
2. 13 个模板全部深色化，无残留白底/浅色硬编码色块
3. 登录 → 仪表盘 → 列表 → 各操作表单 → 审计 → 用户/标准 全流程视觉一致
4. 表格/徽章/表单在深色下对比度合格（状态一眼可辨：在库=绿、待检=琥珀、报废=红、调拨中=琥珀）
5. 响应式（窄屏）下统计卡 2 列、操作卡 2 列的行为保持
6. 无任何后端/路由/Jinja2 逻辑改动（`git diff` 仅涉及 `src/templates/`）

## 6. 实施约束

- 只改 `src/templates/` 下文件；不动 `src/` 下的 Python 代码、不动 `migrations/`、不动 `rust-core/`
- Bootstrap 5.3.3 与 Bootstrap Icons 继续走 CDN，不新增依赖
- 样式继续采用 `base.html` 内联 `<style>` + 各模板 `{% block styles %}` 的既有组织方式
- 实现后人工验证方式：起 dev server 逐页目检（或截图），确认无浅色残留
