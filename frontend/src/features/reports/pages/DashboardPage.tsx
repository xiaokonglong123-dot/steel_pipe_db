// 仪表盘页 — 关键经营指标卡片（总库存/30天出入库/待审批数）+ 出入库及待审批明细表
import { useMemo } from 'react';
import { Card, Row, Col, Statistic, Spin, Table, Typography } from 'antd';
import {
  DatabaseOutlined,
  InboxOutlined,
  ClockCircleOutlined,
  CheckCircleOutlined,
} from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { useDashboard } from '../hooks/useReports';
import type {
  DashboardRecentInbound,
  DashboardRecentOutbound,
  DashboardPendingApproval,
  DashboardRecentQualityFailure,
} from '../types';

const { Title } = Typography;

export default function DashboardPage() {
  const { t } = useTranslation();
  const { data, isLoading } = useDashboard();

  if (isLoading) return <Spin size="large" style={{ display: 'block', margin: '60px auto' }} />;

  const inboundColumns = useMemo(() => [
    { title: '入库单号', dataIndex: 'record_no', key: 'record_no' },
    { title: '类型', dataIndex: 'type', key: 'type' },
    { title: '状态', dataIndex: 'approval_status', key: 'approval_status' },
    { title: '时间', dataIndex: 'created_at', key: 'created_at' },
  ], []);

  const outboundColumns = useMemo(() => [
    { title: '出库单号', dataIndex: 'record_no', key: 'record_no' },
    { title: '类型', dataIndex: 'type', key: 'type' },
    { title: '状态', dataIndex: 'approval_status', key: 'approval_status' },
    { title: '时间', dataIndex: 'created_at', key: 'created_at' },
  ], []);

  const pendingColumns = useMemo(() => [
    { title: '单据号', dataIndex: 'reference_no', key: 'reference_no' },
    { title: '类型', dataIndex: 'reference_type', key: 'reference_type' },
  ], []);

  const failureColumns = useMemo(() => [
    { title: '证书编号', dataIndex: 'cert_no', key: 'cert_no' },
    { title: '管材类型', dataIndex: 'pipe_type', key: 'pipe_type' },
    { title: '检验日期', dataIndex: 'inspect_date', key: 'inspect_date' },
    { title: '备注', dataIndex: 'notes', key: 'notes' },
  ], []);

  return (
    <div>
      <Title level={3}>{t('reports.dashboard')}</Title>
      <Row gutter={[16, 16]}>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title={t('reports.total_pipes')}
              value={data?.total_stock ?? 0}
              prefix={<DatabaseOutlined />}
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="近30天入库"
              value={data?.inbound_30d ?? 0}
              prefix={<InboxOutlined />}
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title={t('reports.pending_orders')}
              value={data?.pending_approvals ?? 0}
              prefix={<ClockCircleOutlined />}
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="近30天出库"
              value={data?.outbound_30d ?? 0}
              prefix={<CheckCircleOutlined />}
            />
          </Card>
        </Col>
      </Row>
      <Row gutter={[16, 16]} style={{ marginTop: 24 }}>
        <Col xs={24} lg={12}>
          <Card title="最近入库记录">
            <Table<DashboardRecentInbound>
              columns={inboundColumns}
              dataSource={data?.recent_inbound}
              rowKey="record_no"
              pagination={false}
              size="small"
            />
          </Card>
        </Col>
        <Col xs={24} lg={12}>
          <Card title="最近出库记录">
            <Table<DashboardRecentOutbound>
              columns={outboundColumns}
              dataSource={data?.recent_outbound}
              rowKey="record_no"
              pagination={false}
              size="small"
            />
          </Card>
        </Col>
      </Row>
      <Row gutter={[16, 16]} style={{ marginTop: 16 }}>
        <Col xs={24} lg={12}>
          <Card title="待审批列表">
            <Table<DashboardPendingApproval>
              columns={pendingColumns}
              dataSource={data?.pending_approval_list}
              rowKey="id"
              pagination={false}
              size="small"
            />
          </Card>
        </Col>
        <Col xs={24} lg={12}>
          <Card title="最近质检不合格">
            <Table<DashboardRecentQualityFailure>
              columns={failureColumns}
              dataSource={data?.recent_quality_failures}
              rowKey="cert_no"
              pagination={false}
              size="small"
            />
          </Card>
        </Col>
      </Row>
    </div>
  );
}
