import { useQuery } from '@tanstack/react-query';
import { Card, Col, Row, Statistic, Table, Tag } from 'antd';
import { DollarOutlined, FileDoneOutlined, WalletOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { biApi, type SalesTrendRow } from '../api/biApi';
import { biQueryKeys } from '../queryKeys';
import { PageLayout } from '@/shared/components/PageLayout';

export default function DashboardPage() {
  const { t } = useTranslation('bi');

  const { data: fin } = useQuery({ queryKey: biQueryKeys.financeSummary, queryFn: biApi.financeSummary });
  const { data: inventory } = useQuery({ queryKey: biQueryKeys.inventoryValue, queryFn: biApi.inventoryValue });
  const { data: trend } = useQuery({ queryKey: biQueryKeys.salesTrend(12), queryFn: () => biApi.salesTrend(12) });

  const invColumns = [
    { title: t('pipeType'), dataIndex: 'pipe_type', key: 'pipe_type' },
    { title: t('onHand'), dataIndex: 'on_hand', key: 'on_hand' },
  ];
  const trendColumns = [
    { title: t('month'), dataIndex: 'month', key: 'month' },
    { title: t('status'), dataIndex: 'status', key: 'status', render: (v: string) => <Tag>{v}</Tag> },
    { title: t('orders'), dataIndex: 'order_count', key: 'order_count' },
    { title: t('amount'), dataIndex: 'total_amount', key: 'total_amount' },
  ];

  const approved = (trend ?? []).filter((r: SalesTrendRow) => r.status === 'approved');
  const totalSales = approved.reduce((s, r) => s + Number(r.total_amount), 0);

  return (
    <PageLayout title={t('title')}>
      <Row gutter={16} style={{ marginBottom: 16 }}>
        <Col span={6}>
          <Card><Statistic title={t('postedEntries')} value={fin?.posted_entries ?? 0} prefix={<FileDoneOutlined />} /></Card>
        </Col>
        <Col span={6}>
          <Card><Statistic title={t('openAr')} value={fin?.open_ar ?? 0} prefix={<DollarOutlined />} precision={2} /></Card>
        </Col>
        <Col span={6}>
          <Card><Statistic title={t('openAp')} value={fin?.open_ap ?? 0} prefix={<WalletOutlined />} precision={2} /></Card>
        </Col>
        <Col span={6}>
          <Card><Statistic title={t('sales12m')} value={totalSales} precision={2} /></Card>
        </Col>
      </Row>

      <Row gutter={16}>
        <Col span={10}>
          <Card title={t('inventoryValue')} style={{ marginBottom: 16 }}>
            <Table rowKey="pipe_type" dataSource={inventory ?? []} columns={invColumns} pagination={false} size="small" />
          </Card>
        </Col>
        <Col span={14}>
          <Card title={t('salesTrend')}>
            <Table rowKey={(r) => `${r.month}-${r.status}`} dataSource={trend ?? []} columns={trendColumns} pagination={false} size="small" />
          </Card>
        </Col>
      </Row>
    </PageLayout>
  );
}
