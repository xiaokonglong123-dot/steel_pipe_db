# D — 库存流转规格

来源：`tests/test_tubes_routes.py`、`test_batch_stockout.py`、`test_concurrent_stockout.py`、
`test_used_count.py`、`test_quality.py`，旧实现 `tube_service.py` / `tubes.py`。

| # | 规格 | 来源 | 状态 |
| --- | --- | --- | --- |
| D1 | 单根入库：几何非法 → `400`；自定义 pipe_id 重复 → `409`；成功 → `201` | `test_tubes_routes` | ✅ M3 |
| D2 | 出库成功 → `200`，状态 `out_stock` + `stock_out_time` 落库 | 同上 | ✅ M3 |
| D3 | 重复出库 / 调拨中的管出库 → **不写任何业务行**，返回 `409` | 同上（TOCTOU 守卫） | ✅ M3 |
| D4 | 批量出库：单事务、单单据、每根一条审计；任一根不合格 → 整体回滚 | `test_batch_stockout` | ✅ M3 |
| D5 | 批量入参去重（同一 pipe_id 重复出现只算一次） | `test_batch_stockout` | ✅ M3 |
| D6 | 并发出库不超卖：12 路并发写审计仍串成一条链，库存不出现负数 | `test_concurrent_stockout` | ✅ M3（infra 并发测试） |
| D7 | `used_count` 只随 `in_stock` 管增减；漂移可检测并可重算修复 | `test_used_count` | ✅ M3 |
| D8 | 调拨：源库位与目标库位相同 → `400`；目标停用/不存在 → `400`；源非 `in_stock` → `409` | `test_tubes_routes` | ✅ M3 |
| D9 | 问题管出库/调拨需 admin（operator → `409`/`403`）；报废管一律禁止 | `test_quality` | ✅ M3 |
| D10 | 质量标注只改标记与备注，不动状态；写一条 `quality_mark` 审计 | `test_quality` | ✅ M3 |
| D11 | 软删：记录保留、`deleted_at` 落库；默认列表不含，`include_deleted=true` 可查 | D6（旧缺陷） | ✅ M3 |
| D12 | 软删在库管要释放 `used_count` | 本实现补强 | ✅ M3 |
| D13 | 管子详情含类型扩展参数（无缝/筛管）+ 流转历史 | D14（旧缺陷） | ✅ M3 |
| D14 | 单据明细 `pipe_id` 无 FK：删管不得破坏历史单据 | 设计 §4.5 | ✅ M3（schema） |
| D15 | 库位停用后不能入库、不能作为调拨目标；可重新启用（旧实现缺口） | D15 / D18 | ✅ M3 |
| D16 | 入库必须匹配 `standard_specs` 中的钢级，否则拒绝（不得静默跳过合规） | 设计 §11 | ✅ M3 |

## 与旧实现的差异

1. **空批次**：旧实现未定义；新实现返回 `400`（`EmptyBatch`）。
2. **软删释放 used_count**：旧实现软删后计数残留，属缺陷，已修。
3. **停用库位校验**：旧实现允许向停用库位入库/调拨，新实现在服务层拒绝。
