import { get, post, put, del } from "./client"
import type {
  Workflow,
  WorkflowInstance,
  WorkflowTask,
  CompleteTaskPayload,
} from "@/types/workflow"

export async function listWorkflows(): Promise<readonly Workflow[]> {
  const r = await get<{ items: readonly Workflow[] }>("/workflow-definitions")
  return r.items
}
export async function getWorkflow(id: number): Promise<Workflow> {
  return get<Workflow>(`/workflow-definitions/${id}`)
}
export async function createWorkflow(payload: { name: string; applies_to: string; is_active?: boolean }): Promise<Workflow> {
  return post<Workflow>("/workflow-definitions", payload)
}
export async function updateWorkflow(id: number, payload: { name: string; applies_to: string; is_active?: boolean }): Promise<Workflow> {
  return put<Workflow>(`/workflow-definitions/${id}`, payload)
}
export async function deleteWorkflow(id: number): Promise<void> {
  return del(`/workflow-definitions/${id}`)
}

export interface InstanceDetail {
  readonly instance: WorkflowInstance
  readonly history: readonly unknown[]
}

export async function listInstances(q: { business_type?: string; status?: string } = {}): Promise<readonly WorkflowInstance[]> {
  const params = new URLSearchParams()
  if (q.business_type) params.set("business_type", q.business_type)
  if (q.status) params.set("status", q.status)
  const s = params.toString()
  const r = await get<{ items: readonly WorkflowInstance[] }>(`/workflow-instances${s ? `?${s}` : ""}`)
  return r.items
}
export async function getInstance(id: number): Promise<InstanceDetail> {
  return get<InstanceDetail>(`/workflow-instances/${id}`)
}

export async function listMyTasks(): Promise<readonly WorkflowTask[]> {
  const r = await get<{ items: readonly WorkflowTask[] }>("/workflow-tasks?mine=true")
  return r.items
}
export async function completeTask(taskId: number, payload: CompleteTaskPayload): Promise<WorkflowInstance> {
  return post<WorkflowInstance>(`/workflow-tasks/${taskId}/complete`, payload)
}