import { Empty, Button } from 'antd';
import type { ReactNode } from 'react';

interface EmptyStateProps {
  description?: string;
  actionText?: string;
  onAction?: () => void;
  extra?: ReactNode;
}

export default function EmptyState({ description = '暂无数据', actionText, onAction, extra }: EmptyStateProps) {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', padding: 48 }}>
      <Empty description={description} />
      {actionText && onAction && (
        <Button type="primary" onClick={onAction} style={{ marginTop: 16 }}>
          {actionText}
        </Button>
      )}
      {extra && <div style={{ marginTop: 16 }}>{extra}</div>}
    </div>
  );
}
