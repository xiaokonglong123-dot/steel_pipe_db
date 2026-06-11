/**
 * StatusBadge — Modern status badge with color coding and icons.
 *
 * Provides consistent status display patterns:
 * - Color coding based on status
 * - Optional icon
 * - Custom status mapping
 */
import React from 'react';
import { Tag } from 'antd';
import {
  CheckCircleOutlined,
  CloseCircleOutlined,
  ClockCircleOutlined,
  SyncOutlined,
  ExclamationCircleOutlined,
} from '@ant-design/icons';

export type StatusType = 'success' | 'error' | 'warning' | 'processing' | 'default';

export interface StatusConfig {
  color: string;
  icon?: React.ReactNode;
  label: string;
}

const DEFAULT_STATUS_MAP: Record<string, StatusConfig> = {
  // Success states
  approved: { color: 'success', icon: <CheckCircleOutlined />, label: '已审批' },
  completed: { color: 'success', icon: <CheckCircleOutlined />, label: '已完成' },
  delivered: { color: 'success', icon: <CheckCircleOutlined />, label: '已交付' },
  paid: { color: 'success', icon: <CheckCircleOutlined />, label: '已付款' },
  active: { color: 'success', icon: <CheckCircleOutlined />, label: '启用' },
  // Error states
  rejected: { color: 'error', icon: <CloseCircleOutlined />, label: '已拒绝' },
  cancelled: { color: 'error', icon: <CloseCircleOutlined />, label: '已取消' },
  inactive: { color: 'error', icon: <CloseCircleOutlined />, label: '停用' },
  // Warning states
  pending: { color: 'warning', icon: <ClockCircleOutlined />, label: '待处理' },
  draft: { color: 'default', icon: <ClockCircleOutlined />, label: '草稿' },
  // Processing states
  processing: { color: 'processing', icon: <SyncOutlined spin />, label: '处理中' },
  in_progress: { color: 'processing', icon: <SyncOutlined spin />, label: '进行中' },
};

export interface StatusBadgeProps {
  /** Status value */
  status: string;
  /** Custom status map (overrides defaults) */
  statusMap?: Record<string, StatusConfig>;
  /** Show as dot instead of tag */
  dot?: boolean;
  /** Custom label override */
  label?: string;
}

export function StatusBadge({
  status,
  statusMap = DEFAULT_STATUS_MAP,
  dot = false,
  label,
}: StatusBadgeProps) {
  const config = statusMap[status] || {
    color: 'default',
    icon: <ExclamationCircleOutlined />,
    label: status,
  };

  const displayLabel = label || config.label;

  if (dot) {
    return (
      <span style={{ display: 'inline-flex', alignItems: 'center', gap: 8 }}>
        <Tag color={config.color} />
        <span>{displayLabel}</span>
      </span>
    );
  }

  return (
    <Tag color={config.color} icon={config.icon}>
      {displayLabel}
    </Tag>
  );
}

export default StatusBadge;
