/**
 * InventoryReportPage — Aggregated inventory summary by pipe type, grade, and location.
 */
import { useState, useEffect } from 'react';
import { Card, Table, Typography, Statistic, Row, Col } from 'antd';
import { BarChartOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import { reportApi } from '../api/reportApi';

const { Title } = Typography;

interface InventorySummaryRow {
  pipe_type: string;
  grade: string;
  total_quantity: number;
  location: string;
}

const columns: ColumnsType<InventorySummaryRow> = [
  { title: '管材类型', dataIndex: 'pipe_type', key: 'pipe_type', width: 120 },
  { title: '钢级', dataIndex: 'grade', key: 'grade', width: 100 },
  { title: '数量', dataIndex: 'total_quantity', key: 'total_quantity', width: 100 },
  { title: '库位', dataIndex: 'location', key: 'location', width: 150 },
];

export default function InventoryReportPage() {
  const [data, setData] = useState<InventorySummaryRow[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    setLoading(true);
    reportApi
      .getInventorySummary()
      .then((res) => setData(res || []))
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  const totalQuantity = data.reduce((sum, r) => sum + r.total_quantity, 0);
  const uniqueGrades = new Set(data.map((r) => r.grade)).size;

  return (
    <div style={{ padding: 24 }}>
      <Title level={3}>
        <BarChartOutlined style={{ marginRight: 8 }} />
        库存汇总报表
      </Title>

      <Row gutter={16} style={{ marginBottom: 16 }}>
        <Col span={8}>
          <Card>
            <Statistic title="总库存数量" value={totalQuantity} />
          </Card>
        </Col>
        <Col span={8}>
          <Card>
            <Statistic title="钢级种类" value={uniqueGrades} />
          </Card>
        </Col>
        <Col span={8}>
          <Card>
            <Statistic title="记录行数" value={data.length} />
          </Card>
        </Col>
      </Row>

      <Card>
        <Table<InventorySummaryRow>
          columns={columns}
          dataSource={data}
          rowKey={(r) => `${r.pipe_type}-${r.grade}-${r.location}`}
          loading={loading}
          pagination={{ pageSize: 20, showTotal: (t) => `共 ${t} 条` }}
          scroll={{ x: 500 }}
        />
      </Card>
    </div>
  );
}
