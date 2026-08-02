// Workflow API — definitions, instances, tasks (approval engine)
import apiClient from '@/api/client';
import type { ApiResponse } from '@/types';
import { validateResponse } from '@/lib/validateResponse';
import { z } from 'zod';

export interface WorkflowDefinition {
  id: number;
  name: string;
  entity_type: string;
  description: string | null;
  definition_json?: unknown;
  callback_action: string | null;
  is_active: boolean;
}

export interface ApprovalTask {
  id: number;
  instance_id: number;
  step_index: number;
  node_key: string;
  assignee_type: string;
  assignee_value: string | null;
  status: string;
  approver_id: number | null;
  approval_reason: string | null;
  due_date: string | null;
}

export interface WorkflowInstance {
  id: number;
  definition_id: number;
  entity_type: string;
  entity_id: number;
  amount: string | null;
  status: string;
  current_step: number;
}

const defSchema = z.object({
  id: z.number(),
  name: z.string(),
  entity_type: z.string(),
  description: z.string().nullable(),
  definition_json: z.unknown(),
  callback_action: z.string().nullable(),
  is_active: z.boolean(),
}).passthrough();

const taskSchema = z.object({
  id: z.number(),
  instance_id: z.number(),
  step_index: z.number(),
  node_key: z.string(),
  assignee_type: z.string(),
  assignee_value: z.string().nullable(),
  status: z.string(),
  approver_id: z.number().nullable(),
  approval_reason: z.string().nullable(),
  due_date: z.string().nullable(),
}).passthrough();

const instanceSchema = z.object({
  id: z.number(),
  definition_id: z.number(),
  entity_type: z.string(),
  entity_id: z.number(),
  amount: z.string().nullable(),
  status: z.string(),
  current_step: z.number(),
}).passthrough();

const arrayOf = <T extends z.ZodTypeAny>(item: T) =>
  z.object({ success: z.boolean(), data: z.array(item) }).passthrough();

export const workflowApi = {
  listDefinitions: async () => {
    const res = await apiClient.get<ApiResponse<WorkflowDefinition[]>>('/workflows/definitions');
    return validateResponse(arrayOf(defSchema), res.data).data;
  },

  createDefinition: async (data: {
    name: string;
    entity_type: string;
    description?: string;
    nodes: unknown[];
  }) => {
    const res = await apiClient.post<ApiResponse<WorkflowDefinition>>('/workflows/definitions', data);
    return validateResponse(defSchema, res.data).data;
  },

  deleteDefinition: async (id: number) => {
    await apiClient.delete(`/workflows/definitions/${id}`);
  },

  startInstance: async (data: {
    definition_id: number;
    entity_type: string;
    entity_id: number;
    amount?: number;
  }) => {
    const res = await apiClient.post<ApiResponse<WorkflowInstance>>('/workflows/instances', data);
    return validateResponse(instanceSchema, res.data).data;
  },

  myTasks: async () => {
    const res = await apiClient.get<ApiResponse<ApprovalTask[]>>('/workflows/my-tasks');
    return validateResponse(arrayOf(taskSchema), res.data).data;
  },

  approveTask: async (nodeId: number, reason?: string) => {
    const res = await apiClient.post<ApiResponse<WorkflowInstance>>(
      `/workflows/tasks/${nodeId}/approve`,
      { reason },
    );
    return validateResponse(instanceSchema, res.data).data;
  },

  rejectTask: async (nodeId: number, reason: string) => {
    const res = await apiClient.post<ApiResponse<WorkflowInstance>>(
      `/workflows/tasks/${nodeId}/reject`,
      { reason },
    );
    return validateResponse(instanceSchema, res.data).data;
  },
};
