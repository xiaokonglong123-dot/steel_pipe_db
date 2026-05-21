import { useState } from 'react';
import { Table, Button, Space, Tag, Input, Select, Popconfirm } from 'antd';
import { PlusOutlined, SearchOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useContracts, useDeleteContract } from '../hooks/useContracts';
import type { Contract } from '../types';

const statusColors: Record<string, string> = {
  draft: 'default',
  active: 'processing',
  completed: 'success',
  terminated: 'error',
};

const typeColors: Record<string, string> = {
  purchase: 'blue',
  sales: 'green',
};

export default function ContractListPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [searchText, setSearchText] = useState('');
  const [statusFilter, setStatusFilter] = useState<string | undefined>();
  const [typeFilter, setTypeFilter] = useState<string | undefined>();

  const { data, isLoading } = useContracts({
    page,
    page_size: pageSize,
    q: searchText || undefined,
    status: statusFilter,
    contract_type: typeFilter,
  });

  const deleteMutation = useDeleteContract();

  const columns = [
    {
      title: t('Contract Number'),
      dataIndex: 'contract_number',
      key: 'contract_number',
    },
    {
      title: t('Contract Name'),
      dataIndex: 'contract_name',
      key: 'contract_name',
    },
    {
      title: t('Contract Type'),
      dataIndex: 'contract_type',
      key: 'contract_type',
      render: (type: string) => <Tag color={typeColors[type]}>{type}</Tag>,
    },
    {
      title: t('Party A'),
      dataIndex: 'party_a',
      key: 'party_a',
    },
    {
      title: t('Party B'),
      dataIndex: 'party_b',
      key: 'party_b',
    },
    {
      title: t('Total Amount'),
      dataIndex: 'total_amount',
      key: 'total_amount',
      align: 'right' as const,
      render: (val: number) => val?.toLocaleString(),
    },
    {
      title: t('Status'),
      dataIndex: 'status',
      key: 'status',
      render: (status: string) => (
        <Tag color={statusColors[status]}>{status}</Tag>
      ),
    },
    {
      title: t('common.actions'),
      key: 'actions',
      render: (_: unknown, record: Contract) => (
        <Space>
          <Button
            type="link"
            onClick={() => navigate(`/contracts/${record.id}`)}
          >
            {t('common.detail')}
          </Button>
          <Button
            type="link"
            onClick={() => navigate(`/contracts/${record.id}/edit`)}
          >
            {t('common.edit')}
          </Button>
          <Popconfirm
            title="确认删除?"
            onConfirm={() => deleteMutation.mutate(record.id)}
          >
            <Button type="link" danger>{t('common.delete')}</Button>
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
            placeholder={t('Contract Type')}
            allowClear
            style={{ width: 140 }}
            value={typeFilter}
            onChange={setTypeFilter}
            options={[
              { label: 'Purchase', value: 'purchase' },
              { label: 'Sales', value: 'sales' },
            ]}
          />
          <Select
            placeholder={t('Status')}
            allowClear
            style={{ width: 140 }}
            value={statusFilter}
            onChange={setStatusFilter}
            options={[
              { label: 'Draft', value: 'draft' },
              { label: 'Active', value: 'active' },
              { label: 'Completed', value: 'completed' },
              { label: 'Terminated', value: 'terminated' },
            ]}
          />
        </Space>
        <Button
          type="primary"
          icon={<PlusOutlined />}
          onClick={() => navigate('/contracts/new')}
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
