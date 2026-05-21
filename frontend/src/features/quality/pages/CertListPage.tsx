import { useState } from 'react';
import { Table, Button, Space, Tag, Input, Select, Popconfirm } from 'antd';
import { PlusOutlined, SearchOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useCerts, useDeleteCert } from '../hooks/useQuality';
import type { QualityCert } from '../types';

const STATUS_COLORS: Record<string, string> = {
  draft: 'default',
  active: 'green',
  void: 'red',
};

export default function CertListPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [searchText, setSearchText] = useState('');
  const [statusFilter, setStatusFilter] = useState<string | undefined>();

  const { data, isLoading } = useCerts({
    page,
    page_size: pageSize,
    q: searchText || undefined,
    status: statusFilter,
  });

  const deleteMutation = useDeleteCert();

  const columns = [
    {
      title: t('quality.certificate_id'),
      dataIndex: 'cert_number',
      key: 'cert_number',
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
      title: t('quality.quantity'),
      dataIndex: 'quantity',
      key: 'quantity',
    },
    {
      title: t('quality.inspector'),
      dataIndex: 'inspector',
      key: 'inspector',
    },
    {
      title: t('quality.status'),
      dataIndex: 'status',
      key: 'status',
      render: (status: string) => (
        <Tag color={STATUS_COLORS[status] ?? 'default'}>{status}</Tag>
      ),
    },
    {
      title: t('common.actions'),
      key: 'actions',
      render: (_: unknown, record: QualityCert) => (
        <Space>
          <Button type="link" onClick={() => navigate(`/quality/certs/${record.id}`)}>
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
          <Select
            placeholder={t('quality.status')}
            allowClear
            style={{ width: 120 }}
            value={statusFilter}
            onChange={(v) => setStatusFilter(v)}
          >
            <Select.Option value="draft">draft</Select.Option>
            <Select.Option value="active">active</Select.Option>
            <Select.Option value="void">void</Select.Option>
          </Select>
        </Space>
        <Button
          type="primary"
          icon={<PlusOutlined />}
          onClick={() => navigate('/quality/certs/new')}
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
