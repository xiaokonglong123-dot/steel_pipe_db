# 012 — 通知平台 (Phase 4)

> **版本**: v1.0 | **日期**: 2026-08-02 | **依赖**: 001-auth | **状态**: Draft

---

## 1. 目标

统一各模块的通知发送：站内信、邮件、企业微信/钉钉、管理个人偏好。

## 2. 功能

| 功能 | 描述 |
|------|------|
| 通知模板 | 支持变量替换（`{user_name}`, `{order_no}`） |
| 通道管理 | 站内信，email，短信，机器人推送 |
| 用户偏好 | 去掉哪些推送方式、渠道 (仅站内信，仅email，both) |
| 通知历史 | query 追 light 通知记录 |

## 3. 数据模型

```sql
CREATE TABLE notification.templates (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(100),   -- 'PO_Approved', 'Task_Assigned'
    channels VARCHAR(50)[] DEFAULT ARRAY['in_app','email'],  -- {in_app, email, sms, push}
    subject VARCHAR(500), body TEXT,
    created_at TIMESTAMPTZ
);

CREATE TABLE notification.notifications (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT REFERENCES auth.users(id),
    template_id BIGINT,
    title VARCHAR(500),
    body TEXT,
    entity_type VARCHAR(100), entity_id BIGINT,
    delivered_channels VARCHAR(50)[],
    read BOOLEAN DEFAULT false,
    read_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ
);

CREATE TABLE notification.user_preferences (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT UNIQUE,
    notify_on_email BOOLEAN DEFAULT TRUE,
    notify_in_app BOOLEAN DEFAULT TRUE
);
```

## 4. API

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/notifications` | 未读通知列表 |
| POST | `/api/notifications/:id/read` | 标记已读 |
| GET | `/api/notifications/preferences` | 偏好 |
| PUT | `/api/notifications/preferences` | Update error model in system |

## 5. 前端

- `features/notification/pages/NotificationCenterPage.tsx`
- `NotificationBell` → 侧栏旁边实时通知铃铛 (WebSocket)