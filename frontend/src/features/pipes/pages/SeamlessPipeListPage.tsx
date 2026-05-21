import { useState } from 'react';
import { Table, Button, Space, Tag, Input, Popconfirm } from 'antd';
import { PlusOutlined, SearchOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useSeamlessPipes, useDeleteSeamlessPipe } from '../hooks/useSeamlessPipes';
import type { SeamlessPipe } from '@/types';

export default function SeamlessPipeListPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [searchText, setSearchText] = useState('');

  const { data, isLoading } = useSeamlessPipes({
    page,
    page_size: pageSize,
    q: searchText || undefined,
  });

  const deleteMutation = useDeleteSeamlessPipe();

  const columns = [
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
      title: t('pipes.status'),
      dataIndex: 'status',
      key: 'status',
      render: (status: string) => {
        const color =
          status === 'in_stock' ? 'green' : status === 'outbound' ? 'orange' : 'red';
        const statusKey = `pipes.${status}`;
        return <Tag color={color}>{t(statusKey)}</Tag>;
      },
    },
    {
      title: t('common.actions'),
      key: 'actions',
      render: (_: unknown, record: SeamlessPipe) => (
        <Space>
          <Button
            type="link"
            onClick={() => navigate(`/pipes/seamless/${record.id}`)}
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
          onClick={() => navigate('/pipes/seamless/new')}
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
