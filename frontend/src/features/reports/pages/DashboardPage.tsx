import { Card, Row, Col, Statistic, Spin, Table, Typography } from 'antd';
import {
  DatabaseOutlined,
  InboxOutlined,
  ClockCircleOutlined,
  CheckCircleOutlined,
} from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { useDashboard } from '../hooks/useReports';

const { Title } = Typography;

export default function DashboardPage() {
  const { t } = useTranslation();
  const { data, isLoading } = useDashboard();

  if (isLoading) return <Spin size="large" style={{ display: 'block', margin: '60px auto' }} />;

  const inventoryColumns = [
    { title: t('reports.pipe_type'), dataIndex: 'pipe_type', key: 'pipe_type' },
    { title: t('reports.quantity'), dataIndex: 'quantity', key: 'quantity' },
  ];

  const orderColumns = [
    { title: t('reports.status'), dataIndex: 'status', key: 'status' },
    { title: t('reports.count'), dataIndex: 'count', key: 'count' },
  ];

  const activityColumns = [
    { title: t('reports.action'), dataIndex: 'action', key: 'action' },
    { title: t('reports.detail'), dataIndex: 'detail', key: 'detail' },
    { title: t('reports.time'), dataIndex: 'timestamp', key: 'timestamp' },
  ];

  return (
    <div>
      <Title level={3}>{t('reports.dashboard')}</Title>
      <Row gutter={[16, 16]}>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title={t('reports.total_pipes')}
              value={data?.total_pipes ?? 0}
              prefix={<DatabaseOutlined />}
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title={t('reports.total_inventory')}
              value={data?.total_inventory ?? 0}
              prefix={<InboxOutlined />}
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title={t('reports.pending_orders')}
              value={data?.pending_orders ?? 0}
              prefix={<ClockCircleOutlined />}
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title={t('reports.recent_quality_certs')}
              value={data?.recent_quality_certs ?? 0}
              prefix={<CheckCircleOutlined />}
            />
          </Card>
        </Col>
      </Row>
      <Row gutter={[16, 16]} style={{ marginTop: 24 }}>
        <Col xs={24} lg={12}>
          <Card title={t('reports.inventory_by_type')}>
            <Table
              columns={inventoryColumns}
              dataSource={data?.inventory_by_type}
              rowKey="pipe_type"
              pagination={false}
              size="small"
            />
          </Card>
        </Col>
        <Col xs={24} lg={12}>
          <Card title={t('reports.orders_by_status')}>
            <Table
              columns={orderColumns}
              dataSource={data?.orders_by_status}
              rowKey="status"
              pagination={false}
              size="small"
            />
          </Card>
        </Col>
      </Row>
      <Row gutter={[16, 16]} style={{ marginTop: 16 }}>
        <Col span={24}>
          <Card title={t('reports.recent_activities')}>
            <Table
              columns={activityColumns}
              dataSource={data?.recent_activities}
              rowKey="id"
              pagination={false}
              size="small"
            />
          </Card>
        </Col>
      </Row>
    </div>
  );
}
