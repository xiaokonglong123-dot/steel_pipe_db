# 前端重设计记录（UI v2 · 高密度暗色数据网格）

> 日期：2026-11-17　范围：`web/` 全部页面 + 新增 `GET /api/stats`

## 1. GitHub 调研（2026-11-17 实时数据）

| 候选 | ★ | 最近推送 | 许可 | 结论 |
| --- | ---: | --- | --- | --- |
| shadcn-ui/ui | 128 389 | 当天 | MIT | 组件质量最高，但需引入 Tailwind + Radix，且组件源码要进仓库 |
| mantinedev/mantine | 31 952 | 当天 | MIT | 组件最全；带独立样式系统，会与现有 token 层冲突 |
| TanStack/table | 28 578 | 当天 | MIT | headless 表格逻辑，样式自理；本轮的表格需求（多选/筛选/分页）手写即可 |
| refinedev/refine | 36 048 | 2026-09 | MIT | CRUD 框架，会绑死数据层约定（我们已有自研 API 与审计语义） |
| satnaing/shadcn-admin | 16 739 | 2026-09 | MIT | 现成布局可裁剪，但页面结构偏通用 SaaS，与"数据网格"目标不同 |
| ag-grid/ag-grid | 15 686 | 当天 | **非 MIT**（社区版自有 EULA） | 功能最强，许可与包体（~1 MB）不适合内部小项目 |
| carbon-design-system/carbon | 9 628 | 当天 | Apache-2.0 | 工业高密度风最接近，但 1 093 open issues、全量引入重 |
| tremorlabs/tremor | 3 701 | 2025-10 | Apache-2.0 | 一年未更新，排除 |

结论：**不引组件库**，保留现有手写 CSS + 自有 token；把上表的样式思路（密度、状态编码、表格结构）吸收进来。

## 2. 决策（用户三选）

| 决策 | 选择 | 影响 |
| --- | --- | --- |
| 视觉方向 | 高密度暗色数据网格 | 行高 28px、控件 26px、圆角 2px、无装饰 |
| 落地页 | KPI 仪表盘 + 表格 | 新增 `GET /api/stats` 聚合端点 |
| 组件基座 | 保持手写 CSS | 无新运行时依赖；体积增量 ≈ 12 kB CSS |

## 3. 设计规范

**密度刻度**（`styles.css` 顶部变量）

```text
--row-h: 28px     表格行高
--ctrl-h: 26px    输入/按钮
--r: 2px          圆角
--fs: 12.5px      正文
--fs-sm: 11.5px   表格/控件
--topbar-h: 40px  顶栏
--sider-w: 188px  侧栏
```

**状态三重编码**：字形 + 中文 + 颜色，颜色永不单独承载语义。

| 状态 | 字形 | 颜色 |
| --- | --- | --- |
| 在库 | ● | ok（绿） |
| 调拨中 | ◐ | warn（琥珀） |
| 已出库 | ○ | none（灰） |
| 盘亏 | ⊘ | danger（红） |
| 正常 / 通过 | ● / ✓ | ok |
| 问题管 | ▲ | warn |
| 报废 | ✕ | danger |
| 盘点进行中 / 已确认 / 已取消 | ◔ / ✓ / — | info / ok / none |
| 盘点结论：一致 / 位置异常 / 盘盈 | ✓ / ↔ / + | ok / warn / info |

**布局**：顶栏（品牌 · 命令入口 · 身份）+ 侧栏（分组导航，盘点异常数红标）+ 内容区。
管子页使用 `split`：左侧 172px 筛选列（状态带计数、质量、库位、钢级）+ 右侧表格。
选中行左侧 2px 强调竖线 + 底部 `bulkbar` 操作坞。

**键盘**：`⌘K` / `Ctrl+K` 或 `/` 打开命令面板；面板内 `↑↓` 选择、`Enter` 执行、`Esc` 关闭。
输入非页面名时，按"打开管子 `<输入>`"处理，直接跳详情。

## 4. 新增/改动清单

| 文件 | 变化 |
| --- | --- |
| `crates/service/src/stats.rs` | 新增：KPI 聚合（状态/质量计数、库位汇总、今日单据、最近审计、链尾） |
| `crates/api/src/routes/stats.rs` | 新增：`GET /api/stats` |
| `crates/infra/src/repo/stats.rs` | 新增：分组计数与时间窗口查询 |
| `crates/infra/src/repo/audit.rs` | 新增：`last_n`、`chain_tip` |
| `web/src/styles.css` | 重写：密度刻度、`.mark` 状态字形、`.kpi`、`.split/.filters`、`.bulkbar`、`.cmd-*` |
| `web/src/components/shell.tsx` | 新增：Topbar / SideNav / Kpi / Meter / CommandPalette / useStats |
| `web/src/components/tube-filters.tsx` | 新增：管子筛选列（带计数） |
| `web/src/components/tube-table.tsx` | 新增：高密度表格 + 行内操作 |
| `web/src/pages/DashboardPage.tsx` | 新增：KPI 行 + 库位占用 + 最近审计 |
| `web/src/pages/TubesPage.tsx` | 重写：split 布局、URL 携带筛选、批量坞 |
| `web/src/App.tsx` | 重写：shell 布局 + 全局快捷键 + 命令面板 |
| `web/src/components/ui.tsx` | `StatusBadge` 改为字形编码（保留原 API） |

## 5. 验收

```text
tsc --noEmit                        通过
vite build                          CSS 12.97 kB（gzip 3.23）/ JS 347 kB（gzip 104.9）
cargo build --release               50s（增量），二进制 8.6 MB
GET /  → index-M2qih77v.js + index-CSaQ3uIK.css（新 hash，确认已内嵌）
GET /api/stats                      200：total 1 / in_stock 1 / used 1/100 / 今日入库 1 / 审计 6 条
SPA 回退 / /tubes / /checks / /audit 全部 200 text/html
RSS                                 13.5 MB
cargo test --workspace              117 passed
clippy -D warnings                  零告警
```

## 6. 后续可做（未实现，按需）

1. 表格列排序（TanStack Table 或手写 comparator）
2. 行密度切换（紧凑 24px / 舒适 32px）
3. 盘点页也用 split 布局（当前为纵向流）
4. 审计页虚拟滚动（> 10k 行时）
5. 深色/浅色主题切换（token 已分层，加 `[data-theme=light]` 即可）

## 7. v2.1 增补（同日，按"字太小 + 三项都做"）

| 项 | 变化 |
| --- | --- |
| 字号 | 正文 12.5 → **14px**；表格 11.5 → **13px**；表头/标签 10.5 → **12px**；KPI 数值 20 → **24px**；行高 28 → **34px**；控件 26 → **30px** |
| 密度三档 | 紧凑 28 / 舒适 34（默认）/ 宽松 40，写入 `<html data-density>`，localStorage 持久化，顶栏「密 适 宽」切换 |
| 列排序 | 后端白名单（`pipe_id` / `steel_grade` / `location` / `status` / `quality` / `updated_at`）；非法字段回落 `id`；`id` 作为稳定次序，翻页不抖 |
| 盘点页 | 改 `split`：左侧会话信息 + 结论筛选（带计数）+ 确认/取消，右侧应盘清单（按结论过滤）+ 盘盈登记 + 已登记差异 |
| 审计页 | 虚拟滚动（只渲染视口内行，上下用等高空行占位）+ 游标「加载更多」；行高随密度切换自动适配 |

### 验收

```text
GET /api/tubes?sort=steel_grade&order=asc    → P-0001/L80, P-9002/N80, P-9001/P110
GET /api/tubes?sort=steel_grade&order=desc   → P-9001/P110, P-9002/N80, P-0001/L80
GET /api/tubes?sort=drop_table&order=desc    → 回落 id 顺序（白名单生效）
GET /api/audit?limit=3                        → id 1,2,3
GET /api/audit?limit=3&after_id=4             → id 5,6（游标分页）
CSS 14.07 kB（gzip 3.43） · JS 353.84 kB（gzip 106.8） · RSS 13.7 MB
cargo test --workspace 118 passed（新增 1 项排序白名单 + 分页稳定性）
```

### 一个实现坑（留档）

`ORDER BY` 最初只改了 `format!` 的多行版本，但 `cargo fmt` 已把它压成单行，替换静默失败——编译照样通过（`sort` 字段正确传入但 SQL 仍是 `ORDER BY id`），直到接口实测发现"升序降序结果一样"才暴露。
教训：**排序/分页这类语义必须用接口断言验证**，类型系统查不出"参数传了但没用"。
