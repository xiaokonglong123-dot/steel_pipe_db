# Workflow Engine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended)

**Goal:** Build a BPMN-like workflow engine in Rust with definition JSON store, instance execution engine, and approval nodes.

**Architecture:** Crate `workflow/` with engine.rs (logic). Shared instance progress via `current_step_index`. Frontend: react-flow based designer page + user approval list with approve/reject/delegate actions plus notification integration.

**Tech Stack:** Rust Axum 0.8, serde-json, TanStack Query, React 19

---

### Task 1: Create workflow schema

```sql
CREATE SCHEMA workflow;
CREATE TABLE workflow.definitions (id BIGSERIAL, name TEXT, model_type TEXT, definition JSONB NOT NULL, status TEXT);
CREATE TABLE workflow.instances(id, ..., started_by, status, entity_type, entity_id);
CREATE TABLE workflow.approval_nodes(instance_id, step_id, assignee_type, status);
```

### Task 2: Workflow service logic

- `WorkflowService::start(def_id, entity_id) → create instance, parse def JSON, create first node, notify assignee`
- `WorkflowService::approve(node_id, reason) → update node, advance to next step`

### Task 3: Add API endpoints

POST `/api/workflow/tasks/:nodeId/approve`, POST `/api/workflow/tasks/:nodeId/reject`, POST `/api/workflow/tasks/delegate`

### Task 4: Frontend

Create `WorkflowApprovalListPage.tsx`, `WorkflowApprovalDetailPage.tsx`, `WorkflowDesignerPage.tsx` (react-flow drag-drop)

---