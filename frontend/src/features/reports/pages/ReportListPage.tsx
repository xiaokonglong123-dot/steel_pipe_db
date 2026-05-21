import { Card, Row, Col, Typography } from 'antd';
import {
  BarChartOutlined,
  ShoppingCartOutlined,
  SafetyCertificateOutlined,
  DashboardOutlined,
} from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';

const { Title } = Typography;

export default function ReportListPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();

  const reports = [
    {
      title: t('Dashboard'),
      icon: <DashboardOutlined style={{ fontSize: 36, color: '#1890ff' }} />,
      path: '/reports/dashboard',
    },
    {
      title: t('Inventory Summary'),
      icon: <BarChartOutlined style={{ fontSize: 36, color: '#52c41a' }} />,
      path: '/reports/inventory',
    },
    {
      title: t('Order Report'),
      icon: <ShoppingCartOutlined style={{ fontSize: 36, color: '#faad14' }} />,
      path: '/reports/orders',
    },
    {
      title: t('Quality Report'),
      icon: <SafetyCertificateOutlined style={{ fontSize: 36, color: '#722ed1' }} />,
      path: '/reports/quality',
    },
  ];

  return (
    <div>
      <Title level={3}>{t('Reports')}</Title>
      <Row gutter={[16, 16]}>
        {reports.map((report) => (
          <Col key={report.path} xs={24} sm={12} lg={6}>
            <Card
              hoverable
              onClick={() => navigate(report.path)}
              style={{ textAlign: 'center' }}
            >
              <div style={{ marginBottom: 12 }}>{report.icon}</div>
              <Title level={5}>{report.title}</Title>
            </Card>
          </Col>
        ))}
      </Row>
    </div>
  );
}
