# ERP API 文档

> 通用 ERP（企业资源计划系统）— 后端 REST API 参考
> 历史沿革：本系统由钢管行业系统重构而来，后端 crate 名为 `erp-server`。

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
  - [商品管理](#2-商品管理)
  - [库存管理（入库/出库/库存/库位/盘点/追溯）](#3-库存管理)
  - [供应商管理](#4-供应商管理)
  - [客户管理](#5-客户管理)
  - [采购订单](#6-采购订单)
  - [销售订单](#7-销售订单)
  - [合同管理](#8-合同管理)
  - [制造管理（BOM/工单/质检）](#9-制造管理)
  - [审批流](#10-审批流)
  - [人力资源](#11-人力资源)
  - [财务](#12-财务)
  - [采购管理（申请/收货/采购报价/评分）](#13-采购管理)
  - [项目与固定资产](#14-项目与固定资产)
  - [通知与门户](#15-通知与门户)
  - [销售 CRM（发货/销售报价/信用）](#16-销售-crm)
  - [报告与 BI](#17-报告与-bi)
  - [数据导入导出](#18-数据导入导出)
  - [ATP 可用库存检查](#19-atp-可用库存检查)
  - [全局搜索](#20-全局搜索)
  - [个人信息](#21-个人信息)
  - [健康检查](#22-健康检查)

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
| ------ | ------ | --------- |
| `admin` | 系统管理员 | 所有功能，包括用户/角色/权限管理 |
| `warehouse` | 仓库管理员 | 商品维护、入库出库、库位、盘点 |
| `sales` | 业务人员 | 采购/销售订单、客户/供应商、合同、数据导入导出 |

> 角色、权限、部门、租户均可通过 `/api/v1/auth/roles`、`/api/v1/auth/permissions`、
> `/api/v1/auth/departments` 动态配置（auth/RBAC 模块）。

### 各功能模块角色矩阵

| 功能模块 | 读取 | 写入 |
| --------- | ------ | ------ |
| 用户管理 | admin | admin |
| 商品（Item/SKU） | 所有已认证用户 | admin, warehouse |
| 入库/出库 | 所有已认证用户 | admin, warehouse |
| 库存/库位/盘点 | 所有已认证用户 | admin, warehouse |
| 供应商 | 所有已认证用户 | admin, warehouse, sales |
| 客户 | 所有已认证用户 | admin, warehouse, sales |
| 采购订单 | 所有已认证用户 | admin, warehouse, sales |
| 销售订单 | 所有已认证用户 | admin, warehouse, sales |
| 合同 | 所有已认证用户 | admin, warehouse, sales |
| 制造（BOM/工单/质检） | 所有已认证用户 | 按 RBAC 角色配置 |
| 审批流 | 所有已认证用户 | 按 RBAC 角色配置 |
| 人力资源 / 财务 | 按 RBAC 角色配置 | admin 或按角色配置 |
| 报告/BI | 所有已认证用户 | — |
| 数据导入导出 | 所有已认证用户 | admin, warehouse, sales |
| ATP 可用库存 | 所有已认证用户 | — |
| 全局搜索 | 所有已认证用户 | — |
| 门户 | 所有已认证用户 | admin（门户账户管理） |

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
  "message": "Item not found: 42",
  "details": null
}
```

---

## 分页格式

所有列表接口支持分页，使用 Query 参数：

| 参数 | 类型 | 默认值 | 说明 |
| ------ | ------ | ------- | ------ |
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
| -------- | ------ | ---------- |
| 10001 | 服务器内部错误 | 500 |
| 10002 | 参数验证失败 | 400 |
| 10003 | 资源不存在 | 404 |
| 10004 | 请求格式错误 | 400 |

### 认证与权限 (110xx)

| 错误码 | 说明 | HTTP 状态 |
| -------- | ------ | ---------- |
| 11001 | 未登录/认证令牌无效 | 401 |
| 11002 | 登录失败（用户名或密码错误） | 401 |
| 11003 | 权限不足 | 403 |
| 11004 | 认证令牌已过期 | 401 |
| 11005 | Refresh Token 无效 | 401 |

### 商品 (120xx)

| 错误码 | 说明 | HTTP 状态 |
| -------- | ------ | ---------- |
| 12001 | 商品不存在 | 404 |
| 12002 | SKU 重复 | 409 |
| 12003 | 商品状态不允许此操作 | 409 |

### 库存 (130xx)

| 错误码 | 说明 | HTTP 状态 |
| -------- | ------ | ---------- |
| 13001 | 库存不足 | 409 |
| 13002 | 库位不存在 | 404 |

### 订单 (140xx)

| 错误码 | 说明 | HTTP 状态 |
| -------- | ------ | ---------- |
| 14001 | 订单不存在 | 404 |
| 14002 | 订单状态不允许此操作 | 409 |

### 质检 (150xx)

| 错误码 | 说明 | HTTP 状态 |
| -------- | ------ | ---------- |
| 15001 | 质检记录不存在 | 404 |
| 15002 | 附件不存在 | 404 |

### 供应商 (160xx)

| 错误码 | 说明 | HTTP 状态 |
| -------- | ------ | ---------- |
| 16001 | 供应商不存在 | 404 |
| 16002 | 供应商编码重复 | 409 |

### 客户 (170xx)

| 错误码 | 说明 | HTTP 状态 |
| -------- | ------ | ---------- |
| 17001 | 客户不存在 | 404 |
| 17002 | 客户编码重复 | 409 |

### 数据导入导出 (180xx)

| 错误码 | 说明 | HTTP 状态 |
| -------- | ------ | ---------- |
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

#### PUT `/api/v1/auth/me` — 更新个人信息

**认证:** Bearer Token

**请求体:**

```json
{
  "display_name": "新名称",
  "email": "user@example.com"
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

**角色可选值:** `admin`, `warehouse`, `sales`（也可通过 RBAC 模块自定义角色）

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
  "role": "warehouse"
}
```

---

#### POST `/api/v1/users/{id}/change-password` — 修改密码

**认证:** Bearer Token（管理员可修改任意用户；普通用户只能修改自己的密码）

**请求体:**

```json
{
  "old_password": "oldpass",
  "new_password": "newpass123"
}
```

---

#### RBAC 管理接口

| 方法 | 路径 | 说明 | 认证 |
| ------ | ------ | ------ | ------ |
| GET | `/api/v1/auth/permissions` | 权限列表 | admin |
| GET/POST | `/api/v1/auth/roles` | 角色列表/创建 | admin |
| GET/PUT/DELETE | `/api/v1/auth/roles/{id}` | 角色详情/更新/删除 | admin |
| PUT | `/api/v1/auth/roles/{id}/permissions` | 角色授权 | admin |
| GET/POST | `/api/v1/auth/departments` | 部门列表/创建 | admin |
| GET/PUT/DELETE | `/api/v1/auth/departments/{id}` | 部门详情/更新/删除 | admin |
| GET | `/api/v1/auth/tenants/{id}` | 租户详情 | admin |
| PUT | `/api/v1/auth/users/{user_id}/roles` | 分配用户角色 | admin |
| GET | `/api/v1/auth/users/{user_id}/permissions` | 查询用户权限 | admin |

---

### 2. 商品管理

> 商品（Item）是全系统的唯一业务实体，SKU 为其唯一业务编码。
> 商品表字段：`sku`、`name`（名称）、`category`（分类）、`unit`（单位）、`spec`（规格）、`status` 等。

#### GET `/api/v1/items` — 商品列表（分页）

**认证:** Bearer Token

**Query 参数:** `page`, `page_size`, `sku`, `name`, `category`, `status` 等

---

#### GET `/api/v1/items/{id}` — 商品详情

**认证:** Bearer Token

---

#### POST `/api/v1/items` — 新增商品

**认证:** Bearer Token + admin/warehouse

**请求体:**

```json
{
  "sku": "ITEM-2025-0001",
  "name": "示例商品",
  "category": "原材料",
  "unit": "件",
  "spec": "标准规格",
  "status": "active"
}
```

---

#### PUT `/api/v1/items/{id}` — 更新商品

**认证:** Bearer Token + admin/warehouse

---

#### DELETE `/api/v1/items/{id}` — 删除商品（软删除）

**认证:** Bearer Token + admin/warehouse

**响应:** 204 No Content

---

#### GET `/api/v1/items/search` — 商品搜索

**认证:** Bearer Token

**Query 参数:** `q`（关键词），支持 SKU、名称、分类、规格等模糊搜索。

---

### 3. 库存管理

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
      "item_id": 1,
      "quantity": 50,
      "location_id": 1
    }
  ],
  "notes": "正常采购入库"
}
```

**入库类型:** `purchase`（采购入库）、`production`（生产退料）、`return`（退货入库）

##### POST `/api/v1/inbound-records/batch` — 批量创建入库记录

##### PUT `/api/v1/inbound-records/{id}` — 更新入库记录

##### DELETE `/api/v1/inbound-records/{id}` — 删除入库记录

##### POST `/api/v1/inbound-records/{id}/approve` — 审批通过

##### POST `/api/v1/inbound-records/{id}/reject` — 审批驳回

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
      "item_id": 1,
      "quantity": 20,
      "location_id": 1
    }
  ],
  "notes": "销售出库"
}
```

##### PUT `/api/v1/outbound-records/{id}` — 更新出库记录

##### DELETE `/api/v1/outbound-records/{id}` — 删除出库记录

##### POST `/api/v1/outbound-records/{id}/approve` — 审批通过

##### POST `/api/v1/outbound-records/{id}/reject` — 审批驳回

---

#### 库存查询

##### GET `/api/v1/inventory` — 库存列表

**认证:** Bearer Token

**Query 参数:** `page`, `page_size`, `item_id`, `location_id` 等

##### GET `/api/v1/inventory/logs` — 库存变动日志

##### GET `/api/v1/inventory/statistics` — 库存统计

##### GET `/api/v1/inventory/inbound/search` — 入库记录搜索

##### GET `/api/v1/inventory/outbound/search` — 出库记录搜索

---

#### 库位管理

##### GET `/api/v1/locations` — 库位列表

##### GET `/api/v1/locations/{id}` — 库位详情

##### POST `/api/v1/locations` — 创建库位

**认证:** Bearer Token + admin/warehouse

##### PUT `/api/v1/locations/{id}` — 更新库位

##### DELETE `/api/v1/locations/{id}` — 删除库位

##### PUT `/api/v1/inventory/locations/{id}/assign` — 库位分配

---

#### 盘点

##### GET `/api/v1/inventory/checks` — 盘点记录列表

##### GET `/api/v1/inventory/checks/{id}` — 盘点详情

##### POST `/api/v1/inventory/checks` — 创建盘点

**认证:** Bearer Token + admin/warehouse

##### POST `/api/v1/inventory/checks/{id}/complete` — 完成盘点

##### PUT `/api/v1/inventory/checks/{check_id}/items/{item_id}` — 盘点明细更新

---

#### 追溯

##### GET `/api/v1/trace/item/{item_id}` — 商品追溯

**认证:** Bearer Token

##### GET `/api/v1/trace/sku/{sku}` — 按 SKU 追溯

##### GET `/api/v1/trace/order/{order_type}/{order_id}` — 订单追溯

---

### 4. 供应商管理

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
  "name": "供应商名称",
  "contact_person": "张三",
  "phone": "13800138000",
  "email": "zhangsan@example.com",
  "address": "示例地址",
  "status": "active"
}
```

#### PUT `/api/v1/suppliers/{id}` — 更新供应商

#### DELETE `/api/v1/suppliers/{id}` — 删除供应商

---

### 5. 客户管理

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
  "name": "客户名称",
  "contact_person": "李四",
  "phone": "13900139000",
  "email": "lisi@example.com",
  "address": "示例地址",
  "status": "active"
}
```

#### PUT `/api/v1/customers/{id}` — 更新客户

#### DELETE `/api/v1/customers/{id}` — 删除客户

---

### 6. 采购订单

#### GET `/api/v1/purchase-orders` — 采购订单列表

**认证:** Bearer Token

#### GET `/api/v1/purchase-orders/{id}` — 采购订单详情

#### GET `/api/v1/purchase-orders/search` — 采购订单搜索

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
      "sku": "ITEM-2025-0001",
      "spec": "标准规格",
      "quantity": 100,
      "unit_price": 3500.00
    }
  ]
}
```

#### PUT `/api/v1/purchase-orders/{id}` — 更新采购订单

#### DELETE `/api/v1/purchase-orders/{id}` — 删除采购订单

#### PUT `/api/v1/purchase-orders/{id}/transition` — 订单状态流转

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

#### PUT `/api/v1/purchase-orders/{order_id}/items/{item_id}` — 更新订单明细

#### POST `/api/v1/purchase-orders/{id}/link-inbound` — 关联入库记录

---

### 7. 销售订单

接口结构与采购订单对称，路径为 `/api/v1/sales-orders`。

#### GET `/api/v1/sales-orders` — 列表

#### GET `/api/v1/sales-orders/{id}` — 详情

#### GET `/api/v1/sales-orders/search` — 搜索

#### POST `/api/v1/sales-orders` — 创建

**认证:** Bearer Token + admin/warehouse/sales

#### PUT `/api/v1/sales-orders/{id}` — 更新

#### DELETE `/api/v1/sales-orders/{id}` — 删除

#### PUT `/api/v1/sales-orders/{id}/transition` — 状态流转

#### POST `/api/v1/sales-orders/{id}/approve` — 审批通过

#### POST `/api/v1/sales-orders/{id}/reject` — 审批驳回

#### PUT `/api/v1/sales-orders/{order_id}/items/{item_id}` — 更新订单明细

#### POST `/api/v1/sales-orders/{id}/link-outbound` — 关联出库记录

---

### 8. 合同管理

#### GET `/api/v1/contracts` — 合同列表

**认证:** Bearer Token

#### GET `/api/v1/contracts/{id}` — 合同详情

#### POST `/api/v1/contracts` — 创建合同

**认证:** Bearer Token + admin/warehouse/sales

**请求体:**

```json
{
  "contract_number": "CON-2025-001",
  "title": "2025年度采购合同",
  "party_a": "我方公司",
  "party_b": "供应商名称",
  "type": "purchase",
  "signing_date": "2025-01-01",
  "effective_date": "2025-01-01",
  "expiry_date": "2025-12-31",
  "total_amount": 5000000.00
}
```

#### PUT `/api/v1/contracts/{id}` — 更新合同

#### DELETE `/api/v1/contracts/{id}` — 删除合同

#### PUT `/api/v1/contracts/{id}/status` — 合同状态流转

#### GET/POST `/api/v1/contracts/{contract_id}/items` — 合同明细列表/添加

#### PUT/DELETE `/api/v1/contracts/{contract_id}/items/{item_id}` — 合同明细更新/删除

#### GET/POST `/api/v1/contracts/{contract_id}/payments` — 合同付款计划列表/添加

#### PUT/DELETE `/api/v1/contracts/{contract_id}/payments/{payment_id}` — 合同付款计划更新/删除

---

### 9. 制造管理

#### BOM（物料清单）

##### GET/POST `/api/v1/manufacturing/boms` — BOM 列表/创建

**认证:** Bearer Token（写入需对应角色）

##### GET/PUT/DELETE `/api/v1/manufacturing/boms/{id}` — BOM 详情/更新/删除

---

#### 工单（Work Order）

##### GET/POST `/api/v1/manufacturing/work-orders` — 工单列表/创建

##### GET/PUT/DELETE `/api/v1/manufacturing/work-orders/{id}` — 工单详情/更新/删除

##### POST `/api/v1/manufacturing/work-orders/{id}/start` — 开始工单

##### POST `/api/v1/manufacturing/work-orders/{id}/complete-step` — 完成工序

---

#### 质检（Inspection）

##### GET/POST `/api/v1/manufacturing/inspections` — 质检记录列表/创建

**认证:** Bearer Token（写入需对应角色）

质检记录关联**工单**，记录检验结果（`pass` / `fail`）与附件。

---

#### 不合格品单（NCR）

##### GET/POST `/api/v1/manufacturing/ncrs` — 不合格品单列表/创建

##### POST `/api/v1/manufacturing/ncrs/{id}/resolve` — 处理不合格品单

---

### 10. 审批流

#### 审批流定义（Workflow Definition）

##### GET/POST `/api/v1/workflows/definitions` — 审批流定义列表/创建

**认证:** Bearer Token（admin 可管理）

##### GET/PUT/DELETE `/api/v1/workflows/definitions/{id}` — 定义详情/更新/删除

---

#### 审批流实例（Workflow Instance）

##### GET/POST `/api/v1/workflows/instances` — 实例列表/发起审批

---

#### 审批任务（Workflow Task）

##### GET `/api/v1/workflows/my-tasks` — 我的待办

##### GET `/api/v1/workflows/tasks/{node_id}` — 任务详情

##### POST `/api/v1/workflows/tasks/{node_id}/approve` — 审批通过

##### POST `/api/v1/workflows/tasks/{node_id}/reject` — 审批驳回

##### GET/POST `/api/v1/workflows/delegations` — 任务委托

---

### 11. 人力资源

##### GET/POST `/api/v1/hr/employees` — 员工列表/创建

**认证:** Bearer Token（写入需对应角色）

##### GET/PUT/DELETE `/api/v1/hr/employees/{id}` — 员工详情/更新/删除

##### POST `/api/v1/hr/employees/{id}/terminate` — 员工离职

##### GET/POST `/api/v1/hr/employees/{id}/contracts` — 员工劳动合同列表/添加

##### GET/POST `/api/v1/hr/contracts` — 劳动合同列表/创建

##### GET/POST `/api/v1/hr/positions` — 岗位列表/创建

##### GET/POST `/api/v1/hr/attendance` — 考勤记录列表/登记

##### POST `/api/v1/hr/attendance/check-in` — 打卡

##### GET/PUT `/api/v1/hr/attendance/rules` — 考勤规则

##### GET/POST `/api/v1/hr/salaries` — 薪资记录列表/发放

##### GET/PUT/DELETE `/api/v1/hr/salaries/{id}` — 薪资记录详情/更新/删除

---

### 12. 财务

#### 会计科目（Account）

##### GET/POST `/api/v1/chart-of-accounts` — 科目列表/创建

**认证:** Bearer Token（写入需对应角色）

##### GET/PUT/DELETE `/api/v1/chart-of-accounts/{id}` — 科目详情/更新/删除

---

#### 日记账（Journal Entry）

##### GET/POST `/api/v1/journal-entries` — 分录列表/创建

##### GET/PUT/DELETE `/api/v1/journal-entries/{id}` — 分录详情/更新/删除

---

#### 试算平衡（Trial Balance）

##### GET `/api/v1/finance/trial-balance` — 试算平衡表

---

#### 发票（Invoice）

##### GET/POST `/api/v1/invoices` — 发票列表/创建

##### GET/PUT/DELETE `/api/v1/invoices/{id}` — 发票详情/更新/删除

##### POST `/api/v1/invoices/{id}/confirm` — 确认发票

##### POST `/api/v1/invoices/{id}/void` — 作废发票

---

#### 付款（Payment）

##### GET/POST `/api/v1/payments` — 付款记录列表/创建

---

### 13. 采购管理

#### 采购申请（Requisition）

##### GET/POST `/api/v1/purchase-requisitions` — 采购申请列表/创建

**认证:** Bearer Token（写入需对应角色）

##### GET/PUT/DELETE `/api/v1/purchase-requisitions/{id}` — 申请详情/更新/删除

---

#### 采购收货（Receipt）

##### GET/POST `/api/v1/po-receipts` — 采购收货列表/创建

##### GET/PUT/DELETE `/api/v1/po-receipts/{id}` — 收货详情/更新/删除

---

#### 采购报价（Supplier Quote）

##### GET/POST `/api/v1/supplier-quotes` — 采购报价列表/创建

##### PUT `/api/v1/supplier-quotes/{id}/status` — 采购报价状态流转

---

#### 供应商评分（Scorecard）

##### GET/POST `/api/v1/suppliers/{supplier_id}/scorecard` — 供应商评分列表/评分

---

### 14. 项目与固定资产

#### 项目（Project）

##### GET/POST `/api/v1/projects` — 项目列表/创建

**认证:** Bearer Token（写入需对应角色）

##### GET/PUT/DELETE `/api/v1/projects/{id}` — 项目详情/更新/删除

##### GET/POST `/api/v1/projects/{id}/wbs` — WBS 列表/创建

##### GET/PUT/DELETE `/api/v1/projects/{project_id}/wbs/{id}` — WBS 节点详情/更新/删除

##### GET `/api/v1/projects/{id}/financials` — 项目预算与财务汇总

##### GET `/api/v1/projects/{id}/transactions` — 项目资金流水

---

#### 固定资产（Fixed Asset）

##### GET/POST `/api/v1/assets` — 固定资产列表/登记

**认证:** Bearer Token（写入需对应角色）

##### GET/PUT/DELETE `/api/v1/assets/{id}` — 资产详情/更新/删除

##### POST `/api/v1/assets/{id}/depreciate` — 计提折旧（直线法）

##### POST `/api/v1/assets/{id}/dispose` — 资产处置

---

### 15. 通知与门户

#### 通知（Notification）

##### GET `/api/v1/notifications` — 通知收件箱

**认证:** Bearer Token

##### GET `/api/v1/notifications/unread-count` — 未读数

##### POST `/api/v1/notifications/{id}/read` — 标记已读

##### GET/PUT `/api/v1/notifications/preferences` — 通知偏好

##### GET/POST `/api/v1/notifications/templates` — 通知模板

---

#### 门户（Portal）

##### GET/POST `/api/v1/portal/accounts` — 门户账户列表/创建

**认证:** Bearer Token + admin

##### POST `/api/v1/portal-api/login` — 门户登录（客户/供应商）

**认证:** 无需（使用门户账户凭证）

##### GET `/api/v1/portal-api/purchases` — 门户查看采购订单

##### POST `/api/v1/portal-api/purchases/{id}/accept` — 门户确认采购订单

##### GET `/api/v1/portal-api/sales` — 门户查看销售订单

##### POST `/api/v1/portal-api/sales/{id}/acknowledge` — 门户确认销售订单

##### GET `/api/v1/portal-api/events` — 门户事件流

---

### 16. 销售 CRM

#### 发货（Shipment）

##### GET/POST `/api/v1/shipments` — 发货记录列表/创建

**认证:** Bearer Token（写入需对应角色）

##### PUT `/api/v1/shipments/{id}/status` — 发货状态流转

---

#### 销售报价（Customer Quote）

##### GET/POST `/api/v1/sales-quotes` — 销售报价列表/创建

##### GET/PUT/DELETE `/api/v1/sales-quotes/{id}` — 销售报价详情/更新/删除

##### PUT `/api/v1/sales-quotes/{id}/status` — 销售报价状态流转

##### POST `/api/v1/sales-quotes/{id}/convert` — 报价转销售订单

---

#### 客户信用（Customer Credit）

##### GET/PUT `/api/v1/customers/{customer_id}/credit` — 客户信用查询/调整

---

### 17. 报告与 BI

#### 报告（Reports）

##### GET `/api/v1/reports/dashboard` — 仪表盘数据

**认证:** Bearer Token

##### GET `/api/v1/reports/inventory-summary` — 库存汇总报告

##### GET `/api/v1/reports/order-report` — 订单报告

---

#### BI 分析（Analytics）

##### GET `/api/v1/bi/sales-trend` — 销售趋势

**认证:** Bearer Token

##### GET `/api/v1/bi/inventory-value` — 库存价值

##### GET `/api/v1/bi/finance-summary` — 财务汇总

##### GET `/api/v1/bi/supplier-performance` — 供应商绩效

---

### 18. 数据导入导出

#### GET `/api/v1/data-io/templates/{entity_type}` — 下载导入模板

**认证:** Bearer Token

**entity_type 可选值:** `items`, `suppliers`, `customers`, `purchase_orders`, `sales_orders`, `contracts`, `employees`, `assets`

#### POST `/api/v1/data-io/import/{entity_type}` — 批量导入

**认证:** Bearer Token + admin/warehouse/sales

**请求:** `multipart/form-data`，字段名 `file`

#### GET `/api/v1/data-io/export/{entity_type}` — 批量导出

**认证:** Bearer Token + admin/warehouse/sales

**响应:** 文件下载（`.xlsx` 或 `.csv`）

#### GET `/api/v1/data-io/operation-logs` — 操作日志

**认证:** Bearer Token + admin

---

### 19. ATP 可用库存检查

#### GET `/api/v1/atp` — 可用库存查询

**认证:** Bearer Token

**Query 参数:** `item_id`, `spec` 等

#### GET `/api/v1/inventory/atp/overview` — ATP 总览

#### GET `/api/v1/inventory/atp/item` — 按商品查询 ATP

---

#### 库存预留（Reservation）

##### GET/POST `/api/v1/inventory/reservations` — 预留列表/创建

##### POST `/api/v1/inventory/reservations/{id}/release` — 释放预留

---

#### 库存转移（Transfer）

##### GET/POST `/api/v1/inventory/transfers` — 转移记录列表/创建

---

#### 盘点模板与会话

##### GET/POST `/api/v1/inventory/count-templates` — 盘点模板列表/创建

##### POST `/api/v1/inventory/count-templates/{template_id}/start` — 启动盘点会话

##### GET/POST `/api/v1/inventory/count-sessions` — 盘点会话列表/创建

---

### 20. 全局搜索

#### GET `/api/v1/search` — 全局搜索

**认证:** Bearer Token

**Query 参数:** `q`（关键词）

搜索范围覆盖商品（Item/SKU）、供应商、客户、采购/销售订单等所有业务实体。

---

### 21. 个人信息

#### GET `/api/v1/auth/me` — 获取个人信息

**认证:** Bearer Token

#### PUT `/api/v1/auth/me` — 更新个人信息

**认证:** Bearer Token

---

### 22. 健康检查

#### GET `/api/v1/health` — 服务健康检查

**认证:** 无需

**响应 (200):**

```json
{
  "status": "ok"
}
```

#### GET `/api/v1/health/ready` — 就绪检查

---

## 通用查询参数

### 分页参数

| 参数 | 类型 | 默认值 | 说明 |
| ------ | ------ | ------- | ------ |
| `page` | u64 | 1 | 页码（从 1 开始） |
| `page_size` | u64 | 20 | 每页条数 |

### 搜索参数

大部分列表接口支持 `q` 参数进行关键词搜索。

### 过滤参数

各列表接口支持按字段过滤，具体字段请参考对应 DTO 定义（`backend/src/dto/`）。

---

## 附录：环境变量

| 变量 | 默认值 | 说明 |
| ------ | ------- | ------ |
| `DATABASE_URL` | `sqlite://data/erp.db?mode=rwc` | SQLite 数据库连接字符串 |
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

操作日志存储在 `operation_logs` 表中，可通过 `/api/v1/data-io/operation-logs` 查询。
