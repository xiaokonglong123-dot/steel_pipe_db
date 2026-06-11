import React from 'react';
import { Tag } from 'antd';
import { useTranslation } from 'react-i18next';
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
  labelKey: string;
}

const STATUS_ICON_MAP: Record<string, { color: string; icon: React.ReactNode; labelKey: string }> = {
  approved: { color: 'success', icon: <CheckCircleOutlined />, labelKey: 'status.approved' },
  completed: { color: 'success', icon: <CheckCircleOutlined />, labelKey: 'status.completed' },
  delivered: { color: 'success', icon: <CheckCircleOutlined />, labelKey: 'status.delivered' },
  paid: { color: 'success', icon: <CheckCircleOutlined />, labelKey: 'status.paid' },
  active: { color: 'success', icon: <CheckCircleOutlined />, labelKey: 'status.active' },
  rejected: { color: 'error', icon: <CloseCircleOutlined />, labelKey: 'status.rejected' },
  cancelled: { color: 'error', icon: <CloseCircleOutlined />, labelKey: 'status.cancelled' },
  inactive: { color: 'error', icon: <CloseCircleOutlined />, labelKey: 'status.inactive' },
  pending: { color: 'warning', icon: <ClockCircleOutlined />, labelKey: 'status.pending' },
  draft: { color: 'default', icon: <ClockCircleOutlined />, labelKey: 'status.draft' },
  processing: { color: 'processing', icon: <SyncOutlined spin />, labelKey: 'status.processing' },
  in_progress: { color: 'processing', icon: <SyncOutlined spin />, labelKey: 'status.in_progress' },
};

export interface StatusBadgeProps {
  status: string;
  dot?: boolean;
  label?: string;
}

export function StatusBadge({ status, dot = false, label }: StatusBadgeProps) {
  const { t } = useTranslation('common');
  const config = STATUS_ICON_MAP[status] || {
    color: 'default',
    icon: <ExclamationCircleOutlined />,
    labelKey: status,
  };

  const displayLabel = label || t(config.labelKey, status);

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
