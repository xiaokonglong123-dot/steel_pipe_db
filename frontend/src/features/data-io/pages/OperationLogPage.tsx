/**
 * OperationLogPage — View import/export operation logs for auditing.
 *
 * Displays a paginated table of all data IO operations with filtering
 * by action type and entity type.
 */
import { useState, useEffect } from 'react';
import {
  Card,
  Table,
  Select,
  Space,
  Tag,
  Typography,
} from 'antd';
import type { ColumnsType } from 'antd/es/table';
import { dataIoApi, ENTITY_TYPES, type OperationLog } from '../api/dataIoApi';

const { Title } = Typography;

const ACTION_OPTIONS = [
  { value: 'import', label: '导入' },
  { value: 'export', label: '导出' },
  { value: 'download_template', label: '下载模板' },
];

const columns: ColumnsType<OperationLog> = [
  {
    title: '时间',
    dataIndex: 'created_at',
    key: 'created_at',
    width: 180,
  },
  {
    title: '操作',
    dataIndex: 'action',
    key: 'action',
    width: 120,
    render: (action: string) => {
      const color =
        action === 'import'
          ? 'blue'
          : action === 'export'
            ? 'green'
            : 'default';
      const label =
        action === 'import'
          ? '导入'
          : action === 'export'
            ? '导出'
            : '下载模板';
      return <Tag color={color}>{label}</Tag>;
    },
  },
  {
    title: '数据类型',
    dataIndex: 'entity_type',
    key: 'entity_type',
    width: 140,
    render: (et: string) => {
      const found = ENTITY_TYPES.find((t) => t.value === et);
      return found ? found.label : et;
    },
  },
  {
    title: '操作人',
    dataIndex: 'username',
    key: 'username',
    width: 120,
    render: (u: string | null) => u || '-',
  },
  {
    title: '详情',
    dataIndex: 'details',
    key: 'details',
    ellipsis: true,
    render: (d: string | null) => d || '-',
  },
  {
    title: 'IP 地址',
    dataIndex: 'ip_address',
    key: 'ip_address',
    width: 140,
    render: (ip: string | null) => ip || '-',
  },
];

export default function OperationLogPage() {
  const [logs, setLogs] = useState<OperationLog[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [loading, setLoading] = useState(false);
  const [actionFilter, setActionFilter] = useState<string | undefined>();
  const [entityTypeFilter, setEntityTypeFilter] = useState<string | undefined>();

  const fetchLogs = async () => {
    setLoading(true);
    try {
      const data = await dataIoApi.listOperationLogs({
        page,
        page_size: pageSize,
        action: actionFilter,
        entity_type: entityTypeFilter,
      });
      if (data) {
        setLogs(data.items);
        setTotal(data.total);
      }
    } catch {
      // silently fail — table will be empty
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchLogs();
  }, [page, pageSize, actionFilter, entityTypeFilter]);

  return (
    <div style={{ padding: 24 }}>
      <Title level={3}>操作日志</Title>

      <Card>
        <Space style={{ marginBottom: 16 }}>
          <Select
            placeholder="操作类型"
            allowClear
            style={{ width: 140 }}
            value={actionFilter}
            onChange={(v) => {
              setActionFilter(v);
              setPage(1);
            }}
            options={ACTION_OPTIONS}
          />
          <Select
            placeholder="数据类型"
            allowClear
            style={{ width: 160 }}
            value={entityTypeFilter}
            onChange={(v) => {
              setEntityTypeFilter(v);
              setPage(1);
            }}
            options={ENTITY_TYPES.map((t) => ({ value: t.value, label: t.label }))}
          />
        </Space>

        <Table<OperationLog>
          columns={columns}
          dataSource={logs}
          rowKey="id"
          loading={loading}
          pagination={{
            current: page,
            pageSize,
            total,
            showSizeChanger: true,
            showTotal: (t) => `共 ${t} 条`,
            onChange: (p, ps) => {
              setPage(p);
              setPageSize(ps);
            },
          }}
          scroll={{ x: 800 }}
        />
      </Card>
    </div>
  );
}
