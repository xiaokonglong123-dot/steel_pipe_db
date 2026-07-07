/**
 * InventoryReportPage — Aggregated inventory summary by status, grade, type, and location occupancy.
 *
 * Backend returns: { by_status, by_grade, by_type, location_occupancy }
 * See backend/src/services/report_service.rs → inventory_summary()
 */
import { Card, Table, Typography, Statistic, Row, Col, Tabs } from 'antd';
import { BarChartOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import { useInventorySummary } from '../hooks/useReports';
import type { InventoryByStatus, InventoryByGrade, InventoryByType, LocationOccupancy } from '../types';

const { Title } = Typography;

const statusColumns: ColumnsType<InventoryByStatus> = [
  { title: '状态', dataIndex: 'status', key: 'status', width: 160 },
  { title: '数量', dataIndex: 'count', key: 'count', width: 100 },
];

const gradeColumns: ColumnsType<InventoryByGrade> = [
  { title: '钢级', dataIndex: 'grade', key: 'grade', width: 120 },
  { title: '钢管类型', dataIndex: 'pipe_type', key: 'pipe_type', width: 100 },
  { title: '数量', dataIndex: 'count', key: 'count', width: 100 },
];

const typeColumns: ColumnsType<InventoryByType> = [
  { title: '管材类型', dataIndex: 'pipe_type', key: 'pipe_type', width: 120 },
  { title: '数量', dataIndex: 'count', key: 'count', width: 100 },
];

const locationColumns: ColumnsType<LocationOccupancy> = [
  { title: '库位', dataIndex: 'location', key: 'location', width: 160 },
  { title: '最大容量', dataIndex: 'max_capacity', key: 'max_capacity', width: 100 },
  { title: '当前使用', dataIndex: 'current_usage', key: 'current_usage', width: 100 },
  { title: '可用', dataIndex: 'available', key: 'available', width: 100 },
  { title: '占用率', dataIndex: 'occupancy_pct', key: 'occupancy_pct', width: 100 },
];

export default function InventoryReportPage() {
  const { data, isLoading } = useInventorySummary();

  const totalPipes =
    data?.by_status?.reduce((sum, r) => sum + r.count, 0) ?? 0;
  const uniqueGrades = new Set(data?.by_grade?.map((r) => r.grade)).size;
  const locationCount = data?.location_occupancy?.length ?? 0;

  const tabItems = [
    {
      key: 'by_status',
      label: '按状态',
      children: (
        <Table<InventoryByStatus>
          columns={statusColumns}
          dataSource={data?.by_status}
          rowKey="status"
          loading={isLoading}
          pagination={false}
          scroll={{ x: 400 }}
        />
      ),
    },
    {
      key: 'by_grade',
      label: '按钢级',
      children: (
        <Table<InventoryByGrade>
          columns={gradeColumns}
          dataSource={data?.by_grade}
          rowKey={(r) => `${r.grade}-${r.pipe_type}`}
          loading={isLoading}
          pagination={{ pageSize: 20, showTotal: (t) => `共 ${t} 条` }}
          scroll={{ x: 400 }}
        />
      ),
    },
    {
      key: 'by_type',
      label: '按类型',
      children: (
        <Table<InventoryByType>
          columns={typeColumns}
          dataSource={data?.by_type}
          rowKey="pipe_type"
          loading={isLoading}
          pagination={false}
          scroll={{ x: 400 }}
        />
      ),
    },
    {
      key: 'location',
      label: '库位占用',
      children: (
        <Table<LocationOccupancy>
          columns={locationColumns}
          dataSource={data?.location_occupancy}
          rowKey="location"
          loading={isLoading}
          pagination={{ pageSize: 20, showTotal: (t) => `共 ${t} 条` }}
          scroll={{ x: 600 }}
        />
      ),
    },
  ];

  return (
    <div style={{ padding: 24 }}>
      <Title level={3}>
        <BarChartOutlined style={{ marginRight: 8 }} />
        库存汇总报表
      </Title>

      <Row gutter={16} style={{ marginBottom: 16 }}>
        <Col span={8}>
          <Card>
            <Statistic title="总库存数量" value={totalPipes} />
          </Card>
        </Col>
        <Col span={8}>
          <Card>
            <Statistic title="钢级种类" value={uniqueGrades} />
          </Card>
        </Col>
        <Col span={8}>
          <Card>
            <Statistic title="库位数" value={locationCount} />
          </Card>
        </Col>
      </Row>

      <Card>
        <Tabs items={tabItems} />
      </Card>
    </div>
  );
}
