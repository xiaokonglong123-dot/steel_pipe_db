# Steel Pipe DB API 文档

> API 5CT 无缝钢管与筛管库存管理系统 — 后端 REST API 参考

**Base URL:** `http://localhost:3000/api/v1`

---

## 目录

- [认证机制](#认证机制)
- [RBAC 角色模型](#rbac-角色模型)
- [统一响应格式](#统一响应格式)
- [分页格式](#分页格式)
- [错误码速查表](#错误码速查表)
- [接口一览](#接口一览)
  - [认证与用户管理](#1-认证与用户管理)
  - [无缝钢管管理](#2-无缝钢管管理)
  - [筛管管理](#3-筛管管理)
  - [库存管理（入库/出库/库存/库位/盘点/统计/追溯）](#4-库存管理)
  - [供应商管理](#5-供应商管理)
  - [客户管理](#6-客户管理)
  - [采购订单](#7-采购订单)
  - [销售订单](#8-销售订单)
  - [质量证书](#9-质量证书)
  - [合同管理](#10-合同管理)
  - [报告与仪表盘](#11-报告与仪表盘)
  - [标签打印](#12-标签打印)
  - [数据导入导出](#13-数据导入导出)
  - [ATP 可用库存检查](#14-atp-可用库存检查)
  - [全局搜索](#15-全局搜索)
  - [个人信息](#16-个人信息)
  - [健康检查](#17-健康检查)

---

## 认证机制

系统采用 **JWT + Refresh Token** 双令牌认证。

### 登录流程

```
POST /api/v1/auth/login
Content-Type: application/json

{
  "username": "admin",
  "password": "admin123"
}
```

**成功响应 (200):**

```json
{
  "success": true,
  "request_id": "req_550e8400-e29b-41d4-a716-446655440000",
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIs...",
    "token_type": "Bearer",
    "expires_in": 7200,
    "user": {
      "id": 1,
      "username": "admin",
      "role": "admin",
      "display_name": "系统管理员"
    }
  }
}
```

同时，响应头中设置 `Set-Cookie`：

```
refresh_token=<token>; Path=/api/v1/auth; HttpOnly; SameSite=Strict; Max-Age=2592000
```

### 请求认证

所有需要认证的接口请在请求头中携带 Access Token：

```
Authorization: Bearer <access_token>
```

### 刷新令牌

Access Token 过期后，使用 Refresh Token 获取新的令牌对：

```
POST /api/v1/auth/refresh
Cookie: refresh_token=<refresh_token>
```

返回新的 `access_token` 和 `refresh_token`，并更新 Cookie。

### 登出

```
POST /api/v1/auth/logout
Authorization: Bearer <access_token>
```

撤销所有 Refresh Token 并清除 Cookie。

### 获取当前用户信息

```
GET /api/v1/auth/me
Authorization: Bearer <access_token>
```

---

## RBAC 角色模型

| 角色 | 说明 | 权限范围 |
|------|------|---------|
| `admin` | 系统管理员 | 所有功能，包括用户管理 |
| `warehouse` | 仓库管理员 | 钢管增删改、入库出库、库位、盘点 |
| `qc` | 质检员 | 质量证书增删改 |
| `sales` | 销售人员 | 采购/销售订单、客户管理、合同、标签、数据导入导出 |

### 各功能模块角色矩阵

| 功能模块 | 读取 | 写入 |
|---------|------|------|
| 用户管理 | admin | admin |
| 钢管（无缝/筛管） | 所有已认证用户 | admin, warehouse |
| 入库/出库 | 所有已认证用户 | admin, warehouse |
| 库存/库位/盘点 | 所有已认证用户 | admin, warehouse |
| 供应商 | 所有已认证用户 | admin, warehouse, sales |
| 客户 | 所有已认证用户 | admin, warehouse, sales |
| 采购订单 | 所有已认证用户 | admin, warehouse, sales |
| 销售订单 | 所有已认证用户 | admin, warehouse, sales |
| 质量证书 | 所有已认证用户 | admin, qc |
| 合同 | 所有已认证用户 | admin, warehouse, sales |
| 报告/仪表盘 | 所有已认证用户 | — |
| 标签打印 | 所有已认证用户 | admin |
| 数据导入导出 | 所有已认证用户 | admin, warehouse, sales |
| ATP 可用库存 | 所有已认证用户 | — |
| 全局搜索 | 所有已认证用户 | — |

---

## 统一响应格式

### 成功响应

```json
{
  "success": true,
  "request_id": "req_<uuid-v4>",
  "data": { ... }
}
```

### 分页响应

```json
{
  "success": true,
  "request_id": "req_<uuid-v4>",
  "meta": {
    "total": 100,
    "page": 1,
    "page_size": 20,
    "total_pages": 5
  },
  "data": {
    "items": [ ... ],
    "total": 100,
    "page": 1,
    "page_size": 20,
    "total_pages": 5
  }
}
```

### 创建成功 (201)

```json
{
  "success": true,
  "request_id": "req_<uuid-v4>",
  "data": { ... }
}
```

### 删除成功 (204)

无响应体。

### 错误响应

```json
{
  "success": false,
  "code": 12001,
  "request_id": "req_<uuid-v4>",
  "message": "Pipe not found: 42",
  "details": null
}
```

---

## 分页格式

所有列表接口支持分页，使用 Query 参数：

| 参数 | 类型 | 默认值 | 说明 |
|------|------|-------|------|
| `page` | u64 | 1 | 页码 |
| `page_size` | u64 | 20 | 每页条数 |

**响应中包含 `meta` 字段：**

```json
{
  "meta": {
    "total": 100,
    "page": 1,
    "page_size": 20,
    "total_pages": 5
  }
}
```

---

## 错误码速查表

### 通用错误 (100xx)

| 错误码 | 说明 | HTTP 状态 |
|--------|------|----------|
| 10001 | 服务器内部错误 | 500 |
| 10002 | 参数验证失败 | 400 |
| 10003 | 资源不存在 | 404 |
| 10004 | 请求格式错误 | 400 |

### 认证与权限 (110xx)

| 错误码 | 说明 | HTTP 状态 |
|--------|------|----------|
| 11001 | 未登录/认证令牌无效 | 401 |
| 11002 | 登录失败（用户名或密码错误） | 401 |
| 11003 | 权限不足 | 403 |
| 11004 | 认证令牌已过期 | 401 |
| 11005 | Refresh Token 无效 | 401 |

### 钢管 (120xx)

| 错误码 | 说明 | HTTP 状态 |
|--------|------|----------|
| 12001 | 钢管不存在 | 404 |
| 12002 | 管号重复 | 409 |
| 12003 | 钢管状态不允许此操作 | 409 |

### 库存 (130xx)

| 错误码 | 说明 | HTTP 状态 |
|--------|------|----------|
| 13001 | 库存不足 | 409 |
| 13002 | 库位不存在 | 404 |

### 订单 (140xx)

| 错误码 | 说明 | HTTP 状态 |
|--------|------|----------|
| 14001 | 订单不存在 | 404 |
| 14002 | 订单状态不允许此操作 | 409 |

### 质量 (150xx)

| 错误码 | 说明 | HTTP 状态 |
|--------|------|----------|
| 15001 | 质量证书不存在 | 404 |
| 15002 | 附件不存在 | 404 |

### 供应商 (160xx)

| 错误码 | 说明 | HTTP 状态 |
|--------|------|----------|
| 16001 | 供应商不存在 | 404 |
| 16002 | 供应商编码重复 | 409 |

### 客户 (170xx)

| 错误码 | 说明 | HTTP 状态 |
|--------|------|----------|
| 17001 | 客户不存在 | 404 |
| 17002 | 客户编码重复 | 409 |

### 数据导入导出 (180xx)

| 错误码 | 说明 | HTTP 状态 |
|--------|------|----------|
| 18001 | 导入失败 | 400 |
| 18002 | 导出失败 | 400 |

### 数据库 (50001)

| 错误码 | 说明 | HTTP 状态 |
|--------|------|----------|
| 50001 | 数据库错误 | 500 |

---

## 接口一览

### 1. 认证与用户管理

#### POST `/api/v1/auth/login` — 用户登录

**认证:** 无需

**请求体:**

```json
{
  "username": "admin",
  "password": "admin123"
}
```

**响应 (200):**

```json
{
  "success": true,
  "request_id": "req_...",
  "data": {
    "access_token": "eyJhbGci...",
    "token_type": "Bearer",
    "expires_in": 7200,
    "user": {
      "id": 1,
      "username": "admin",
      "role": "admin",
      "display_name": "系统管理员"
    }
  }
}
```

---

#### POST `/api/v1/auth/refresh` — 刷新令牌

**认证:** Cookie `refresh_token`

**响应 (200):** 与登录相同格式，返回新的 access_token 和 refresh_token。

---

#### POST `/api/v1/auth/logout` — 登出

**认证:** Bearer Token

**响应 (200):**

```json
{
  "success": true,
  "request_id": "req_...",
  "data": "Logged out"
}
```

---

#### GET `/api/v1/auth/me` — 获取当前用户

**认证:** Bearer Token

**响应 (200):**

```json
{
  "success": true,
  "request_id": "req_...",
  "data": {
    "id": 1,
    "username": "admin",
    "role": "admin",
    "display_name": "系统管理员",
    "created_at": "2025-01-01T00:00:00Z"
  }
}
```

---

#### GET `/api/v1/users` — 用户列表（管理员）

**认证:** Bearer Token + admin 角色

**Query 参数:** `page`, `page_size`, `q`（搜索关键词）

---

#### POST `/api/v1/users` — 创建用户

**认证:** Bearer Token + admin 角色

**请求体:**

```json
{
  "username": "warehouse01",
  "password": "securepass123",
  "role": "warehouse",
  "display_name": "仓库管理员"
}
```

**角色可选值:** `admin`, `warehouse`, `qc`, `sales`

---

#### PUT `/api/v1/users/{id}` — 更新用户

**认证:** Bearer Token + admin 角色

---

#### DELETE `/api/v1/users/{id}` — 删除用户

**认证:** Bearer Token + admin 角色

**响应:** 204 No Content

---

#### PUT `/api/v1/users/{id}/role` — 修改用户角色

**认证:** Bearer Token + admin 角色

**请求体:**

```json
{
  "role": "qc"
}
```

---

#### PUT `/api/v1/auth/password/{id}` — 修改密码

**认证:** Bearer Token（管理员可修改任意用户；普通用户只能修改自己的密码）

**请求体:**

```json
{
  "old_password": "oldpass",
  "new_password": "newpass123"
}
```

---

### 2. 无缝钢管管理

#### GET `/api/v1/seamless-pipes` — 列表（分页）

**认证:** Bearer Token

**Query 参数:** `page`, `page_size`, `pipe_number`, `grade`, `spec`, `status` 等

---

#### GET `/api/v1/seamless-pipes/{id}` — 详情

---

#### POST `/api/v1/seamless-pipes` — 新增

**认证:** Bearer Token + admin/warehouse

**请求体:**

```json
{
  "pipe_number": "SP-2025-001",
  "grade": "J55",
  "outer_diameter": 139.7,
  "wall_thickness": 7.72,
  "length": 9.6,
  "steel_grade": "J55",
  "heat_number": "HT2025001",
  "manufacturer": "宝钢",
  "status": "in_stock"
}
```

---

#### PUT `/api/v1/seamless-pipes/{id}` — 更新

**认证:** Bearer Token + admin/warehouse

---

#### DELETE `/api/v1/seamless-pipes/{id}` — 删除（软删除）

**认证:** Bearer Token + admin/warehouse

**响应:** 204 No Content

---

### 3. 筛管管理

接口路径和参数结构与无缝钢管相同，路径为 `/api/v1/screen-pipes`。

#### GET `/api/v1/screen-pipes` — 列表

#### GET `/api/v1/screen-pipes/{id}` — 详情

#### POST `/api/v1/screen-pipes` — 新增

**认证:** Bearer Token + admin/warehouse

#### PUT `/api/v1/screen-pipes/{id}` — 更新

#### DELETE `/api/v1/screen-pipes/{id}` — 删除

---

#### GET `/api/v1/pipes/search` — 全局钢管搜索

**认证:** Bearer Token

**Query 参数:** `q`（关键词），支持管号、等级、规格等模糊搜索。

---

### 4. 库存管理

#### 入库记录

##### GET `/api/v1/inbound-records` — 入库记录列表

**认证:** Bearer Token

##### GET `/api/v1/inbound-records/{id}` — 入库记录详情

##### GET `/api/v1/inbound-records/{id}/items` — 入库明细列表

##### POST `/api/v1/inbound-records` — 创建入库记录

**认证:** Bearer Token + admin/warehouse

**请求体:**

```json
{
  "inbound_type": "purchase",
  "supplier_id": 1,
  "purchase_order_id": 10,
  "items": [
    {
      "pipe_id": 1,
      "quantity": 50,
      "location_id": 1
    }
  ],
  "notes": "正常采购入库"
}
```

**入库类型:** `purchase`（采购入库）、`production`（生产退料）、`return`（退货入库）

##### PUT `/api/v1/inbound-records/{id}` — 更新入库记录

##### DELETE `/api/v1/inbound-records/{id}` — 删除入库记录

---

#### 出库记录

##### GET `/api/v1/outbound-records` — 出库记录列表

##### GET `/api/v1/outbound-records/{id}` — 出库记录详情

##### GET `/api/v1/outbound-records/{id}/items` — 出库明细列表

##### POST `/api/v1/outbound-records` — 创建出库记录

**认证:** Bearer Token + admin/warehouse

**请求体:**

```json
{
  "outbound_type": "sales",
  "customer_id": 1,
  "sales_order_id": 5,
  "items": [
    {
      "pipe_id": 1,
      "quantity": 20,
      "location_id": 1
    }
  ],
  "notes": "销售出库"
}
```

---

#### 库存查询

##### GET `/api/v1/inventory` — 库存列表

**认证:** Bearer Token

**Query 参数:** `page`, `page_size`, `pipe_id`, `location_id` 等

##### GET `/api/v1/inventory/logs` — 库存变动日志

##### GET `/api/v1/inventory/statistics` — 库存统计

---

#### 库位管理

##### GET `/api/v1/locations` — 库位列表

##### GET `/api/v1/locations/{id}` — 库位详情

##### POST `/api/v1/locations` — 创建库位

**认证:** Bearer Token + admin/warehouse

##### PUT `/api/v1/locations/{id}` — 更新库位

##### DELETE `/api/v1/locations/{id}` — 删除库位

---

#### 盘点

##### GET `/api/v1/inventory/checks` — 盘点记录列表

##### GET `/api/v1/inventory/checks/{id}` — 盘点详情

##### POST `/api/v1/inventory/checks` — 创建盘点

**认证:** Bearer Token + admin/warehouse

---

#### 追溯

##### GET `/api/v1/trace/pipe/{pipe_type}/{pipe_id}` — 钢管追溯

**认证:** Bearer Token

##### GET `/api/v1/trace/order/{order_type}/{order_id}` — 订单追溯

---

### 5. 供应商管理

#### GET `/api/v1/suppliers` — 供应商列表

**认证:** Bearer Token

#### GET `/api/v1/suppliers/search` — 供应商搜索

#### GET `/api/v1/suppliers/active` — 活跃供应商列表

#### GET `/api/v1/suppliers/{id}` — 供应商详情

#### POST `/api/v1/suppliers` — 新增供应商

**认证:** Bearer Token + admin/warehouse/sales

**请求体:**

```json
{
  "code": "SUP-001",
  "name": "宝钢集团",
  "contact_person": "张三",
  "phone": "13800138000",
  "email": "zhangsan@baosteel.com",
  "address": "上海市宝山区",
  "status": "active"
}
```

#### PUT `/api/v1/suppliers/{id}` — 更新供应商

#### DELETE `/api/v1/suppliers/{id}` — 删除供应商

---

### 6. 客户管理

#### GET `/api/v1/customers` — 客户列表

**认证:** Bearer Token

#### GET `/api/v1/customers/search` — 客户搜索

#### GET `/api/v1/customers/active` — 活跃客户列表

#### GET `/api/v1/customers/{id}` — 客户详情

#### POST `/api/v1/customers` — 新增客户

**认证:** Bearer Token + admin/warehouse/sales

**请求体:**

```json
{
  "code": "CUS-001",
  "name": "中石油",
  "contact_person": "李四",
  "phone": "13900139000",
  "email": "lisi@cnpc.com",
  "address": "北京市东城区",
  "status": "active"
}
```

#### PUT `/api/v1/customers/{id}` — 更新客户

#### DELETE `/api/v1/customers/{id}` — 删除客户

---

### 7. 采购订单

#### GET `/api/v1/purchase-orders` — 采购订单列表

**认证:** Bearer Token

#### GET `/api/v1/purchase-orders/{id}` — 采购订单详情

#### POST `/api/v1/purchase-orders` — 创建采购订单

**认证:** Bearer Token + admin/warehouse/sales

**请求体:**

```json
{
  "order_number": "PO-2025-001",
  "supplier_id": 1,
  "order_date": "2025-01-15",
  "expected_date": "2025-02-15",
  "items": [
    {
      "pipe_number": "SP-2025-001",
      "grade": "J55",
      "spec": "139.7x7.72",
      "quantity": 100,
      "unit_price": 3500.00
    }
  ]
}
```

#### PUT `/api/v1/purchase-orders/{id}` — 更新采购订单

#### DELETE `/api/v1/purchase-orders/{id}` — 删除采购订单

#### PUT `/api/v1/purchase-orders/{id}/status` — 订单状态流转

**请求体:**

```json
{
  "status": "approved"
}
```

**订单状态流转:**
- `draft` → `pending` | `cancelled`
- `pending` → `approved` | `rejected`
- `rejected` → `draft`
- `approved` → `completed` | `cancelled`

#### POST `/api/v1/purchase-orders/{id}/approve` — 审批通过

#### POST `/api/v1/purchase-orders/{id}/reject` — 审批驳回

#### POST `/api/v1/purchase-orders/{id}/link-inbound` — 关联入库记录

---

### 8. 销售订单

接口结构与采购订单对称，路径为 `/api/v1/sales-orders`。

#### GET `/api/v1/sales-orders` — 列表

#### GET `/api/v1/sales-orders/{id}` — 详情

#### POST `/api/v1/sales-orders` — 创建

**认证:** Bearer Token + admin/warehouse/sales

#### PUT `/api/v1/sales-orders/{id}` — 更新

#### DELETE `/api/v1/sales-orders/{id}` — 删除

#### PUT `/api/v1/sales-orders/{id}/status` — 状态流转

#### POST `/api/v1/sales-orders/{id}/approve` — 审批通过

#### POST `/api/v1/sales-orders/{id}/reject` — 审批驳回

#### POST `/api/v1/sales-orders/{id}/link-outbound` — 关联出库记录

---

### 9. 质量证书

#### GET `/api/v1/quality/certs` — 质量证书列表

**认证:** Bearer Token

#### GET `/api/v1/quality/certs/{id}` — 证书详情

#### POST `/api/v1/quality/certs` — 新增证书

**认证:** Bearer Token + admin/qc

**请求体:**

```json
{
  "pipe_id": 1,
  "cert_number": "QC-2025-001",
  "issue_date": "2025-01-20",
  "expiry_date": "2027-01-20",
  "result": "pass",
  "inspector": "王五",
  "notes": "各项指标合格"
}
```

#### PUT `/api/v1/quality/certs/{id}` — 更新证书

#### DELETE `/api/v1/quality/certs/{id}` — 删除证书

---

#### GET `/api/v1/quality/grades` — 质量等级列表

#### GET `/api/v1/quality/grades/query` — 查询质量等级

---

### 10. 合同管理

#### GET `/api/v1/contracts` — 合同列表

**认证:** Bearer Token

#### GET `/api/v1/contracts/{id}` — 合同详情

#### POST `/api/v1/contracts` — 创建合同

**认证:** Bearer Token + admin/warehouse/sales

**请求体:**

```json
{
  "contract_number": "CON-2025-001",
  "title": "2025年度无缝钢管采购合同",
  "party_a": "我方公司",
  "party_b": "宝钢集团",
  "type": "purchase",
  "signing_date": "2025-01-01",
  "effective_date": "2025-01-01",
  "expiry_date": "2025-12-31",
  "total_amount": 5000000.00
}
```

#### PUT `/api/v1/contracts/{id}` — 更新合同

#### DELETE `/api/v1/contracts/{id}` — 删除合同

---

### 11. 报告与仪表盘

#### GET `/api/v1/reports` — 报告列表

**认证:** Bearer Token

#### GET `/api/v1/reports/dashboard` — 仪表盘数据

#### GET `/api/v1/reports/inventory` — 库存报告

#### GET `/api/v1/reports/orders` — 订单报告

#### GET `/api/v1/reports/quality` — 质量报告

---

### 12. 标签打印

#### GET `/api/v1/labels` — 获取标签数据

**认证:** Bearer Token + admin

#### POST `/api/v1/labels/print` — 打印标签

**认证:** Bearer Token + admin

---

### 13. 数据导入导出

#### GET `/api/v1/data-io/templates/{entity_type}` — 下载导入模板

**认证:** Bearer Token

**entity_type 可选值:** `seamless_pipes`, `screen_pipes`, `suppliers`, `customers`, `purchase_orders`, `sales_orders`, `quality_certs`

#### POST `/api/v1/data-io/import/{entity_type}` — 批量导入

**认证:** Bearer Token + admin/warehouse/sales

**请求:** `multipart/form-data`，字段名 `file`

#### GET `/api/v1/data-io/export/{entity_type}` — 批量导出

**认证:** Bearer Token + admin/warehouse/sales

**响应:** 文件下载（`.xlsx` 或 `.csv`）

#### GET `/api/v1/data-io/logs` — 操作日志

**认证:** Bearer Token + admin

---

### 14. ATP 可用库存检查

#### GET `/api/v1/atp` — 可用库存查询

**认证:** Bearer Token

**Query 参数:** `pipe_id`, `grade`, `spec` 等

---

### 15. 全局搜索

#### GET `/api/v1/search` — 全局搜索

**认证:** Bearer Token

**Query 参数:** `q`（关键词）

搜索范围覆盖钢管、供应商、客户、订单等所有业务实体。

---

### 16. 个人信息

#### GET `/api/v1/profile` — 获取个人信息

**认证:** Bearer Token

#### PUT `/api/v1/profile` — 更新个人信息

**认证:** Bearer Token

---

### 17. 健康检查

#### GET `/health` — 服务健康检查

**认证:** 无需

**响应 (200):**

```json
{
  "status": "ok"
}
```

---

## 通用查询参数

### 分页参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|-------|------|
| `page` | u64 | 1 | 页码（从 1 开始） |
| `page_size` | u64 | 20 | 每页条数 |

### 搜索参数

大部分列表接口支持 `q` 参数进行关键词搜索。

### 过滤参数

各列表接口支持按字段过滤，具体字段请参考对应 DTO 定义（`backend/src/dto/`）。

---

## 附录：环境变量

| 变量 | 默认值 | 说明 |
|------|-------|------|
| `DATABASE_URL` | `sqlite:steel_pipe.db` | 数据库连接字符串 |
| `JWT_SECRET` | — | JWT 签名密钥（必填） |
| `JWT_EXPIRY_HOURS` | 2 | Access Token 有效期（小时） |
| `REFRESH_TOKEN_EXPIRY_DAYS` | 30 | Refresh Token 有效期（天） |
| `APP_ENV` | `development` | 运行环境（`development` / `production`） |
| `CORS_ORIGINS` | `http://localhost:5173` | CORS 允许的来源 |
| `RATE_LIMIT_LOGIN` | 10 | 登录接口每分钟最大请求次数 |
| `RATE_LIMIT_PASSWORD_CHANGE` | 5 | 修改密码每分钟最大请求次数 |

---

## 附录：操作日志

系统对以下操作自动记录操作日志：

- 登录/登出
- 创建/更新/删除用户
- 修改密码/角色
- 创建/更新/删除业务实体

操作日志存储在 `operation_logs` 表中，可通过 `/api/v1/data-io/logs` 查询。
