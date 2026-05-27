/**
 * 状态标签 — 根据状态值自动匹配颜色
 *
 * 内置常见业务状态的色值映射（如 active→绿、rejected→红、draft→灰），
 * 支持自定义显示文字。适用于列表页状态列。
 */
import { Tag } from 'antd';

const STATUS_COLOR: Record<string, string> = {
  active: 'green',
  inactive: 'default',
  pending: 'orange',
  approved: 'green',
  rejected: 'red',
  draft: 'default',
  completed: 'blue',
  cancelled: 'default',
  delivered: 'purple',
  invoiced: 'cyan',
  in_stock: 'green',
  outbound: 'blue',
  scrapped: 'red',
  inprogress: 'processing',
  true: 'green',
  false: 'red',
  match: 'green',
  mismatch: 'red',
  found: 'green',
  missing: 'red',
  damaged: 'orange',
  in_progress: 'processing',
};

interface StatusTagProps {
  status: string;
  label?: string;
}

export default function StatusTag({ status, label }: StatusTagProps) {
  const color = STATUS_COLOR[status] || 'default';
  return <Tag color={color}>{label || status}</Tag>;
}
