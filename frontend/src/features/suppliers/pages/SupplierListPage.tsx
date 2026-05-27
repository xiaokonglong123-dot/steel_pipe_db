// Supplier list page — paginated query, search, status tags, CRUD
import { useState } from 'react';
import { Table, Button, Space, Tag, Input, Popconfirm } from 'antd';
import { PlusOutlined, SearchOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useSuppliers, useDeleteSupplier } from '../hooks/useSuppliers';
import type { Supplier } from '../types';

export default function SupplierListPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
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
      dataIndex: 'code',
      key: 'code',
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
    },
    {
      title: t('suppliers.phone'),
      dataIndex: 'phone',
      key: 'phone',
    },
    {
      title: t('suppliers.grade_supply'),
      dataIndex: 'grade_supply',
      key: 'grade_supply',
      render: (val: string) =>
        val
          ? val.split(',').map((g) => (
              <Tag key={g.trim()} color="blue" style={{ marginBottom: 2 }}>
                {g.trim()}
              </Tag>
            ))
          : '-',
    },
    {
      title: t('suppliers.status'),
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
      render: (_: unknown, record: Supplier) => (
        <Space>
          <Button
            type="link"
            onClick={() => navigate(`/suppliers/${record.id}`)}
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
          onClick={() => navigate('/suppliers/new')}
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
