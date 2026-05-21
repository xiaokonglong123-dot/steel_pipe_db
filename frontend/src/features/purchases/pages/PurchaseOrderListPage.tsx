// 采购订单列表页 — 分页查询、状态彩色标签、搜索、新建/编辑/删除/审核流转
import { useState } from 'react';
import { Table, Button, Space, Tag, Input, Popconfirm } from 'antd';
import { PlusOutlined, SearchOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
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
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
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
      dataIndex: 'order_number',
      key: 'order_number',
      sorter: true,
    },
    {
      title: t('purchases.supplier'),
      dataIndex: 'supplier_name',
      key: 'supplier_name',
    },
    {
      title: t('purchases.order_date'),
      dataIndex: 'order_date',
      key: 'order_date',
    },
    {
      title: t('purchases.expected_delivery'),
      dataIndex: 'expected_date',
      key: 'expected_date',
      render: (val: string | undefined) => val ?? '-',
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
      render: (val: number) => val.toFixed(2),
    },
    {
      title: t('common.actions'),
      key: 'actions',
      render: (_: unknown, record: PurchaseOrder) => (
        <Space>
          <Button
            type="link"
            onClick={() => navigate(`/purchases/${record.id}`)}
          >
            {t('common.view')}
          </Button>
          <Button
            type="link"
            onClick={() => navigate(`/purchases/${record.id}/edit`)}
          >
            {t('common.edit')}
          </Button>
          <Popconfirm
            title={t('common.confirm_delete')}
            onConfirm={() => deleteMutation.mutate(record.id)}
          >
            <Button type="link" danger loading={deleteMutation.isPending}>
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
        </Space>
        <Button
          type="primary"
          icon={<PlusOutlined />}
          onClick={() => navigate('/purchases/new')}
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
          showTotal: (total) => t('common.total_items', { total }),
        }}
      />
    </div>
  );
}
