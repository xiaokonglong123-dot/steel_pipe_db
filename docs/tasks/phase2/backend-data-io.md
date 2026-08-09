# Phase 2 — Backend: Data Import/Export Module (P1)

> Based on: `docs/requirements.en.md` §3.5; `docs/detailed-design.en.md` §11
> Architecture: backend crate `erp-server` (Rust + Axum + SQLx 0.8 **sqlite** feature)；DB = SQLite3 at `sqlite://data/erp.db?mode=rwc`. 导入导出对象统一为「商品 (Item) + SKU」+ 库存/订单/合同等通用业务实体；无管材专属字段. 依赖：`calamine` (读 Excel)、`rust_xlsxwriter` (写 Excel)、`csv` (CSV). 错误码 180xx (Data IO). 术语见 `specs/UBIQUITOUS_LANGUAGE_LATEST.md`。

## Tasks

### 1.1 Dependencies

- [ ] Add Cargo deps: `calamine` (read Excel), `rust_xlsxwriter` (write Excel), `csv` (CSV read/write)

### 1.2 Import

- [ ] Implement Excel parser:
  - Read `.xlsx` / `.xls` formats
  - Auto-detect header row and map to struct fields
  - Support zh/en header matching
  - Validate data format and required fields row by row
- [ ] Implement CSV parser:
  - Auto-detect UTF-8 / GBK encoding
  - Auto-detect comma / tab delimiter
- [ ] Implement import Service:
  - `import_items(file)`: 批量导入通用商品主数据 (sku/name/category/unit/spec/...)
  - `import_suppliers(file)`: 批量导入供应商
  - `import_customers(file)`: 批量导入客户
  - `import_inventory_balances(file)`: 批量导入期初库存余额
  - Tx: commit only if all succeed, rollback partial failures and return error rows
  - Generate import result report (success count / failure count + per-row error reason)
  - SKU 重复处理策略 (skip / overwrite / auto-generate new SKU)
- [ ] Implement import endpoints:
  - `POST /api/v1/import/items` — upload file to import items
  - `POST /api/v1/import/suppliers` — upload file to import suppliers
  - `POST /api/v1/import/customers` — upload file to import customers
  - `GET /api/v1/import/template/items` — download items import template
  - `GET /api/v1/import/template/suppliers` — download suppliers import template
  - `GET /api/v1/import/template/customers` — download customers import template

### 1.3 Export

- [ ] Implement Excel generator:
  - Multi-sheet export support
  - Auto-set column widths, header styles (industrial blue theme)
  - Split large data across sheets
- [ ] Implement CSV generator
- [ ] Implement export Service:
  - `export_inventory_report(filter)`: 导出库存报表
  - `export_inbound_detail(filter)`: 导出入库明细
  - `export_outbound_detail(filter)`: 导出出库明细
  - `export_item_list(filter)`: 导出商品主数据列表
  - `export_purchase_orders(filter)`: 导出采购订单
  - `export_sales_orders(filter)`: 导出销售订单
  - Support selectable export fields
  - Support paginated export (stream all data)
- [ ] Implement export endpoints:
  - `POST /api/v1/export/inventory` — export inventory report
  - `POST /api/v1/export/inbound` — export inbound detail
  - `POST /api/v1/export/outbound` — export outbound detail
  - `POST /api/v1/export/items` — export item list
  - `POST /api/v1/export/purchase-orders` — export purchase orders
  - `POST /api/v1/export/sales-orders` — export sales orders
  - (Request body specifies format: excel/csv + filter criteria)

### 1.4 Tests

- [ ] Test import template generation (correct format)
- [ ] Test 10k row import performance (should be ≤ 10s)
- [ ] Test import data validation (missing required fields, format errors, duplicate SKU)
- [ ] Test export file readability (verify by reading back with calamine)

> **Deps**: 商品主数据 (item) 模块、采购模块、销售模块、库存模块 (被引用的实体)
