// Customer list page — uses DataTable + PageLayout shared components
import { useState } from 'react';
import { Button, Tag, Input, Popconfirm } from 'antd';
import { PlusOutlined, SearchOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { PageLayout } from '@/shared/components/PageLayout';
import { DataTable } from '@/shared/components/DataTable';
import { useCustomers, useDeleteCustomer } from '../hooks/useCustomers';
import type { Customer } from '../types';

export default function CustomerListPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [searchText, setSearchText] = useState('');

  const { data, isLoading } = useCustomers({
    page,
    page_size: pageSize,
    q: searchText || undefined,
  });

  const deleteMutation = useDeleteCustomer();

  const columns = [
    {
      title: t('customers.code'),
      dataIndex: 'customer_code',
      key: 'customer_code',
      sorter: true,
    },
    {
      title: t('customers.name'),
      dataIndex: 'name',
      key: 'name',
    },
    {
      title: t('customers.contact_person'),
      dataIndex: 'contact_person',
      key: 'contact_person',
      render: (val: string) => val || '-',
    },
    {
      title: t('customers.phone'),
      dataIndex: 'phone',
      key: 'phone',
      render: (val: string) => val || '-',
    },
    {
      title: t('customers.status'),
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
      render: (_: unknown, record: Customer) => (
        <>
          <Button
            type="link"
            size="small"
            onClick={() => navigate(`/customers/${record.id}/edit`)}
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
      title={t('customers.title')}
      extra={
        <Button type="primary" icon={<PlusOutlined />} onClick={() => navigate('/customers/new')}>
          {t('common.create')}
        </Button>
      }
    >
      <Input
        placeholder={t('common.search')}
        prefix={<SearchOutlined />}
        value={searchText}
        onChange={(e) => setSearchText(e.target.value)}
        style={{ width: 250, marginBottom: 16 }}
        allowClear
      />
      <DataTable<Customer>
        columns={columns}
        items={data?.items}
        total={data?.total}
        page={page}
        pageSize={pageSize}
        loading={isLoading}
        onPaginationChange={(p, ps) => {
          setPage(p);
          setPageSize(ps);
        }}
      />
    </PageLayout>
  );
}
