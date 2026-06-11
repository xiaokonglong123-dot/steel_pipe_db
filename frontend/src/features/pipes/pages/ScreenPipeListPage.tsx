// 筛管列表页 — 使用 DataTable + PageLayout 共享组件
import { useState } from 'react';
import { Button, Tag, Input, Popconfirm } from 'antd';
import { PlusOutlined, SearchOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { PageLayout } from '@/shared/components/PageLayout';
import { DataTable } from '@/shared/components/DataTable';
import { usePagination } from '@/shared/hooks/usePagination';
import { useScreenPipes, useDeleteScreenPipe } from '../hooks/useScreenPipes';
import type { ScreenPipe } from '@/types';

export default function ScreenPipeListPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { page, pageSize, onPaginationChange, reset } = usePagination();
  const [searchText, setSearchText] = useState('');

  const { data, isLoading } = useScreenPipes({
    page,
    page_size: pageSize,
    q: searchText || undefined,
  });

  const deleteMutation = useDeleteScreenPipe();

  const columns = [
    {
      title: t('pipes.pipe_number'),
      dataIndex: 'pipe_number',
      key: 'pipe_number',
      sorter: true,
    },
    {
      title: t('screen_pipes.screen_type'),
      dataIndex: 'screen_type',
      key: 'screen_type',
      render: (type: string) => <Tag color="cyan">{type}</Tag>,
    },
    {
      title: t('screen_pipes.base_grade'),
      dataIndex: 'base_grade',
      key: 'base_grade',
      render: (grade: string) => <Tag color="blue">{grade}</Tag>,
    },
    {
      title: t('pipes.od'),
      dataIndex: 'base_od',
      key: 'base_od',
    },
    {
      title: t('pipes.status'),
      dataIndex: 'status',
      key: 'status',
      render: (status: string) => {
        const color =
          status === 'in_stock' ? 'green' : status === 'outbound' ? 'orange' : 'red';
        return <Tag color={color}>{t(`pipes.status.${status}`)}</Tag>;
      },
    },
    {
      title: t('common.actions'),
      key: 'actions',
      render: (_: unknown, record: ScreenPipe) => (
        <>
          <Button
            type="link"
            size="small"
            onClick={() => navigate(`/pipes/screen/${record.id}`)}
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
      title={t('pipes.screen_pipes')}
      extra={
        <Button type="primary" icon={<PlusOutlined />} onClick={() => navigate('/pipes/screen/new')}>
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
      <DataTable<ScreenPipe>
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
