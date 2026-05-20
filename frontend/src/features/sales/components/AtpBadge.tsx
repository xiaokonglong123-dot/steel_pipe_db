import React from 'react';
import { Tag } from 'antd';
import type { AtpResult } from '../api/salesApi';

interface Props {
  result: AtpResult;
}

export default function AtpBadge({ result }: Props) {
  return (
    <Tag color={result.available ? 'green' : 'red'}>
      {result.available ? '充足' : '不足'}
      {' '}
      (可售:{result.available_qty}/{result.requested_qty})
    </Tag>
  );
}
