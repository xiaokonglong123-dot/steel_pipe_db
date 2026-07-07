import React from 'react';
import { Tag } from 'antd';
import { useTranslation } from 'react-i18next';

export interface StatusTagProps {
  /** Status value (e.g., 'in_stock', 'pending', 'approved') */
  status: string;
  /** Optional custom label */
  label?: string;
  /** Show as dot style */
  dot?: boolean;
}

/**
 * Simple status tag component - wrapper around Tag with consistent status colors.
 * 
 * Usage:
 * ```tsx
 * <StatusTag status="in_stock" />
 * <StatusTag status="pending" label="Waiting" />
 * <StatusTag status="approved" dot />
 * ```
 */
export const StatusTag = React.memo(function StatusTag({ status, label, dot = false }: StatusTagProps) {
  const { t } = useTranslation('common');
  
  // Status color mapping
  const statusColors: Record<string, 'success' | 'warning' | 'error' | 'processing' | 'default'> = {
    // Active/positive states
    in_stock: 'success',
    approved: 'success',
    completed: 'success',
    delivered: 'success',
    paid: 'success',
    active: 'success',
    received: 'success',
    shipped: 'success',
    passed: 'success',
    
    // Pending/waiting states
    pending: 'warning',
    draft: 'warning',
    processing: 'warning',
    in_progress: 'warning',
    awaiting: 'warning',
    partial: 'warning',
    
    // Error/negative states
    rejected: 'error',
    cancelled: 'error',
    scrapped: 'error',
    failed: 'error',
    inactive: 'error',
    outbound: 'error',
    out_of_stock: 'error',
    
    // Neutral states
    new: 'default',
    returned: 'default',
    transferred: 'default',
  };

  const color = statusColors[status] || 'default';
  const displayLabel = label || t(`status.${status}`, status);

  if (dot) {
    return (
      <span style={{ display: 'inline-flex', alignItems: 'center', gap: 6 }}>
        <Tag color={color} />
        <span>{displayLabel}</span>
      </span>
    );
  }

  return <Tag color={color}>{displayLabel}</Tag>;
});