// 销售订单列表页 — 使用 DataTable + PageLayout 共享组件
import { useState } from 'react';
import { Button, Tag, Input, Popconfirm, Select } from 'antd';
import { PlusOutlined, SearchOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { PageLayout } from '@/shared/components/PageLayout';
import { DataTable } from '@/shared/components/DataTable';
import { usePagination } from '@/shared/hooks/usePagination';
import { ORDER_STATUSES } from '@/shared/constants';
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

export default function SalesOrderListPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { page, pageSize, onPaginationChange, reset } = usePagination();
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
      title: t('sales.order_number'),
      dataIndex: 'order_no',
      key: 'order_no',
      sorter: true,
    },
    {
      title: t('sales.order_date'),
      dataIndex: 'order_date',
      key: 'order_date',
    },
    {
      title: t('sales.total_amount'),
      dataIndex: 'total_amount',
      key: 'total_amount',
      render: (val: number | null) => val != null ? val.toLocaleString() : '-',
    },
    {
      title: t('sales.status'),
      dataIndex: 'status',
      key: 'status',
      render: (status: string) => (
        <Tag color={STATUS_COLORS[status] ?? 'default'}>{t('sales.status.' + status)}</Tag>
      ),
    },
    {
      title: t('common.actions'),
      key: 'actions',
      render: (_: unknown, record: SalesOrder) => (
        <>
          <Button
            type="link"
            size="small"
            onClick={() => navigate(`/sales/${record.id}`)}
          >
            {t('common.edit')}
          </Button>
          <Popconfirm
            title={t('common.confirm_delete')}
            onConfirm={() => deleteMutation.mutate(record.id)}
          >
            <Button type="link" danger size="small" loading={deleteMutation.isPending}>
              {t('common.delete')}
            </Button>
          </Popconfirm>
        </>
      ),
    },
  ];

  return (
    <PageLayout
      title={t('sales.title')}
      extra={
        <Button type="primary" icon={<PlusOutlined />} onClick={() => navigate('/sales/new')}>
          {t('common.create')}
        </Button>
      }
    >
      <div style={{ display: 'flex', gap: 12, marginBottom: 16 }}>
        <Input
          placeholder={t('common.search')}
          prefix={<SearchOutlined />}
          value={searchText}
          onChange={(e) => {
            setSearchText(e.target.value);
            reset();
          }}
          style={{ width: 250 }}
          allowClear
        />
        <Select
          placeholder={t('sales.status')}
          allowClear
          value={statusFilter}
          onChange={setStatusFilter}
          style={{ width: 150 }}
        >
          {ORDER_STATUSES.map((s) => (
            <Select.Option key={s} value={s}>
              {t('sales.status.' + s)}
            </Select.Option>
          ))}
        </Select>
      </div>
      <DataTable<SalesOrder>
        columns={columns}
        items={data?.items}
        total={data?.total}
        page={page}
        pageSize={pageSize}
        loading={isLoading}
        onPaginationChange={onPaginationChange}
      />
    </PageLayout>
  );
}
