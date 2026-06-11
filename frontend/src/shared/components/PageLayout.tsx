/**
 * PageLayout — Modern page layout with header, content, and optional footer.
 *
 * Provides consistent page structure:
 * - Page header with title and breadcrumbs
 * - Content area with padding
 * - Optional footer with actions
 * - Loading overlay
 */
import { Card, Space, Typography, Breadcrumb, Skeleton } from 'antd';

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
}

export function PageLayout({
  title,
  subtitle,
  breadcrumbs,
  actions,
  children,
  footer,
  loading = false,
  extra,
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

export default PageLayout;
