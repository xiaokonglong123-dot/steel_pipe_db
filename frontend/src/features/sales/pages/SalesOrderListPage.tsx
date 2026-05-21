import { useState } from 'react';
import { Table, Button, Space, Tag, Input, Popconfirm, Select } from 'antd';
import { PlusOutlined, SearchOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useSalesOrders, useDeleteSalesOrder } from '../hooks/useSales';
import type { SalesOrder } from '../types';

const STATUS_COLORS: Record<string, string> = {
  draft: 'default',
  pending: 'blue',
  approved: 'cyan',
  delivered: 'green',
  invoiced: 'purple',
  cancelled: 'red',
};

const ORDER_STATUSES = ['draft', 'pending', 'approved', 'delivered', 'invoiced', 'cancelled'];

export default function SalesOrderListPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [searchText, setSearchText] = useState('');
  const [statusFilter, setStatusFilter] = useState<string | undefined>();

  const { data, isLoading } = useSalesOrders({
    page,
    page_size: pageSize,
    q: searchText || undefined,
    status: statusFilter,
  });

  const deleteMutation = useDeleteSalesOrder();

  const columns = [
    {
      title: t('Order Number'),
      dataIndex: 'order_number',
      key: 'order_number',
      sorter: true,
    },
    {
      title: t('Customer'),
      dataIndex: 'customer_name',
      key: 'customer_name',
    },
    {
      title: t('Order Date'),
      dataIndex: 'order_date',
      key: 'order_date',
    },
    {
      title: t('Expected Delivery'),
      dataIndex: 'expected_delivery',
      key: 'expected_delivery',
      render: (val: string | null) => val ?? '-',
    },
    {
      title: t('Total Amount'),
      dataIndex: 'total_amount',
      key: 'total_amount',
      render: (val: number) => val.toLocaleString(),
    },
    {
      title: t('Status'),
      dataIndex: 'status',
      key: 'status',
      render: (status: string) => (
        <Tag color={STATUS_COLORS[status] ?? 'default'}>{status}</Tag>
      ),
    },
    {
      title: t('common.actions'),
      key: 'actions',
      render: (_: unknown, record: SalesOrder) => (
        <Space>
          <Button
            type="link"
            onClick={() => navigate(`/sales/${record.id}`)}
          >
            {t('common.edit')}
          </Button>
          <Popconfirm
            title="确认删除?"
            onConfirm={() => deleteMutation.mutate(record.id)}
          >
            <Button type="link" danger>
              {t('common.delete')}
            </Button>
          </Popconfirm>
        </Space>
      ),
    },
  ];

  return (
    <div>
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          marginBottom: 16,
        }}
      >
        <Space>
          <Input
            placeholder={t('common.search')}
            prefix={<SearchOutlined />}
            value={searchText}
            onChange={(e) => setSearchText(e.target.value)}
            style={{ width: 250 }}
          />
          <Select
            placeholder={t('Status')}
            allowClear
            value={statusFilter}
            onChange={(val) => setStatusFilter(val)}
            style={{ width: 150 }}
          >
            {ORDER_STATUSES.map((s) => (
              <Select.Option key={s} value={s}>
                {s}
              </Select.Option>
            ))}
          </Select>
        </Space>
        <Button
          type="primary"
          icon={<PlusOutlined />}
          onClick={() => navigate('/sales/new')}
        >
          {t('common.create')}
        </Button>
      </div>
      <Table
        columns={columns}
        dataSource={data?.items}
        rowKey="id"
        loading={isLoading}
        pagination={{
          current: page,
          pageSize,
          total: data?.total,
          onChange: (p, ps) => {
            setPage(p);
            setPageSize(ps);
          },
          showSizeChanger: true,
          showTotal: (total) => `Total ${total} items`,
        }}
      />
    </div>
  );
}
