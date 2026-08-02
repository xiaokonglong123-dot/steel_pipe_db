// 焊接钢管列表页 — 使用 DataTable + PageLayout 共享组件
import { useState, useMemo } from 'react';
import { Button, Tag, Input, Popconfirm } from 'antd';
import { PlusOutlined, SearchOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { PageLayout } from '@/shared/components/PageLayout';
import { DataTable } from '@/shared/components/DataTable';
import { usePagination } from '@/shared/hooks/usePagination';
import { useWeldedPipes, useDeleteWeldedPipe } from '../hooks/useWeldedPipes';
import type { WeldedPipe } from '@/types';

export default function WeldedPipeListPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { page, pageSize, onPaginationChange, reset } = usePagination();
  const [searchText, setSearchText] = useState('');

  const { data, isLoading } = useWeldedPipes({
    page,
    page_size: pageSize,
    q: searchText || undefined,
  });

  const deleteMutation = useDeleteWeldedPipe();

  const columns = useMemo(() => [
    {
      title: t('pipes.pipe_number'),
      dataIndex: 'pipe_number',
      key: 'pipe_number',
      sorter: true,
    },
    {
      title: t('pipes.grade'),
      dataIndex: 'grade',
      key: 'grade',
      render: (grade: string) => <Tag color="blue">{grade}</Tag>,
    },
    {
      title: t('pipes.od'),
      dataIndex: 'od',
      key: 'od',
    },
    {
      title: t('pipes.wt'),
      dataIndex: 'wt',
      key: 'wt',
    },
    {
      title: t('pipes.pipe_type'),
      dataIndex: 'pipe_type',
      key: 'pipe_type',
      render: (pt: string) => t(`pipe_type.${pt}`),
    },
    {
      title: t('pipes.status'),
      dataIndex: 'status',
      key: 'status',
      render: (status: string) => {
        const color =
          status === 'in_stock' ? 'green' : status === 'outbound' ? 'orange' : 'red';
        return <Tag color={color}>{t(`pipes.${status}`)}</Tag>;
      },
    },
    {
      title: t('common.actions'),
      key: 'actions',
      render: (_: unknown, record: WeldedPipe) => (
        <>
          <Button
            type="link"
            size="small"
            onClick={() => navigate(`/pipes/welded/${record.id}/edit`)}
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
  ], [t, navigate]);

  return (
    <PageLayout
      title={t('pipes.welded_pipes')}
      extra={
        <Button type="primary" icon={<PlusOutlined />} onClick={() => navigate('/pipes/welded/new')}>
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
      <DataTable<WeldedPipe>
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
