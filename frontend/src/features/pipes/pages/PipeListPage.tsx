import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  Card,
  Table,
  Button,
  Space,
  Input,
  Select,
  Tag,
  Popconfirm,
  message,
  Tabs,
  Row,
  Col,
} from 'antd';
import { PlusOutlined, SearchOutlined, EditOutlined, DeleteOutlined } from '@ant-design/icons';
import type { TablePaginationConfig, ColumnsType } from 'antd/es/table';
import { pipeApi, SeamlessPipe, ScreenPipe, PipeFilter } from '../api/pipeApi';
import type { PipeCategory } from '../../../shared/types';

const statusConfig: Record<string, { color: string; label: string }> = {
  in_stock: { color: 'green', label: '在库' },
  outbound: { color: 'blue', label: '已出库' },
  scrapped: { color: 'red', label: '报废' },
};

const defaultFilter: PipeFilter = {
  page: 1,
  page_size: 20,
};

export default function PipeListPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [activeTab, setActiveTab] = useState<PipeCategory>('seamless');
  const [filter, setFilter] = useState<PipeFilter>(defaultFilter);
  const [searchText, setSearchText] = useState('');

  const isSeamless = activeTab === 'seamless';

  const { data: seamlessData, isLoading: seamlessLoading } = useQuery({
    queryKey: ['seamless-pipes', filter],
    queryFn: () => pipeApi.listSeamless(filter),
    enabled: isSeamless,
  });

  const { data: screenData, isLoading: screenLoading } = useQuery({
    queryKey: ['screen-pipes', filter],
    queryFn: () => pipeApi.listScreen(filter),
    enabled: !isSeamless,
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) =>
      isSeamless ? pipeApi.deleteSeamless(id) : pipeApi.deleteScreen(id),
    onSuccess: () => {
      message.success('删除成功');
      queryClient.invalidateQueries({ queryKey: [isSeamless ? 'seamless-pipes' : 'screen-pipes'] });
    },
    onError: () => {
      message.error('删除失败');
    },
  });

  const handleSearch = () => {
    setFilter((prev) => ({ ...prev, search: searchText || undefined, page: 1 }));
  };

  const handleFilterChange = (key: string, value?: string) => {
    setFilter((prev) => ({ ...prev, [key]: value || undefined, page: 1 }));
  };

  const handleTableChange = (pagination: TablePaginationConfig) => {
    setFilter((prev) => ({
      ...prev,
      page: pagination.current || 1,
      page_size: pagination.pageSize || 20,
    }));
  };

  const renderStatus = (status: string) => {
    const cfg = statusConfig[status] || { color: 'default', label: status };
    return <Tag color={cfg.color}>{cfg.label}</Tag>;
  };

  const renderActions = (id: string) => (
    <Space size="small">
      <Button
        type="link"
        size="small"
        icon={<EditOutlined />}
        onClick={() => navigate(`/pipes/${id}/edit`)}
      >
        编辑
      </Button>
      <Popconfirm
        title="确认删除"
        description="确定要删除这条管材记录吗？"
        onConfirm={() => deleteMutation.mutate(id)}
        okText="确定"
        cancelText="取消"
      >
        <Button type="link" size="small" danger icon={<DeleteOutlined />}>
          删除
        </Button>
      </Popconfirm>
    </Space>
  );

  const seamlessColumns: ColumnsType<SeamlessPipe> = [
    { title: '管材编号', dataIndex: 'pipe_number', key: 'pipe_number', width: 200, fixed: 'left' },
    { title: '钢级', dataIndex: 'grade', key: 'grade', width: 100 },
    { title: '规格', key: 'spec', width: 140, render: (_, r) => `${r.od}×${r.wt}` },
    { title: '长度 (m)', dataIndex: 'length', key: 'length', width: 100, align: 'right' },
    { title: '重量 (kg)', dataIndex: 'weight', key: 'weight', width: 100, align: 'right' },
    { title: '状态', dataIndex: 'status', key: 'status', width: 100, render: (s: string) => renderStatus(s) },
    { title: '库位', dataIndex: 'location', key: 'location', width: 120, ellipsis: true },
    { title: '操作', key: 'action', width: 140, fixed: 'right', render: (_, r) => renderActions(r.id) },
  ];

  const screenColumns: ColumnsType<ScreenPipe> = [
    { title: '管材编号', dataIndex: 'pipe_number', key: 'pipe_number', width: 200, fixed: 'left' },
    { title: '钢级', dataIndex: 'grade', key: 'grade', width: 100 },
    { title: '规格', key: 'spec', width: 140, render: (_, r) => `${r.od}×${r.wt}` },
    { title: '长度 (m)', dataIndex: 'length', key: 'length', width: 100, align: 'right' },
    { title: '重量 (kg)', dataIndex: 'weight', key: 'weight', width: 100, align: 'right' },
    {
      title: '筛管类型', dataIndex: 'screen_type', key: 'screen_type', width: 100,
    },
    {
      title: '缝宽 (mm)', dataIndex: 'slot_width', key: 'slot_width', width: 100, align: 'right',
      render: (v?: number) => (v != null ? v : '-'),
    },
    { title: '状态', dataIndex: 'status', key: 'status', width: 100, render: (s: string) => renderStatus(s) },
    { title: '库位', dataIndex: 'location', key: 'location', width: 120, ellipsis: true },
    { title: '操作', key: 'action', width: 140, fixed: 'right', render: (_, r) => renderActions(r.id) },
  ];

  const listData = isSeamless ? seamlessData : screenData;
  const isLoading = isSeamless ? seamlessLoading : screenLoading;

  return (
    <Card title="管材管理" styles={{ body: { padding: 16 } }}>
      <Row gutter={[12, 12]} style={{ marginBottom: 16 }}>
        <Col xs={24} sm={12} md={6}>
          <Input
            placeholder="搜索管材编号"
            prefix={<SearchOutlined />}
            value={searchText}
            onChange={(e) => setSearchText(e.target.value)}
            onPressEnter={handleSearch}
            allowClear
          />
        </Col>
        <Col xs={12} sm={6} md={4}>
          <Select
            placeholder="钢级"
            style={{ width: '100%' }}
            value={filter.grade}
            onChange={(v) => handleFilterChange('grade', v)}
            allowClear
            options={[
              { label: 'J55', value: 'J55' },
              { label: 'K55', value: 'K55' },
              { label: 'N80', value: 'N80' },
              { label: 'L80', value: 'L80' },
              { label: 'P110', value: 'P110' },
            ]}
          />
        </Col>
        <Col xs={12} sm={6} md={4}>
          <Select
            placeholder="状态"
            style={{ width: '100%' }}
            value={filter.status}
            onChange={(v) => handleFilterChange('status', v)}
            allowClear
            options={[
              { label: '在库', value: 'in_stock' },
              { label: '已出库', value: 'outbound' },
              { label: '报废', value: 'scrapped' },
            ]}
          />
        </Col>
        <Col xs={24} sm={12} md={4}>
          <Space>
            <Button type="primary" icon={<SearchOutlined />} onClick={handleSearch}>
              搜索
            </Button>
            <Button
              icon={<PlusOutlined />}
              type="primary"
              ghost
              onClick={() => navigate('/pipes/new')}
            >
              新增
            </Button>
          </Space>
        </Col>
      </Row>

      <Tabs
        activeKey={activeTab}
        onChange={(key) => {
          setActiveTab(key as PipeCategory);
          setFilter(defaultFilter);
          setSearchText('');
        }}
        items={[
          { key: 'seamless', label: '无缝钢管' },
          { key: 'screen', label: '筛管' },
        ]}
      />

      <Table
        columns={(isSeamless ? seamlessColumns : screenColumns) as ColumnsType<SeamlessPipe | ScreenPipe>}
        dataSource={listData?.data?.data}
        rowKey="id"
        loading={isLoading}
        locale={{ emptyText: '暂无数据' }}
        scroll={{ x: 960 }}
        pagination={{
          current: listData?.data?.meta?.page || 1,
          pageSize: listData?.data?.meta?.page_size || 20,
          total: listData?.data?.meta?.total || 0,
          showSizeChanger: true,
          showQuickJumper: true,
          showTotal: (total) => `共 ${total} 条`,
        }}
        onChange={handleTableChange}
      />
    </Card>
  );
}
