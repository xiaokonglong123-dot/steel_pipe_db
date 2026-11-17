# E — API 5CT 合规规格

来源：`tests/test_stock_in_compliance.py`、旧 `rust-core/src/compliance.rs`、设计 §11。

恒定产出 5 条检查，顺序固定；跳过项也占位并在 `reason` 写明原因。

| # | 规格 | 状态 |
| --- | --- | --- |
| E1 | `steel_grade_match`：钢级精确匹配，失败文案 `expected grade 'X', got 'Y'` | ✅ M2 |
| E2 | `wall_thickness_range`：闭区间；仅给 min 或仅给 max 也成立；两者皆空 → 跳过 | ✅ M2 |
| E3 | `outer_diameter_match`：相对偏差 ≤ 1%；恰好 1.00% 通过，1.1% 失败并报百分比 | ✅ M2 |
| E4 | `length_positive`：长度 > 0 | ✅ M2 |
| E5 | `screen_specs_valid`：筛管必须带筛管参数且槽宽/过滤精度 > 0；非筛管跳过 | ✅ M2 |
| E6 | 无匹配标准规格 → 拒绝入库（`422`），不得跳过校验 | ✅ M3 |
| E7 | `force=true` 需 admin，否则 `403`；成功后额外写一条 `forced_intake` 审计 | ✅ M3 |
| E8 | 合规未通过但被强制入库时，`tubes.compliance_pass = false` 必须落库 | ✅ M3 |

## 与旧实现的差异

1. 旧实现允许 Python 侧读不到规格时自行决定；新实现把"无规格即拒绝"写死为 `422`。
2. 强制入库的审计（`forced_intake`）旧实现标注为"未实现"，本轮补齐。
