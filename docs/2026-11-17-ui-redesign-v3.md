# 前端重设计记录（UI v3 · 石墨指挥台）

> 日期：2026-11-17　范围：`web/` 全部页面与组件；后端接口、`openapi.json` 类型未变

UI v2（高密度暗色数据网格）交付后判断为「功能齐、表现弱」：8 个等宽 KPI 无主次、
全站同一套 1px 边框与 2px 圆角导致卡片/表格/输入框无层次、172px 常驻筛选列挤压表格、
表单靠 `.toolbar` 拼贴、破坏性操作依赖 `window.confirm/prompt`。本轮推倒表现层重做。

## 1. 方向

**石墨指挥台**（用户三选一确认）：深色石墨底、左侧 68px 图标轨、顶部命令条、
钢蓝强调色、KPI 分主次、工业监控感。不引组件库 / Tailwind / 图标库，维持
「无 CDN、构建期内嵌单二进制」的约束。

## 2. 设计系统

**tokens**（`web/src/styles.css`）

```text
底色      --bg #0a0d11 · --surface #101419 · --surface-2 #151a21 · --surface-3 #1a2028
线        --line #232a34 · --line-2 #2e3743
文字      --text #e7ebf1 · --muted #9aa5b5 · --dim #6b7686
强调      --accent #5b9dff（钢蓝）· --accent-2 #8cc0ff
语义      ok #46d39a · warn #f0b85c · danger #ff7a7a · info #67d3f0
尺度      --topbar-h 56 · --rail-w 68 · --row-h 42 · --ctrl-h 34
圆角      --r 8 · --r-card 12 · --r-pill 999
密度      紧凑 36px / 舒适 42px（默认）/ 宽松 48px，写入 <html data-density>
```

**无障碍约束**

- 状态仍是三重编码：字形（●◐○⊘▲✕✓↔+）+ 中文 + 颜色；颜色不单独承载语义
- 所有交互元素有 `:focus-visible` 2px 描边
- `prefers-reduced-motion: reduce` 下动画与过渡全部关闭
- 侧栏图标下常驻中文标签，不依赖图标单独识别

## 3. 布局与组件

| 区域 | v2 | v3 |
| --- | --- | --- |
| 外壳 | 46px 顶栏 + 200px 文字侧栏 | 68px 图标轨（图标 + 10.5px 中文标签、分组分隔、盘点异常红点）+ 56px 顶栏（品牌 · 命令入口 · 密度 · 身份） |
| KPI | 8 个等宽格子 | 1 张 hero（在库 + 容量条）+ 3 张次卡；今日流转独立卡 |
| 仪表盘 | KPI 条 + 左筛选列 + 最近审计表 | hero KPI + 库位占用列表 + 今日流转 + 最近活动时间线 + 审计链摘要 |
| 筛选 | 172px 常驻左栏 | 顶部状态 chips（带计数）+ 质量/库位/钢级下拉 |
| 表格 | 34px 行高、统一边框 | 42px 行高、表头 12px、悬停底色、选中左侧 3px 竖线、整行可点 |
| 表单 | `.toolbar` 拼贴 | 分组卡片（身份/来源/存放），`form-grid` 自适应列 |
| 对话框 | `window.confirm/prompt` | `ConfirmDialog` / `PromptDialog`（Esc、遮罩点击关闭、pending 态） |
| 反馈 | 仅内联错误 | 内联 `Problem` + 右下 `Toast`（成功/失败，自动消失） |
| 登录 | 居中 330px 卡片 | 左品牌视觉（网格纹理 + 事实块）+ 右登录表单 |
| 详情 | KV 列表 + 操作行内堆叠 | 左信息卡 + 流转历史，右操作卡 + 库位状态，sticky |

新增文件：`components/icons.tsx`（内联 SVG 线图标）、`lib/format.ts`（时间格式化）。
`ui.tsx` 新增 `ToastProvider/useToast`、`ConfirmDialog`、`PromptDialog`、`Notice`、`SectionTitle`。

## 4. 页面改动

- **仪表盘**：KPI 分主次；库位占用逐条进度；最近活动竖线时间线；审计链 tip 完整展示
- **管子**：筛选上移；整行点击进详情；出库改确认对话框；批量坞显示原子操作说明
- **管子详情**：两栏；出库/软删走确认；调拨从自由输入改为库位下拉（排除当前库位）
- **盘点详情**：主区在左、盘点单与确认区在右；结论 chips；确认/取消均走确认对话框
- **入库**：三张分组卡；筛管字段按类型切换
- **导入**：文件拖放/选择区 + 粘贴区 + 结果统计与失败行表
- **库位 / 规格 / 用户**：新增表单卡 + 状态徽章；删除/停用/改密走对话框与 Toast
- **审计**：虚拟滚动保留；action 用 chip 标记；哈希截断 + `title` 全文

## 5. 验收（实测）

```text
tsc --noEmit                        通过
vite build                          CSS 25.32 kB（gzip 5.97）/ JS 377.84 kB（gzip 112.13）
1440×900 逐页截图                   10 页 + 登录 + 盘点详情 + 1024 / 760 两档
水平溢出                            8 个页面 scrollWidth ≤ innerWidth（无横向滚动）
cargo build --release --workspace   21.47s，二进制 8.7 MB（内嵌新 dist）
GET /                               index-DBHqa7AI.js + index-CU74IMfB.css（新 hash）
cargo test --workspace              118 passed / 0 failed
```

## 6. 已知取舍

1. 深色单主题，未做浅色切换（用户深色优先）；token 已分层，可加 `[data-theme]`
2. 未引字体文件：西文走系统 Inter 可用与否，中文走 Noto Sans SC / 系统字体
3. 表格仍是 25 行服务端分页，未做窗口化（审计页除外）
4. 行内操作在窄屏下折行；760px 以下表格横向滚动