# Steel Pipe DB — 系统设计文档

## 1. 系统架构

### 1.1 整体架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                        用户层 (User Layer)                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   浏览器      │  │   移动端      │  │   第三方系统   │          │
│  │  (React 19)  │  │   (未来)     │  │   (API)      │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      API 网关层 (Gateway Layer)                   │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Vite Dev Server (端口 5173) / Nginx (生产)              │  │
│  │  - 静态资源服务                                            │  │
│  │  - 反向代理 /api/* → :3000                                │  │
│  │  - CORS 处理                                              │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    应用服务层 (Application Layer)                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Axum 0.8 Server (端口 3000)                             │  │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐        │  │
│  │  │ 认证中间件   │  │ RBAC 中间件 │  │ 限流中间件   │        │  │
│  │  │ (JWT/Argon2)│  │ (4 角色)   │  │ (Per-IP)   │        │  │
│  │  └────────────┘  └────────────┘  └────────────┘        │  │
│  │                         │                                │  │
│  │                         ▼                                │  │
│  │  ┌────────────────────────────────────────────────┐    │  │
│  │  │              Handler 层 (16 个模块)              │    │  │
│  │  │  auth │ pipe │ inventory │ purchase │ sales     │    │  │
│  │  │  quality │ contract │ customer │ supplier      │    │  │
│  │  │  report │ label │ data_io │ atp │ check        │    │  │
│  │  └────────────────────────────────────────────────┘    │  │
│  │                         │                                │  │
│  │                         ▼                                │  │
│  │  ┌────────────────────────────────────────────────┐    │  │
│  │  │              Service 层 (19 个模块)              │    │  │
│  │  │  业务逻辑 + 数据验证 + 事务管理                     │    │  │
│  │  └────────────────────────────────────────────────┘    │  │
│  │                         │                                │  │
│  │                         ▼                                │  │
│  │  ┌────────────────────────────────────────────────┐    │  │
│  │  │              Repository 层 (20 个模块)           │    │  │
│  │  │  纯 SQL 查询 + 软删除感知                         │    │  │
│  │  └────────────────────────────────────────────────┘    │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      数据存储层 (Data Layer)                      │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  SQLite (WAL 模式)                                        │  │
│  │  - 数据库文件: ./data/steel_pipe.db                       │  │
│  │  - 连接池: 最大 5 连接                                      │  │
│  │  - 自动迁移: 启动时执行 migrations/                        │  │
│  └──────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  内存缓存 (可选)                                           │  │
│  │  - TTL 缓存: API 5CT 钢级参考数据                          │  │
│  │  - 查询结果缓存: 热点查询                                   │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### 1.2 设计原则

| 原则 | 实现方式 |
|------|----------|
| **单一职责** | Handler 只做提取和响应，Service 处理业务逻辑，Repository 只管数据访问 |
| **依赖倒置** | 通过 Extension 层注入依赖，无全局状态 |
| **软删除** | 所有业务表使用 `deleted_at` 字段，物理删除仅用于维护 |
| **无外键约束** | 应用层保证数据完整性，便于未来分库分表 |
| **数字错误码** | 域前缀编码 (100xx-50001)，便于前端国际化 |

## 2. 组件结构

### 2.1 后端模块结构

```
backend/src/
├── main.rs              ← 入口: tracing, DB 池, 迁移, 启动服务器
├── lib.rs               ← 模块声明
├── config.rs            ← 环境变量配置
├── error.rs             ← AppError 枚举 + 数字错误码
├── response.rs          ← ApiResponse<T>, PaginatedResponse<T>
├── router.rs            ← ~70 个端点组装
├── cache.rs             ← TTL 内存缓存 (新增)
│
├── middleware/          ← 中间件层
│   ├── auth.rs          ← JWT 验证, Claims, AuthContext
│   ├── rbac.rs          ← 角色权限控制
│   └── rate_limit.rs    ← Per-IP 限流
│
├── domain/              ← 领域类型
│   ├── pipe.rs          ← 管材类型枚举
│   ├── inventory.rs     ← 库存状态枚举
│   └── order.rs         ← 订单状态枚举
│
├── dto/                 ← 请求/响应类型 (14 个文件)
│
├── models/              ← 数据库行结构 (12 个文件)
│
├── repositories/        ← 数据访问层 (20 个文件)
│
├── services/            ← 业务逻辑层 (19 个文件)
│
└── handlers/            ← HTTP 处理层 (16 个文件)
```

### 2.2 前端模块结构

```
frontend/src/
├── main.tsx             ← React DOM 入口
├── App.tsx              ← ConfigProvider + QueryClientProvider + RouterProvider
│
├── api/                 ← Axios 实例 + QueryClient 配置
│   ├── client.ts        ← baseURL: '/api/v1', Bearer token 自动附加
│   └── queryClient.ts   ← staleTime: 2min, gcTime: 5min
│
├── routes/              ← 路由配置
│   ├── index.tsx        ← createBrowserRouter
│   └── ProtectedRoute.tsx ← 认证守卫
│
├── features/            ← 13 个功能模块
│   ├── auth/            ← 登录/注册/用户管理
│   ├── pipes/           ← 无缝管/筛管管理
│   ├── inventory/       ← 入库/出库/库存/库位/盘点
│   ├── suppliers/       ← 供应商管理
│   ├── customers/       ← 客户管理
│   ├── purchases/       ← 采购订单
│   ├── sales/           ← 销售订单
│   ├── quality/         ← 质量证书
│   ├── contracts/       ← 合同管理
│   ├── reports/         ← 报表/仪表盘
│   ├── labels/          ← 标签打印
│   ├── search/          ← 全局搜索
│   └── profile/         ← 个人设置
│
├── stores/              ← 客户端状态
│   ├── authStore.ts     ← Zustand: auth_token + auth_user
│   ├── appStore.ts      ← 全局 UI 状态
│   └── unitStore.ts     ← 单位转换
│
├── shared/              ← 共享资源
│   ├── components/      ← 9 个共享组件
│   ├── hooks/           ← useDebounce 等
│   └── utils/           ← 工具函数
│
├── i18n/                ← 国际化 (zh-CN 主要)
│
├── lib/                 ← 运行时验证
│   └── validateResponse.ts ← Zod 响应验证
│
└── zod-schemas/         ← 7 个 Zod 模式文件
```

## 3. 数据流

### 3.1 认证流程

```
用户登录
  │
  ▼
POST /api/v1/auth/login
  │
  ├─→ AuthService::verify_password()
  │     └─→ Argon2id 验证
  │
  ├─→ 生成 access_token (15min) + refresh_token (7d)
  │
  └─→ 返回 { access_token, refresh_token, user }
        │
        ▼
前端存储到 Zustand (localStorage)
  │
  ├─→ apiClient 拦截器自动附加 Authorization: Bearer <token>
  │
  └─→ 401 响应 → 清除存储 → 重定向到 /login
```

### 3.2 入库流程

```
创建入库单
  │
  ▼
POST /api/v1/inventory/inbound
  │
  ├─→ InboundService::create_inbound()
  │     ├─→ 验证供应商存在
  │     ├─→ 生成入库单号 (IN + 年月日 + 序号)
  │     ├─→ 创建 inbound_records 记录
  │     ├─→ 批量创建 inbound_items
  │     └─→ 更新管材状态为 'in_stock'
  │
  └─→ 返回入库单详情

审批入库
  │
  ▼
PUT /api/v1/inventory/inbound/{id}/approve
  │
  ├─→ InboundService::approve_inbound()
  │     ├─→ 验证审批权限 (admin/warehouse)
  │     ├─→ 更新 approval_status = 'approved'
  │     ├─→ 更新 handled_by, handled_at
  │     └─→ 记录操作日志
  │
  └─→ 返回更新后的入库单

执行入库 (批量上架)
  │
  ▼
PUT /api/v1/inventory/inbound/{id}/execute
  │
  ├─→ InboundService::execute_inbound()
  │     ├─→ 更新管材 location_id
  │     ├─→ 更新管材 status = 'in_stock'
  │     ├─→ 创建 inventory_logs 记录
  │     └─→ 记录操作日志
  │
  └─→ 返回执行结果
```

### 3.3 出库流程

```
创建出库单
  │
  ▼
POST /api/v1/inventory/outbound
  │
  ├─→ OutboundService::create_outbound()
  │     ├─→ 验证客户存在
  │     ├─→ ATP 校验 (Available-to-Promise)
  │     │     └─→ 检查库存是否满足出库数量
  │     ├─→ 生成出库单号 (OUT + 年月日 + 序号)
  │     ├─→ 创建 outbound_records 记录
  │     └─→ 批量创建 outbound_items
  │
  └─→ 返回出库单详情

执行出库
  │
  ▼
PUT /api/v1/inventory/outbound/{id}/execute
  │
  ├─→ OutboundService::execute_outbound()
  │     ├─→ 更新管材 status = 'outbound'
  │     ├─→ 清空管材 location_id
  │     ├─→ 创建 inventory_logs 记录
  │     └─→ 记录操作日志
  │
  └─→ 返回执行结果
```

### 3.4 管材生命周期追踪

```
采购订单 → 入库 → 库存 → 出库 → 销售订单
   │        │      │      │        │
   ▼        ▼      ▼      ▼        ▼
purchase  inbound  stock  outbound  sales
_order    _records        _records  _order
   │        │      │      │        │
   └────────┴──────┴──────┴────────┘
                  │
                  ▼
           inventory_logs
           (完整审计追踪)
```

## 4. API 设计

### 4.1 端点总览

| 模块 | 端点 | 方法 | 描述 |
|------|------|------|------|
| **认证** | /api/v1/auth/login | POST | 用户登录 |
| | /api/v1/auth/refresh | POST | 刷新令牌 |
| | /api/v1/auth/logout | POST | 用户登出 |
| | /api/v1/auth/me | GET | 获取当前用户 |
| | /api/v1/auth/change-password | PUT | 修改密码 |
| | /api/v1/users | GET/POST | 用户列表/创建 |
| | /api/v1/users/{id} | PUT/DELETE | 更新/删除用户 |
| **管材** | /api/v1/pipes/seamless | GET/POST | 无缝管列表/创建 |
| | /api/v1/pipes/seamless/{id} | GET/PUT/DELETE | 无缝管详情/更新/删除 |
| | /api/v1/pipes/screen | GET/POST | 筛管列表/创建 |
| | /api/v1/pipes/screen/{id} | GET/PUT/DELETE | 筛管详情/更新/删除 |
| **库存** | /api/v1/inventory/inbound | GET/POST | 入库单列表/创建 |
| | /api/v1/inventory/inbound/{id} | GET | 入库单详情 |
| | /api/v1/inventory/inbound/{id}/approve | PUT | 审批入库 |
| | /api/v1/inventory/inbound/{id}/reject | PUT | 拒绝入库 |
| | /api/v1/inventory/inbound/{id}/execute | PUT | 执行入库 |
| | /api/v1/inventory/outbound | GET/POST | 出库单列表/创建 |
| | /api/v1/inventory/outbound/{id} | GET | 出库单详情 |
| | /api/v1/inventory/outbound/{id}/approve | PUT | 审批出库 |
| | /api/v1/inventory/outbound/{id}/reject | PUT | 拒绝出库 |
| | /api/v1/inventory/outbound/{id}/execute | PUT | 执行出库 |
| | /api/v1/inventory/stock | GET | 库存查询 |
| | /api/v1/inventory/locations | GET/POST | 库位列表/创建 |
| | /api/v1/inventory/check | GET/POST | 盘点列表/创建 |
| **采购** | /api/v1/purchases | GET/POST | 采购订单列表/创建 |
| | /api/v1/purchases/{id} | GET/PUT/DELETE | 采购订单详情/更新/删除 |
| | /api/v1/purchases/{id}/approve | PUT | 审批采购 |
| **销售** | /api/v1/sales | GET/POST | 销售订单列表/创建 |
| | /api/v1/sales/{id} | GET/PUT/DELETE | 销售订单详情/更新/删除 |
| | /api/v1/sales/{id}/approve | PUT | 审批销售 |
| **质量** | /api/v1/quality/certs | GET/POST | 质量证书列表/创建 |
| | /api/v1/quality/certs/{id} | GET/PUT/DELETE | 证书详情/更新/删除 |
| **合同** | /api/v1/contracts | GET/POST | 合同列表/创建 |
| | /api/v1/contracts/{id} | GET/PUT/DELETE | 合同详情/更新/删除 |
| **供应商** | /api/v1/suppliers | GET/POST | 供应商列表/创建 |
| | /api/v1/suppliers/{id} | GET/PUT/DELETE | 供应商详情/更新/删除 |
| **客户** | /api/v1/customers | GET/POST | 客户列表/创建 |
| | /api/v1/customers/{id} | GET/PUT/DELETE | 客户详情/更新/删除 |
| **报表** | /api/v1/reports/dashboard | GET | 仪表盘数据 |
| | /api/v1/reports/inventory | GET | 库存报表 |
| | /api/v1/reports/purchase | GET | 采购报表 |
| | /api/v1/reports/sales | GET | 销售报表 |
| **标签** | /api/v1/labels | GET/POST | 标签列表/生成 |
| **数据导入** | /api/v1/data-io/import/{entity} | POST | 数据导入 |
| | /api/v1/data-io/export/{entity} | GET | 数据导出 |
| **追溯** | /api/v1/trace/pipe/{id} | GET | 管材追溯 |
| | /api/v1/trace/heat/{heat_no} | GET | 炉号追溯 |
| **ATP** | /api/v1/atp/available | GET | 可用量查询 |

### 4.2 请求/响应格式

**分页查询**
```http
GET /api/v1/pipes/seamless?page=1&page_size=20&grade=N80&status=in_stock
```

**响应**
```json
{
  "success": true,
  "request_id": "req_abc123",
  "data": {
    "items": [...],
    "meta": {
      "total": 150,
      "page": 1,
      "page_size": 20,
      "total_pages": 8
    }
  }
}
```

**错误响应**
```json
{
  "success": false,
  "code": 12001,
  "request_id": "req_def456",
  "message": "管材不存在",
  "details": null
}
```

## 5. 数据库设计

### 5.1 核心表结构

```sql
-- 用户表
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    display_name TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('admin', 'warehouse', 'qc', 'sales')),
    email TEXT,
    phone TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);

-- 无缝管表
CREATE TABLE seamless_pipes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pipe_number TEXT NOT NULL UNIQUE,
    batch_number TEXT,
    pipe_type TEXT NOT NULL CHECK (pipe_type IN ('casing', 'tubing')),
    grade TEXT NOT NULL,
    od REAL NOT NULL,
    wt REAL NOT NULL,
    length REAL,
    weight_per_unit REAL,
    end_type TEXT,
    coupling_type TEXT,
    location_id INTEGER,
    status TEXT NOT NULL DEFAULT 'in_stock' CHECK (status IN ('in_stock', 'outbound', 'scrapped')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);

-- 入库单表
CREATE TABLE inbound_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    inbound_no TEXT NOT NULL UNIQUE,
    inbound_type TEXT NOT NULL CHECK (inbound_type IN ('purchase', 'production', 'return')),
    order_id INTEGER,
    supplier_id INTEGER,
    approval_status TEXT NOT NULL DEFAULT 'auto_approved' CHECK (approval_status IN ('auto_approved', 'pending', 'approved', 'rejected')),
    handled_by INTEGER,
    handled_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);

-- 出库单表
CREATE TABLE outbound_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    outbound_no TEXT NOT NULL UNIQUE,
    outbound_type TEXT NOT NULL CHECK (outbound_type IN ('sales', 'transfer', 'scrapped')),
    order_id INTEGER,
    customer_id INTEGER,
    approval_status TEXT NOT NULL DEFAULT 'auto_approved' CHECK (approval_status IN ('auto_approved', 'pending', 'approved', 'rejected')),
    handled_by INTEGER,
    handled_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);

-- 库存变动日志
CREATE TABLE inventory_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pipe_type TEXT NOT NULL,
    pipe_id INTEGER NOT NULL,
    change_type TEXT NOT NULL CHECK (change_type IN ('inbound', 'outbound', 'transfer', 'check_adjust')),
    ref_type TEXT,
    ref_id INTEGER,
    from_location_id INTEGER,
    to_location_id INTEGER,
    notes TEXT,
    created_by INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### 5.2 索引策略

| 表 | 索引 | 用途 |
|----|------|------|
| seamless_pipes | grade, od, wt, status | 组合查询 |
| seamless_pipes | pipe_number | 唯一标识 |
| seamless_pipes | heat_number | 炉号追溯 |
| inbound_records | inbound_no | 单号查询 |
| outbound_records | outbound_no | 单号查询 |
| inventory_logs | pipe_type, pipe_id | 管材追溯 |
| inventory_logs | created_at | 时间范围查询 |
| purchase_orders | order_no, status | 订单查询 |
| sales_orders | order_no, status | 订单查询 |

## 6. 缓存策略

### 6.1 缓存层级

```
┌─────────────────────────────────────────┐
│           浏览器缓存 (HTTP Cache)         │
│  - 静态资源: Cache-Control: max-age=1y   │
│  - API 响应: ETag / Last-Modified        │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│           CDN 缓存 (生产环境)             │
│  - 静态资源分发                           │
│  - 可选: API 响应缓存                     │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│           应用层缓存 (Rust 内存)          │
│  - TTL 缓存: 参考数据 (5 分钟)           │
│  - 查询缓存: 热点查询 (1 分钟)            │
│  - 实现: cache.rs 模块                   │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│           数据库缓存 (SQLite)             │
│  - WAL 模式: 读写并发                    │
│  - 连接池: 最大 5 连接                    │
│  - 索引: 覆盖主要查询路径                 │
└─────────────────────────────────────────┘
```

### 6.2 缓存实现

```rust
// backend/src/cache.rs
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

pub struct CacheEntry<T> {
    pub value: T,
    pub inserted_at: Instant,
}

pub struct Cache<T> {
    store: Arc<RwLock<HashMap<String, CacheEntry<T>>>>,
    ttl: Duration,
}

impl<T: Clone> Cache<T> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    pub async fn get(&self, key: &str) -> Option<T> {
        let store = self.store.read().await;
        store.get(key).and_then(|entry| {
            if entry.inserted_at.elapsed() < self.ttl {
                Some(entry.value.clone())
            } else {
                None
            }
        })
    }

    pub async fn set(&self, key: String, value: T) {
        let mut store = self.store.write().await;
        store.insert(key, CacheEntry {
            value,
            inserted_at: Instant::now(),
        });
    }

    pub async fn invalidate(&self, key: &str) {
        let mut store = self.store.write().await;
        store.remove(key);
    }
}
```

### 6.3 缓存使用场景

| 数据类型 | TTL | 说明 |
|----------|-----|------|
| API 5CT 钢级 | 5 分钟 | 只读参考数据 |
| 库位列表 | 2 分钟 | 低频变更 |
| 用户信息 | 1 分钟 | 登录后缓存 |
| 统计数据 | 30 秒 | 仪表盘缓存 |

## 7. 安全设计

### 7.1 认证机制

- **密码哈希**: Argon2id (OWASP 推荐)
- **访问令牌**: JWT, 15 分钟过期
- **刷新令牌**: JWT, 7 天过期, 数据库存储
- **令牌轮换**: 每次刷新生成新令牌对

### 7.2 授权机制

| 角色 | 权限 |
|------|------|
| admin | 全部权限 |
| warehouse | 管材、库存、采购、供应商读写 |
| qc | 质量证书读写 |
| sales | 销售、客户读写 |

### 7.3 输入验证

- **请求验证**: validator 库 + derive 宏
- **SQL 注入防护**: SQLx 参数化查询
- **XSS 防护**: 前端 React 自动转义
- **CSRF 防护**: SameSite Cookie + Bearer Token

### 7.4 限流策略

| 端点 | 限制 | 说明 |
|------|------|------|
| /auth/login | 5 次/分钟/IP | 防暴力破解 |
| /auth/refresh | 10 次/分钟/IP | 防令牌滥用 |
| /data-io/import | 10 次/分钟/IP | 防资源耗尽 |

## 8. 部署架构

### 8.1 开发环境

```
┌─────────────────────────────────────┐
│         开发机器 (Linux/macOS)       │
│  ┌─────────────┐  ┌─────────────┐  │
│  │ cargo run   │  │ npm run dev │  │
│  │ (端口 3000) │  │ (端口 5173) │  │
│  └─────────────┘  └─────────────┘  │
│           │              │          │
│           └──────┬───────┘          │
│                  ▼                  │
│         SQLite 文件数据库           │
└─────────────────────────────────────┘
```

### 8.2 生产环境

```
┌─────────────────────────────────────────────────┐
│                  负载均衡器                       │
│              (Nginx / Traefik)                   │
└─────────────────────────────────────────────────┘
                       │
         ┌─────────────┼─────────────┐
         ▼             ▼             ▼
┌─────────────┐ ┌─────────────┐ ┌─────────────┐
│  Axum 实例1  │ │  Axum 实例2  │ │  Axum 实例3  │
│  (端口 3000) │ │  (端口 3001) │ │  (端口 3002) │
└─────────────┘ └─────────────┘ └─────────────┘
         │             │             │
         └─────────────┼─────────────┘
                       ▼
              ┌─────────────┐
              │  SQLite DB   │
              │  (共享存储)   │
              └─────────────┘
```

### 8.3 容器化部署

```dockerfile
# Dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libsqlite3-dev
COPY --from=builder /app/target/release/steel-pipe-db /usr/local/bin/
EXPOSE 3000
CMD ["steel-pipe-db"]
```

## 9. 监控与日志

### 9.1 日志格式

```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "level": "INFO",
  "message": "Request completed",
  "request_id": "req_abc123",
  "method": "GET",
  "path": "/api/v1/pipes/seamless",
  "status": 200,
  "duration_ms": 45
}
```

### 9.2 监控指标

- **请求量**: 按端点统计
- **响应时间**: P50/P95/P99
- **错误率**: 按状态码统计
- **数据库**: 连接池使用率、查询耗时

### 9.3 告警规则

- 错误率 > 5% 持续 5 分钟
- P95 响应时间 > 2 秒
- 数据库连接池使用率 > 80%

## 10. 扩展性设计

### 10.1 水平扩展

- **无状态服务**: JWT 认证, 可多实例部署
- **数据库**: SQLite 适合中小规模, 大规模可迁移至 PostgreSQL
- **缓存**: 可引入 Redis 替代内存缓存

### 10.2 垂直扩展

- **连接池**: 调整 max_connections
- **异步处理**: 长时间任务使用后台任务
- **索引优化**: 根据查询模式添加复合索引

### 10.3 未来演进

| 阶段 | 目标 | 技术选型 |
|------|------|----------|
| Phase 1 | 单机部署 | SQLite + 单实例 |
| Phase 2 | 多用户并发 | 连接池优化 + 缓存 |
| Phase 3 | 多站点 | PostgreSQL + Redis |
| Phase 4 | 微服务拆分 | 按领域拆分服务 |

## 11. 实现清单

### 11.1 已完成

- [x] 基础架构搭建
- [x] 认证授权系统
- [x] 管材管理 (无缝管/筛管)
- [x] 库存管理 (入库/出库/盘点)
- [x] 采购/销售订单
- [x] 质量证书管理
- [x] 合同管理
- [x] 供应商/客户管理
- [x] 报表/仪表盘
- [x] 标签打印
- [x] 数据导入导出
- [x] 全局搜索
- [x] 管材追溯
- [x] ATP 可用量查询
- [x] i18n 国际化
- [x] 前端 Zod 验证
- [x] TTL 缓存模块

### 11.2 待优化

- [ ] 前端测试覆盖率提升
- [ ] API 文档自动生成 (OpenAPI)
- [ ] 性能基准测试
- [ ] 安全审计
- [ ] 生产环境配置
- [ ] CI/CD 流水线
- [ ] 监控告警集成

---

**文档版本**: 1.0
**最后更新**: 2024-01-15
**维护者**: Steel Pipe DB Team
