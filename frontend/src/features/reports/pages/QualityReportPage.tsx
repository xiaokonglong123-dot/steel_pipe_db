/**
 * QualityReportPage — Quality inspection pass/fail statistics by grade.
 */
import { useState, useEffect } from 'react';
import {
  Card,
  Table,
  Typography,
  Tag,
  Statistic,
  Row,
  Col,
  Progress,
} from 'antd';
import { SafetyCertificateOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import { reportApi } from '../api/reportApi';

const { Title } = Typography;

interface QualityReportRow {
  period: string;
  total_certificates: number;
  passed: number;
  failed: number;
  by_grade: Record<
    string,
    { total: number; passed: number; failed: number }
  >;
}

const columns: ColumnsType<QualityReportRow> = [
  { title: '期间', dataIndex: 'period', key: 'period', width: 140 },
  {
    title: '证书总数',
    dataIndex: 'total_certificates',
    key: 'total_certificates',
    width: 100,
  },
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
    width: 120,
    render: (_: unknown, r: QualityReportRow) => {
      const rate =
        r.total_certificates > 0
          ? Math.round((r.passed / r.total_certificates) * 100)
          : 0;
      return <Progress percent={rate} size="small" />;
    },
  },
  {
    title: '按钢级',
    dataIndex: 'by_grade',
    key: 'by_grade',
    render: (by_grade: Record<string, { total: number; passed: number; failed: number }>) => (
      <span>
        {Object.entries(by_grade).map(([grade, info]) => (
          <Tag key={grade}>
            {grade}: {info.passed}/{info.total}
          </Tag>
        ))}
      </span>
    ),
  },
];

export default function QualityReportPage() {
  const [data, setData] = useState<QualityReportRow[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    setLoading(true);
    reportApi
      .getQualityReport()
      .then((res) => {
        setData(Array.isArray(res) ? res : res ? [res] : []);
      })
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  const totalCerts = data.reduce((sum, r) => sum + r.total_certificates, 0);
  const totalPassed = data.reduce((sum, r) => sum + r.passed, 0);
  const totalFailed = data.reduce((sum, r) => sum + r.failed, 0);
  const passRate =
    totalCerts > 0 ? Math.round((totalPassed / totalCerts) * 100) : 0;

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
        <Table<QualityReportRow>
          columns={columns}
          dataSource={data}
          rowKey="period"
          loading={loading}
          pagination={{ pageSize: 20, showTotal: (t) => `共 ${t} 条` }}
          scroll={{ x: 700 }}
        />
      </Card>
    </div>
  );
}
