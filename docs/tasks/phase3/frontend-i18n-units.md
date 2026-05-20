# Phase 3 — 前端：国际化与单位制切换 (P2)

> 基于：`docs/前端设计文档.md` §3.2, §6

## 任务清单

### 1.1 完整国际化支持
- [ ] 逐模块审查所有中文字段，确保全部提取到 i18n key：
  - `zh/common.json`：通用字段（操作、保存、取消、查询、重置、状态、日期等）
  - `en/common.json`：英文翻译
  - `zh/pipes.json` / `en/pipes.json`：管材字段
  - `zh/inventory.json` / `en/inventory.json`：库存字段
  - `zh/system.json` / `en/system.json`：系统管理字段
  - `zh/quality.json` / `en/quality.json`：质量字段
  - `zh/orders.json` / `en/orders.json`：订单字段
  - `zh/contracts.json` / `en/contracts.json`：合同字段
  - `zh/reports.json` / `en/reports.json`：报表字段
  - `zh/labels.json` / `en/labels.json`：标签字段
  - `zh/validation.json` / `en/validation.json`：校验提示信息
- [ ] 英文翻译准确度审核：确保专业术语（API 5CT 钢级名、管材类型名）保留英文不翻译
- [ ] 替换所有硬编码中文字段为 `useTranslation()` hook

### 1.2 单位制切换
- [ ] 实现 `src/stores/unitStore.ts`（Zustand）：
  - 状态：unitSystem: 'metric' | 'imperial'
  - 操作：toggleUnitSystem, setUnitSystem
- [ ] 实现 `src/shared/utils/unit-convert.ts`：
  - `formatLength(mm, unitSystem)`：mm ↔ inch
  - `formatWeight(kg, unitSystem)`：kg ↔ lb
  - `formatDiameter(mm, unitSystem)`：mm ↔ inch（保留 2 位小数）
  - `formatPressure(MPa, unitSystem)`：MPa ↔ psi
- [ ] 实现 `hooks/useUnit.ts`：
  - 从 unitStore 读取当前单位制
  - 返回格式化函数（自动根据当前单位制转换）
  - 数字附近显示单位后缀
- [ ] 在所有管材规格展示/表格/详情中集成单位转换
- [ ] Header 中保留单位制切换按钮

### 1.3 日期格式化
- [ ] 使用 dayjs locale 切换（zh-cn / en）
- [ ] 全局日期显示格式统一：中文 `YYYY-MM-DD HH:mm` / 英文 `MM/DD/YYYY HH:mm`

### 1.4 测试
- [ ] 切换语言后验证所有页面文案是否正确切换
- [ ] 切换单位制后验证所有数字和单位是否正确转换 + 四舍五入精度

> **依赖**: 全部前端模块（需要修改到每个模块）
