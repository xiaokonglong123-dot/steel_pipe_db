import React from 'react';
import { Tag } from 'antd';
import type { OrderStatus } from '../../../shared/types';

const statusConfig: Record<OrderStatus, { color: string; label: string }> = {
  draft: { color: 'default', label: '草稿' },
  pending: { color: 'orange', label: '待审核' },
  approved: { color: 'blue', label: '已审核' },
  completed: { color: 'green', label: '已完成' },
  cancelled: { color: 'red', label: '已取消' },
};

interface Props {
  status: OrderStatus;
}

export default function OrderStatusTag({ status }: Props) {
  const cfg = statusConfig[status] || { color: 'default', label: status };
  return <Tag color={cfg.color}>{cfg.label}</Tag>;
}
