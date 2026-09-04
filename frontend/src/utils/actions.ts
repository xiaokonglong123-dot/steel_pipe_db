// utils/actions.ts — 订单动作 / 状态 → 文案与标签色集中映射
import type { OrderAction } from "@/types/order"

type TagType = "primary" | "success" | "info" | "warning" | "danger" | ""

export const ACTION_LABELS: Record<OrderAction, string> = {
  edit: "编辑",
  delete: "删除",
  submit: "提交",
  approve: "审批通过",
  reject: "驳回",
  cancel: "取消",
  receive: "收货",
  ship: "发货",
}

export const ACTION_TYPES: Record<OrderAction, TagType> = {
  edit: "primary",
  delete: "danger",
  submit: "primary",
  approve: "success",
  reject: "danger",
  cancel: "warning",
  receive: "success",
  ship: "success",
}

/** 采购订单状态映射 */
export const PURCHASE_STATUS: Record<string, { label: string; type: TagType }> = {
  draft: { label: "草稿", type: "info" },
  submitted: { label: "待审批", type: "warning" },
  approved: { label: "已审批", type: "primary" },
  rejected: { label: "已驳回", type: "danger" },
  cancelled: { label: "已取消", type: "info" },
  partially_received: { label: "部分收货", type: "warning" },
  received: { label: "已收货", type: "success" },
}

/** 销售订单状态映射 */
export const SALES_STATUS: Record<string, { label: string; type: TagType }> = {
  draft: { label: "草稿", type: "info" },
  submitted: { label: "待审批", type: "warning" },
  approved: { label: "已审批", type: "primary" },
  rejected: { label: "已驳回", type: "danger" },
  cancelled: { label: "已取消", type: "info" },
  shipped: { label: "已发货", type: "success" },
}

/** 工作流实例状态映射 */
export const INSTANCE_STATUS: Record<string, { label: string; type: TagType }> = {
  running: { label: "进行中", type: "warning" },
  completed: { label: "已完成", type: "success" },
  cancelled: { label: "已取消", type: "info" },
}

/** 任务状态映射 */
export const TASK_STATUS: Record<string, { label: string; type: TagType }> = {
  pending: { label: "待办", type: "warning" },
  completed: { label: "已完成", type: "success" },
  skipped: { label: "已跳过", type: "info" },
}