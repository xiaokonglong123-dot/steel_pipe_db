// Supplier list page — uses DataTable + PageLayout shared components
import { useState } from 'react';
import { Button, Tag, Input, Popconfirm } from 'antd';
import { PlusOutlined, SearchOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { PageLayout } from '@/shared/components/PageLayout';
import { DataTable } from '@/shared/components/DataTable';
import { usePagination } from '@/shared/hooks/usePagination';
import { useSuppliers, useDeleteSupplier } from '../hooks/useSuppliers';
import type { Supplier } from '../types';

export default function SupplierListPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { page, pageSize, onPaginationChange, reset } = usePagination();
  const [searchText, setSearchText] = useState('');

  const { data, isLoading } = useSuppliers({
    page,
    page_size: pageSize,
    q: searchText || undefined,
  });

  const deleteMutation = useDeleteSupplier();

  const columns = [
    {
      title: t('suppliers.code'),
      dataIndex: 'supplier_code',
      key: 'supplier_code',
      sorter: true,
    },
    {
      title: t('suppliers.name'),
      dataIndex: 'name',
      key: 'name',
    },
    {
      title: t('suppliers.contact_person'),
      dataIndex: 'contact_person',
      key: 'contact_person',
      render: (val: string) => val || '-',
    },
    {
      title: t('suppliers.phone'),
      dataIndex: 'phone',
      key: 'phone',
      render: (val: string) => val || '-',
    },
    {
      title: t('suppliers.status'),
      dataIndex: 'is_active',
      key: 'is_active',
      render: (isActive: boolean) => (
        <Tag color={isActive ? 'green' : 'red'}>
          {isActive ? t('common.active') : t('common.inactive')}
        </Tag>
      ),
    },
    {
      title: t('common.actions'),
      key: 'actions',
      render: (_: unknown, record: Supplier) => (
        <>
          <Button
            type="link"
            size="small"
            onClick={() => navigate(`/suppliers/${record.id}/edit`)}
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
      title={t('suppliers.title')}
      extra={
        <Button type="primary" icon={<PlusOutlined />} onClick={() => navigate('/suppliers/new')}>
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
      <DataTable<Supplier>
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
