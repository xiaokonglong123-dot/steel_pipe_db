# 012 — 通知平台 (Phase 4)

> **版本**: v1.0 | **日期**: 2026-08-02 | **依赖**: 001-auth | **状态**: Draft

---

## 1. 目标

统一各模块的通知发送：站内信、邮件、企业微信/钉钉、管理个人偏好。

## 2. 功能

| 功能 | 描述 |
| ------ | ------ |
| 通知模板 | 支持变量替换（`{user_name}`, `{order_no}`） |
| 通道管理 | 站内信，email，短信，机器人推送 |
| 用户偏好 | 选择推送方式、渠道 (仅站内信，仅email，both) |
| 通知历史 | 查询通知记录 |

## 3. 数据模型

> 数据库为 SQLite3（`sqlite://data/erp.db?mode=rwc`），通道数组以 JSON 文本存储。

```sql
CREATE TABLE notification_templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT,   -- 'PO_Approved', 'Task_Assigned'
    channels TEXT,  -- JSON 数组 ["in_app","email"]
    subject TEXT,
    body TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE notifications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER REFERENCES users(id),
    template_id INTEGER,
    title TEXT,
    body TEXT,
    entity_type TEXT,
    entity_id INTEGER,
    delivered_channels TEXT,  -- JSON 数组
    read INTEGER DEFAULT 0,
    read_at TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE user_preferences (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER UNIQUE,
    notify_on_email INTEGER DEFAULT 1,
    notify_in_app INTEGER DEFAULT 1
);
```

## 4. API

| Method | Path | Description |
| -------- | ------ | ------------- |
| GET | `/api/notifications` | 未读通知列表 |
| POST | `/api/notifications/:id/read` | 标记已读 |
| GET | `/api/notifications/preferences` | 偏好 |
| PUT | `/api/notifications/preferences` | Update preferences |

## 5. 前端

- `features/notification/pages/NotificationCenterPage.tsx`
- `NotificationBell` → 侧栏旁边实时通知铃铛 (WebSocket)
