import { useState } from 'react';
import { Table, Button, Space, Tag, Input, Popconfirm } from 'antd';
import { PlusOutlined, SearchOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
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
      dataIndex: 'code',
      key: 'code',
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
    },
    {
      title: t('customers.phone'),
      dataIndex: 'phone',
      key: 'phone',
    },
    {
      title: t('customers.industry'),
      dataIndex: 'industry',
      key: 'industry',
    },
    {
      title: t('customers.status'),
      dataIndex: 'status',
      key: 'status',
      render: (status: string) => {
        const color = status === 'active' ? 'green' : 'red';
        return <Tag color={color}>{status}</Tag>;
      },
    },
    {
      title: t('common.actions'),
      key: 'actions',
      render: (_: unknown, record: Customer) => (
        <Space>
          <Button
            type="link"
            onClick={() => navigate(`/customers/${record.id}`)}
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
          onClick={() => navigate('/customers/new')}
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
