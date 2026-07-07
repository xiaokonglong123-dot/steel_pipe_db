/**
 * PageLayout — Modern page layout with header, content, and optional footer.
 *
 * Provides consistent page structure:
 * - Page header with title and breadcrumbs
 * - Content area with padding
 * - Optional footer with actions
 * - Loading overlay
 */
import React from 'react';
import { Button, Card, Space, Typography, Breadcrumb, Skeleton } from 'antd';
import { ArrowLeftOutlined } from '@ant-design/icons';

const { Title, Text } = Typography;

export interface PageLayoutProps {
  /** Page title */
  title: string;
  /** Optional subtitle or description */
  subtitle?: string;
  /** Breadcrumb items */
  breadcrumbs?: Array<{ label: string; path?: string }>;
  /** Header action buttons */
  actions?: React.ReactNode;
  /** Page content */
  children: React.ReactNode;
  /** Footer content */
  footer?: React.ReactNode;
  /** Loading state */
  loading?: boolean;
  /** Extra content in header */
  extra?: React.ReactNode;
  /** Back button click handler — shows ArrowLeft button when provided */
  onBack?: () => void;
}

function PageLayoutInner({
  title,
  subtitle,
  breadcrumbs,
  actions,
  children,
  footer,
  loading = false,
  extra,
  onBack,
}: PageLayoutProps) {
  return (
    <div style={{ padding: '24px' }}>
      {/* Breadcrumbs */}
      {breadcrumbs && breadcrumbs.length > 0 && (
        <Breadcrumb
          items={breadcrumbs.map((item) => ({
            title: item.path ? (
              <a href={item.path}>{item.label}</a>
            ) : (
              item.label
            ),
          }))}
          style={{ marginBottom: 16 }}
        />
      )}

      {/* Header */}
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'flex-start',
          marginBottom: 24,
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
          {onBack && (
            <Button
              type="text"
              icon={<ArrowLeftOutlined />}
              onClick={onBack}
              style={{ fontSize: 16 }}
            />
          )}
          <div>
            <Title level={4} style={{ margin: 0 }}>
              {title}
            </Title>
            {subtitle && (
              <Text type="secondary" style={{ marginTop: 4, display: 'block' }}>
                {subtitle}
              </Text>
            )}
          </div>
        </div>
        <Space>
          {extra}
          {actions}
        </Space>
      </div>

      {/* Content */}
      <Card loading={loading}>
        {loading ? (
          <Skeleton active paragraph={{ rows: 4 }} />
        ) : (
          children
        )}
      </Card>

      {/* Footer */}
      {footer && (
        <div
          style={{
            marginTop: 24,
            padding: '16px 24px',
            background: '#fafafa',
            borderRadius: 8,
            display: 'flex',
            justifyContent: 'flex-end',
            gap: 12,
          }}
        >
          {footer}
        </div>
      )}
    </div>
  );
}

export const PageLayout = React.memo(PageLayoutInner);

export default PageLayout;
