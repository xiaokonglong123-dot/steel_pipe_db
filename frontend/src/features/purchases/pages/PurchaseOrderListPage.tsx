// 采购订单列表页 — 使用 DataTable + PageLayout 共享组件
import { useState } from 'react';
import { Button, Tag, Input, Popconfirm } from 'antd';
import { PlusOutlined, SearchOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { PageLayout } from '@/shared/components/PageLayout';
import { DataTable } from '@/shared/components/DataTable';
import { usePagination } from '@/shared/hooks/usePagination';
import { usePurchases, useDeletePurchaseOrder } from '../hooks/usePurchases';
import type { PurchaseOrder } from '../types';

const STATUS_COLORS: Record<string, string> = {
  draft: 'default',
  pending: 'orange',
  approved: 'blue',
  received: 'green',
  cancelled: 'red',
};

export default function PurchaseOrderListPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { page, pageSize, onPaginationChange, reset } = usePagination();
  const [searchText, setSearchText] = useState('');

  const { data, isLoading } = usePurchases({
    page,
    page_size: pageSize,
    q: searchText || undefined,
  });

  const deleteMutation = useDeletePurchaseOrder();

  const columns = [
    {
      title: t('purchases.order_number'),
      dataIndex: 'order_no',
      key: 'order_no',
      sorter: true,
    },
    {
      title: t('purchases.order_date'),
      dataIndex: 'order_date',
      key: 'order_date',
    },
    {
      title: t('purchases.status'),
      dataIndex: 'status',
      key: 'status',
      render: (status: string) => (
        <Tag color={STATUS_COLORS[status] ?? 'default'}>{t('purchases.status.' + status)}</Tag>
      ),
    },
    {
      title: t('purchases.total_amount'),
      dataIndex: 'total_amount',
      key: 'total_amount',
      render: (val: number | null) => val != null ? val.toFixed(2) : '-',
    },
    {
      title: t('common.actions'),
      key: 'actions',
      render: (_: unknown, record: PurchaseOrder) => (
        <>
          <Button
            type="link"
            size="small"
            onClick={() => navigate(`/purchases/${record.id}`)}
          >
            {t('common.view')}
          </Button>
          <Button
            type="link"
            size="small"
            onClick={() => navigate(`/purchases/${record.id}/edit`)}
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
      title={t('purchases.title')}
      extra={
        <Button type="primary" icon={<PlusOutlined />} onClick={() => navigate('/purchases/new')}>
          {t('common.create')}
        </Button>
      }
    >
      <Input
        placeholder={t('common.search')}
        prefix={<SearchOutlined />}
        value={searchText}
        onChange={(e) => {
          setSearchText(e.target.value);
          reset();
        }}
        style={{ width: 250, marginBottom: 16 }}
        allowClear
      />
      <DataTable<PurchaseOrder>
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
