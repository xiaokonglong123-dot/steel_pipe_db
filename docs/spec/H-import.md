# H — 导入与上传上限规格

来源：`tests/test_import_http.py`（256 行）、`tests/test_upload_limit.py`（205 行）、设计 §17 D7。

| # | 规格 | 状态 |
| --- | --- | --- |
| H1 | 必填列：`pipe_id, pipe_type, outer_diameter_mm, wall_thickness_mm, length_m, steel_grade, manufacturer, location`；缺列 → `400` 且一行不写 | ✅ M5 |
| H2 | 行级部分成功：失败行返回 `{row, pipe_id, error}`，成功行照常入库（非整体回滚） | ✅ M5 |
| H3 | 单次上限 10 000 行：超限整体拒绝并提示分批（`400`），已写行数为 0 | ✅ M5 |
| H4 | UTF-8 BOM 容错 | ✅ M5 |
| H5 | 可选列：`thread_type/api_5ct_cert/batch_no/heat_no/heat_treat_method/quality_mark/quality_note` 与类型扩展列 | ✅ M5 |
| H6 | 导入与单根入库走同一合规门（不通过即该行失败，不静默跳过） | ✅ M5 |
| H7 | `pipe_type=screen_pipe` 缺筛管参数时该行失败（合规第五项） | ✅ M5 |
| H8 | 请求体上限由中间件（`STEELPIPE_MAX_BODY_BYTES`，默认 20 MiB）拦截 → `413` | ✅ M0 中间件 |
| H9 | 导入需要 operator+ 且必须带 CSRF | ✅ M5 |

## 与旧实现的差异

1. 旧实现用 `pandas.read_csv`；新实现用 `csv` crate 流式解析，不引 pandas。
2. 旧实现行级失败只累加错误计数；新实现返回结构化 `failed[]`，前端可直接高亮行号。
3. 上传上限从"路由里手工检查"改为中间件统一拦截（chunked 也拦），语义一致但覆盖面更全。
