// 质检证书列表页 — 使用 DataTable + PageLayout 共享组件
import { useState } from 'react';
import { Button, Tag, Input, Popconfirm } from 'antd';
import { PlusOutlined, SearchOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { PageLayout } from '@/shared/components/PageLayout';
import { DataTable } from '@/shared/components/DataTable';
import { usePagination } from '@/shared/hooks/usePagination';
import { useCerts, useDeleteCert } from '../hooks/useQuality';
import type { QualityCert } from '../types';

export default function CertListPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { page, pageSize, onPaginationChange, reset } = usePagination();
  const [searchText, setSearchText] = useState('');

  const { data, isLoading } = useCerts({
    page,
    page_size: pageSize,
    q: searchText || undefined,
  });

  const deleteMutation = useDeleteCert();

  const columns = [
    {
      title: t('quality.certificate_id'),
      dataIndex: 'cert_number',
      key: 'cert_number',
    },
    {
      title: t('quality.pipe_type'),
      dataIndex: 'pipe_type',
      key: 'pipe_type',
    },
    {
      title: t('quality.result'),
      dataIndex: 'result',
      key: 'result',
      render: (result: string) => (
        <Tag color={result === 'pass' ? 'green' : 'red'}>{result}</Tag>
      ),
    },
    {
      title: t('quality.inspector'),
      dataIndex: 'inspector',
      key: 'inspector',
      render: (val: string | null) => val ?? '-',
    },
    {
      title: t('quality.cert_date'),
      dataIndex: 'cert_date',
      key: 'cert_date',
      render: (val: string | null) => val ?? '-',
    },
    {
      title: t('common.actions'),
      key: 'actions',
      render: (_: unknown, record: QualityCert) => (
        <>
          <Button
            type="link"
            size="small"
            onClick={() => navigate(`/quality/certs/${record.id}`)}
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
      title={t('quality.certificates')}
      extra={
        <Button type="primary" icon={<PlusOutlined />} onClick={() => navigate('/quality/certs/new')}>
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
      <DataTable<QualityCert>
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
