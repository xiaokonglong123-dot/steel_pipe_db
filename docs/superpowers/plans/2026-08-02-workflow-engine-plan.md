# Workflow Engine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended)

**Goal:** Build a BPMN-like workflow engine in Rust with definition JSON store, instance execution engine, and approval nodes.

**Architecture:** Module `backend/src/workflow/` (services.rs logic). Shared instance progress via `current_step_index`. Frontend: react-flow based designer page + user approval list with approve/reject/delegate actions plus notification integration.

**Tech Stack:** Rust Axum 0.8, serde-json, TanStack Query, React 19
**Database:** SQLite3 (`sqlite://data/erp.db?mode=rwc`), 无 schema 前缀，JSON 存 TEXT

---

### Task 1: Create workflow schema

```sql
CREATE TABLE IF NOT EXISTS workflow_definitions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT, model_type TEXT, definition TEXT NOT NULL, status TEXT
);
CREATE TABLE IF NOT EXISTS workflow_instances (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    definition_id INTEGER, started_by INTEGER,
    status TEXT, entity_type TEXT, entity_id INTEGER
);
CREATE TABLE IF NOT EXISTS approval_nodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    instance_id INTEGER, step_id TEXT,
    assignee_type TEXT, status TEXT
);
```

### Task 2: Workflow service logic

- `WorkflowService::start(def_id, entity_id) → create instance, parse def JSON, create first node, notify assignee`
- `WorkflowService::approve(node_id, reason) → update node, advance to next step`

### Task 3: Add API endpoints

POST `/api/workflow/tasks/:nodeId/approve`, POST `/api/workflow/tasks/:nodeId/reject`, POST `/api/workflow/tasks/delegate`

### Task 4: Frontend

Create `WorkflowApprovalListPage.tsx`, `WorkflowApprovalDetailPage.tsx`, `WorkflowDesignerPage.tsx` (react-flow drag-drop)

---
