// 合同列表页 — 使用 DataTable + PageLayout 共享组件
import { useState } from 'react';
import { Button, Tag, Input, Popconfirm, Select } from 'antd';
import { PlusOutlined, SearchOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { PageLayout } from '@/shared/components/PageLayout';
import { DataTable } from '@/shared/components/DataTable';
import { usePagination } from '@/shared/hooks/usePagination';
import { useContracts, useDeleteContract } from '../hooks/useContracts';
import type { Contract } from '../types';

const STATUS_COLORS: Record<string, string> = {
  draft: 'default',
  active: 'processing',
  completed: 'success',
  terminated: 'error',
};

const TYPE_COLORS: Record<string, string> = {
  purchase: 'blue',
  sales: 'green',
};

const CONTRACT_STATUS_OPTIONS = ['draft', 'active', 'completed', 'terminated'] as const;

export default function ContractListPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { page, pageSize, onPaginationChange, reset } = usePagination();
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
      title: t('contracts.contract_number'),
      dataIndex: 'contract_no',
      key: 'contract_no',
    },
    {
      title: t('contracts.contract_name'),
      dataIndex: 'title',
      key: 'title',
    },
    {
      title: t('contracts.contract_type'),
      dataIndex: 'contract_type',
      key: 'contract_type',
      render: (type: string) => <Tag color={TYPE_COLORS[type]}>{type}</Tag>,
    },
    {
      title: t('contracts.party_a'),
      dataIndex: 'party_a',
      key: 'party_a',
    },
    {
      title: t('contracts.party_b'),
      dataIndex: 'party_b',
      key: 'party_b',
    },
    {
      title: t('contracts.total_amount'),
      dataIndex: 'total_amount',
      key: 'total_amount',
      align: 'right' as const,
      render: (val: number | null) => val != null ? val.toLocaleString() : '-',
    },
    {
      title: t('contracts.status'),
      dataIndex: 'status',
      key: 'status',
      render: (status: string) => (
        <Tag color={STATUS_COLORS[status]}>{status}</Tag>
      ),
    },
    {
      title: t('common.actions'),
      key: 'actions',
      render: (_: unknown, record: Contract) => (
        <>
          <Button
            type="link"
            size="small"
            onClick={() => navigate(`/contracts/${record.id}`)}
          >
            {t('common.detail')}
          </Button>
          <Button
            type="link"
            size="small"
            onClick={() => navigate(`/contracts/${record.id}/edit`)}
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
      title={t('contracts.title')}
      extra={
        <Button type="primary" icon={<PlusOutlined />} onClick={() => navigate('/contracts/new')}>
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
          placeholder={t('contracts.contract_type')}
          allowClear
          style={{ width: 140 }}
          value={typeFilter}
          onChange={setTypeFilter}
          options={[
            { label: t('contracts.purchase'), value: 'purchase' },
            { label: t('contracts.sales'), value: 'sales' },
          ]}
        />
        <Select
          placeholder={t('contracts.status')}
          allowClear
          style={{ width: 140 }}
          value={statusFilter}
          onChange={setStatusFilter}
          options={CONTRACT_STATUS_OPTIONS.map((s) => ({
            label: t(`contracts.status.${s}`),
            value: s,
          }))}
        />
      </div>
      <DataTable<Contract>
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
