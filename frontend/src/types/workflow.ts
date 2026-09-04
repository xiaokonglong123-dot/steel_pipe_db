import type { Id } from "./common"

export interface Workflow {
  readonly id: Id
  readonly name: string
  readonly applies_to: string
  readonly is_active: number
  readonly created_at: string
}

export interface WorkflowState {
  readonly id: Id
  readonly workflow_id: number
  readonly state_key: string
  readonly doc_status: number
  readonly is_initial: number
  readonly is_final: number
}

export interface WorkflowTransition {
  readonly id: Id
  readonly workflow_id: number
  readonly from_state_id: number
  readonly to_state_id: number
  readonly action: string
  readonly required_role: string | null
  readonly is_auto: number
  readonly amount_threshold: string | null
}

export interface WorkflowInstance {
  readonly id: Id
  readonly workflow_id: number
  readonly business_type: string
  readonly business_id: number
  readonly current_state: string
  readonly status: string
  readonly created_at: string
  readonly updated_at: string
}

export interface WorkflowTask {
  readonly id: Id
  readonly instance_id: number
  readonly state_key: string
  readonly assignee_id: number | null
  readonly status: string
  readonly action: string | null
  readonly comment: string | null
  readonly created_at: string
  readonly completed_at: string | null
}

export interface CompleteTaskPayload {
  action: string
  comment?: string | null
}