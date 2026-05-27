/**
 * 页面标题栏 — 标题 + 副标题 + 右侧操作区
 *
 * 用于业务页面顶部，左侧展示页面标题与副标题，
 * 右侧通过 extra 插槽放置操作按钮（如"新建"）。
 */
import { Typography, Space } from 'antd';
import type { ReactNode } from 'react';

const { Title, Text } = Typography;

interface PageHeaderProps {
  title: string;
  subtitle?: string;
  extra?: ReactNode;
}

export default function PageHeader({ title, subtitle, extra }: PageHeaderProps) {
  return (
    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: 24, flexWrap: 'wrap', gap: 12 }}>
      <div>
        <Title level={4} style={{ margin: 0 }}>{title}</Title>
        {subtitle && <Text type="secondary">{subtitle}</Text>}
      </div>
      {extra && <Space wrap>{extra}</Space>}
    </div>
  );
}
