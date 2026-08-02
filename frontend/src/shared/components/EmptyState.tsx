import React from 'react';
import { Empty } from 'antd';

export interface EmptyStateProps {
  /** Optional description text shown below the icon */
  description?: string;
  /** Optional extra content (e.g. a call-to-action button) */
  children?: React.ReactNode;
}

/**
 * Placeholder for empty lists / empty data states.
 *
 * Usage:
 * ```tsx
 * <EmptyState description={t('no_records')} />
 * <EmptyState description={t('no_records')}>
 *   <Button type="primary">Create</Button>
 * </EmptyState>
 * ```
 */
export const EmptyState = React.memo(function EmptyState({
  description,
  children,
}: EmptyStateProps) {
  return (
    <Empty
      image={Empty.PRESENTED_IMAGE_SIMPLE}
      description={description ?? undefined}
      style={{ padding: '32px 0' }}
    >
      {children}
    </Empty>
  );
});
