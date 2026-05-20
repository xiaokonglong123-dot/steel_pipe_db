import React, { useState } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  Card,
  Table,
  Button,
  Space,
  Input,
  Select,
  message,
  Popconfirm,
  Row,
  Col,
  DatePicker,
} from 'antd';
import {
  PlusOutlined,
  SearchOutlined,
  CheckCircleOutlined,
  CloseOutlined,
  EditOutlined,
} from '@ant-design/icons';
import type { TablePaginationConfig, ColumnsType } from 'antd/es/table';
import { salesApi, SalesOrder } from '../api/salesApi';
import OrderStatusTag from '../components/OrderStatusTag';
import type { OrderStatus } from '../../../shared/types';

const { RangePicker } = DatePicker;

const defaultFilter = { page: 1, page_size: 20 };

export default function SalesOrderListPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [filter, setFilter] = useState<Record<string, unknown>>(defaultFilter);
  const [searchText, setSearchText] = useState('');

  const { data, isLoading } = useQuery({
    queryKey: ['sales-orders', filter],
    queryFn: () => salesApi.list(filter as Record<string, undefined>),
  });

  const approveMutation = useMutation({
    mutationFn: (id: string) => salesApi.approve(id) as Promise<unknown>,
    onSuccess: () => {
      message.success('审核通过');
      queryClient.invalidateQueries({ queryKey: ['sales-orders'] });
    },
    onError: () => {
      message.error('审核失败');
    },
  });

  const cancelMutation = useMutation({
    mutationFn: (id: string) => salesApi.cancel(id) as Promise<unknown>,
    onSuccess: () => {
      message.success('订单已取消');
      queryClient.invalidateQueries({ queryKey: ['sales-orders'] });
    },
    onError: () => {
      message.error('取消失败');
    },
  });

  const handleSearch = () => {
    setFilter((prev) => ({ ...prev, search: searchText || undefined, page: 1 }));
  };

  const handleFilterChange = (key: string, value?: unknown) => {
    setFilter((prev) => ({ ...prev, [key]: value ?? undefined, page: 1 }));
  };

  const handleDateChange = (_: unknown, dateStrings: [string, string]) => {
    setFilter((prev) => ({
      ...prev,
      date_from: dateStrings[0] || undefined,
      date_to: dateStrings[1] || undefined,
      page: 1,
    }));
  };

  const handleTableChange = (pagination: TablePaginationConfig) => {
    setFilter((prev) => ({
      ...prev,
      page: pagination.current || 1,
      page_size: pagination.pageSize || 20,
    }));
  };

  const columns: ColumnsType<SalesOrder> = [
    { title: '订单编号', dataIndex: 'order_no', key: 'order_no', width: 180, fixed: 'left' },
    { title: '客户', dataIndex: 'customer_name', key: 'customer_name', width: 160, ellipsis: true },
    {
      title: '操作人',
      dataIndex: 'operator_name',
      key: 'operator_name',
      width: 120,
      render: (v: string) => v || '-',
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      width: 100,
      render: (s: OrderStatus) => <OrderStatusTag status={s} />,
    },
    {
      title: '总金额',
      dataIndex: 'total_amount',
      key: 'total_amount',
      width: 120,
      align: 'right',
      render: (v: number) => `¥${v.toLocaleString()}`,
    },
    { title: '创建时间', dataIndex: 'created_at', key: 'created_at', width: 170 },
    {
      title: '出库单',
      key: 'outbound_refs',
      width: 160,
      render: (_, r) =>
        r.outbound_refs && r.outbound_refs.length > 0
          ? r.outbound_refs.map((ref) => (
              <div key={ref.outbound_id}>
                <Link to={`/inventory/outbound/${ref.outbound_id}`}>{ref.outbound_no}</Link>
              </div>
            ))
          : '-',
    },
    {
      title: '操作',
      key: 'action',
      width: 200,
      fixed: 'right',
      render: (_, r) => (
        <Space size="small">
          <Link to={`/sales/${r.id}`}>
            <Button type="link" size="small">
              查看
            </Button>
          </Link>
          {(r.status === 'draft' || r.status === 'pending') && (
            <Link to={`/sales/${r.id}/edit`}>
              <Button type="link" size="small" icon={<EditOutlined />} />
            </Link>
          )}
          {r.status === 'pending' && (
            <Button
              type="link"
              size="small"
              icon={<CheckCircleOutlined />}
              onClick={() => approveMutation.mutate(r.id)}
            >
              审核
            </Button>
          )}
          {(r.status === 'draft' || r.status === 'pending') && (
            <Popconfirm
              title="确定取消此订单？"
              onConfirm={() => cancelMutation.mutate(r.id)}
            >
              <Button type="link" size="small" icon={<CloseOutlined />} danger>
                取消
              </Button>
            </Popconfirm>
          )}
        </Space>
      ),
    },
  ];

  return (
    <Card title="销售订单" styles={{ body: { padding: 16 } }}>
      <Row gutter={[12, 12]} style={{ marginBottom: 16 }}>
        <Col xs={24} sm={12} md={6}>
          <Input
            placeholder="搜索订单编号"
            prefix={<SearchOutlined />}
            value={searchText}
            onChange={(e) => setSearchText(e.target.value)}
            onPressEnter={handleSearch}
            allowClear
          />
        </Col>
        <Col xs={12} sm={6} md={4}>
          <Select
            placeholder="状态"
            style={{ width: '100%' }}
            value={filter.status as string | undefined}
            onChange={(v) => handleFilterChange('status', v)}
            allowClear
            options={[
              { label: '草稿', value: 'draft' },
              { label: '待审核', value: 'pending' },
              { label: '已审核', value: 'approved' },
              { label: '已完成', value: 'completed' },
              { label: '已取消', value: 'cancelled' },
            ]}
          />
        </Col>
        <Col xs={24} sm={12} md={6}>
          <RangePicker
            style={{ width: '100%' }}
            onChange={handleDateChange}
          />
        </Col>
        <Col xs={24} sm={12} md={4}>
          <Space>
            <Button type="primary" icon={<SearchOutlined />} onClick={handleSearch}>
              搜索
            </Button>
            <Button
              icon={<PlusOutlined />}
              type="primary"
              ghost
              onClick={() => navigate('/sales/new')}
            >
              新增
            </Button>
          </Space>
        </Col>
      </Row>

      <Table
        columns={columns}
        dataSource={data?.data?.data}
        rowKey="id"
        loading={isLoading}
        locale={{ emptyText: '暂无数据' }}
        scroll={{ x: 1200 }}
        pagination={{
          current: data?.data?.meta?.page || 1,
          pageSize: data?.data?.meta?.page_size || 20,
          total: data?.data?.meta?.total || 0,
          showSizeChanger: true,
          showQuickJumper: true,
          showTotal: (total) => `共 ${total} 条`,
        }}
        onChange={handleTableChange}
      />
    </Card>
  );
}
