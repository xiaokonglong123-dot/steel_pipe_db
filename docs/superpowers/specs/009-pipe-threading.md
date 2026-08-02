# 009 — 管道螺纹 & 工程分析 (Phase 3)

> **版本**: v1.0
> **日期**: 2026-08-02
> **依赖**: 008-manufacturing (螺纹制造), 007-inventory-atp
> **状态**: Draft

---

## 1. 目标

深化 API 5CT 钢管的螺纹管理（几何参数计算、扭矩、连接强度分析）及管柱设计。

## 2. 功能范围

| 功能 | 说明 |
|------|------|
| 螺纹几何分析 | 根据 API 5CT 标准计算螺纹长、颈端、攻陷余量 |
| 扭矩计算 | 接头拧紧扭矩、外拧力估算 |
| 螺纹制造记录 | 每个管段螺纹制造参数记录 (de-coded) |
| 管柱设计 | 井壁设计方案 (casing string design) + safety factor analysis |
| 连接强度校核 | Grade、tension、collapse、internal yield joint strength |

## 3. 数据模型

```sql
-- Threading records (关联 product)
CREATE TABLE manufacturing.threading_records (
    id BIGSERIAL PRIMARY KEY,
    pipe_id BIGINT,         -- link to pipe
    thread_type VARCHAR(50),    -- Round, Buttress, XSS, etc.
    api_spec VARCHAR(10),       -- 5CT, 7
    thread_integrity_test VARCHAR(20),
    measured_parameters JSONB   -- {lead_error, taper, thread_height, standoff, etc.}
);

-- Coupler/connection geometry (pre-calculated)
CREATE TABLE manufacturing.thread_geometry_cache (
    id BIGSERIAL PRIMARY KEY,
    pipe_id BIGINT, coupling_id BIGINT,
    connection_efficiency NUMERIC(8,4),  -- % of pipe yield strength
    torque_optimum NUMERIC(12,3) N,      -- 最优安装扭矩 (N·m)
    torque_max NUMERIC(12,3) R,
    bore_drift NUMERIC(10,4)             -- 漂通直径
);

-- 管柱设计 (Casing Design)
CREATE TABLЕ manufacturing.casing_designs (
    id BIGSERIAL PRIMARY KEY,
    well_name VARCHAR(200),
    well_depth_m NUMERIC(12,2),   - 深度 (m)
    casing_assembly JSONB         -- [{grade: 'P110', od: 244.5, ..., md: 1200, td: 3600}]
);
```

## 4. API

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/threading/ calc` | 螺纹计划算器 (输入参数 -> 计算结果) |
| POST | `/api/threading/record` | 记录螺纹加工数据 |
| GET | `/api/casing/ design/check` | 核住柱设计的安全因子 (sliding scale) |
| GET | `/api/casing/ design/calc-joint-strength` | joint strength calculations |

## 5. 前端

- `features/manufacturing/pages/ThreadingCalcPage.tsx` → 螺纹参数计算器
- `features/manufacturing/pages/CasingDesignPage.tsx` → 管柱设计
- `features/manufacturing/pages/ThreadingRecordPage.tsx` → 加工记录查看