/**
 * QualityReportPage — Quality inspection pass/fail statistics by grade and by month.
 *
 * Backend returns: { by_grade, by_month }
 * See backend/src/services/report_service.rs → quality_report()
 */
import {
  Card,
  Table,
  Typography,
  Tag,
  Statistic,
  Row,
  Col,
  Progress,
  Space,
  Tabs,
} from 'antd';
import { SafetyCertificateOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import { useQualityReport } from '../hooks/useReports';
import type { QualityByGrade, QualityByMonth } from '../types';

const { Title } = Typography;

const byGradeColumns: ColumnsType<QualityByGrade> = [
  { title: '钢级', dataIndex: 'grade', key: 'grade', width: 120 },
  { title: '管材类型', dataIndex: 'pipe_type', key: 'pipe_type', width: 100 },
  {
    title: '通过',
    dataIndex: 'pass_count',
    key: 'pass_count',
    width: 80,
    render: (v: number) => <Tag color="green">{v}</Tag>,
  },
  {
    title: '不合格',
    dataIndex: 'fail_count',
    key: 'fail_count',
    width: 80,
    render: (v: number) => <Tag color="red">{v}</Tag>,
  },
  {
    title: '总数',
    dataIndex: 'total',
    key: 'total',
    width: 80,
  },
  {
    title: '合格率',
    dataIndex: 'pass_rate',
    key: 'pass_rate',
    width: 100,
  },
];

const byMonthColumns: ColumnsType<QualityByMonth> = [
  { title: '月份', dataIndex: 'month', key: 'month', width: 120 },
  { title: '总数', dataIndex: 'total', key: 'total', width: 80 },
  {
    title: '通过',
    dataIndex: 'passed',
    key: 'passed',
    width: 80,
    render: (v: number) => <Tag color="green">{v}</Tag>,
  },
  {
    title: '不合格',
    dataIndex: 'failed',
    key: 'failed',
    width: 80,
    render: (v: number) => <Tag color="red">{v}</Tag>,
  },
  {
    title: '合格率',
    key: 'pass_rate',
    width: 160,
    render: (_: unknown, r: QualityByMonth) => {
      const rateNum = parseFloat(r.pass_rate);
      return (
        <Space>
          <Progress percent={isNaN(rateNum) ? 0 : rateNum} size="small" />
          <span>{r.pass_rate}</span>
        </Space>
      );
    },
  },
];

export default function QualityReportPage() {
  const { data, isLoading } = useQualityReport();

  const totalCerts = data?.by_month?.reduce((sum, r) => sum + r.total, 0) ?? 0;
  const totalPassed = data?.by_month?.reduce((sum, r) => sum + r.passed, 0) ?? 0;
  const totalFailed = data?.by_month?.reduce((sum, r) => sum + r.failed, 0) ?? 0;
  const passRate =
    totalCerts > 0 ? Math.round((totalPassed / totalCerts) * 100) : 0;

  const tabItems = [
    {
      key: 'by_grade',
      label: '按钢级',
      children: (
        <Table<QualityByGrade>
          columns={byGradeColumns}
          dataSource={data?.by_grade}
          rowKey={(r) => `${r.grade}-${r.pipe_type}`}
          loading={isLoading}
          pagination={{ pageSize: 20, showTotal: (t) => `共 ${t} 条` }}
          scroll={{ x: 600 }}
        />
      ),
    },
    {
      key: 'by_month',
      label: '按月份',
      children: (
        <Table<QualityByMonth>
          columns={byMonthColumns}
          dataSource={data?.by_month}
          rowKey="month"
          loading={isLoading}
          pagination={false}
          scroll={{ x: 600 }}
        />
      ),
    },
  ];

  return (
    <div style={{ padding: 24 }}>
      <Title level={3}>
        <SafetyCertificateOutlined style={{ marginRight: 8 }} />
        质量报表
      </Title>

      <Row gutter={16} style={{ marginBottom: 16 }}>
        <Col span={6}>
          <Card>
            <Statistic title="证书总数" value={totalCerts} />
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Statistic title="通过" value={totalPassed} valueStyle={{ color: '#52c41a' }} />
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Statistic title="不合格" value={totalFailed} valueStyle={{ color: '#ff4d4f' }} />
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Statistic title="合格率" value={passRate} suffix="%" />
          </Card>
        </Col>
      </Row>

      <Card>
        <Tabs items={tabItems} />
      </Card>
    </div>
  );
}

