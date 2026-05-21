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
    { title: t('Pipe Type'), dataIndex: 'pipe_type', key: 'pipe_type' },
    { title: t('Quantity'), dataIndex: 'quantity', key: 'quantity' },
  ];

  const orderColumns = [
    { title: t('Status'), dataIndex: 'status', key: 'status' },
    { title: t('Count'), dataIndex: 'count', key: 'count' },
  ];

  const activityColumns = [
    { title: t('Action'), dataIndex: 'action', key: 'action' },
    { title: t('Detail'), dataIndex: 'detail', key: 'detail' },
    { title: t('Time'), dataIndex: 'timestamp', key: 'timestamp' },
  ];

  return (
    <div>
      <Title level={3}>{t('Dashboard')}</Title>
      <Row gutter={[16, 16]}>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title={t('Total Pipes')}
              value={data?.total_pipes ?? 0}
              prefix={<DatabaseOutlined />}
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title={t('Total Inventory')}
              value={data?.total_inventory ?? 0}
              prefix={<InboxOutlined />}
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title={t('Pending Orders')}
              value={data?.pending_orders ?? 0}
              prefix={<ClockCircleOutlined />}
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title={t('Recent Quality Certs')}
              value={data?.recent_quality_certs ?? 0}
              prefix={<CheckCircleOutlined />}
            />
          </Card>
        </Col>
      </Row>
      <Row gutter={[16, 16]} style={{ marginTop: 24 }}>
        <Col xs={24} lg={12}>
          <Card title={t('Inventory by Type')}>
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
          <Card title={t('Orders by Status')}>
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
          <Card title={t('Recent Activities')}>
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
