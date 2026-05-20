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
  Row,
  Col,
} from 'antd';
import { PlusOutlined, SearchOutlined, EditOutlined, DeleteOutlined } from '@ant-design/icons';
import type { TablePaginationConfig, ColumnsType } from 'antd/es/table';
import { supplierApi, Supplier, SupplierFilter } from '../api/supplierApi';

const defaultFilter: SupplierFilter = {
  page: 1,
  page_size: 20,
};

export default function SupplierListPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [filter, setFilter] = useState<SupplierFilter>(defaultFilter);
  const [searchText, setSearchText] = useState('');

  const { data, isLoading } = useQuery({
    queryKey: ['suppliers', filter],
    queryFn: () => supplierApi.list(filter),
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => supplierApi.delete(id),
    onSuccess: () => {
      message.success('删除成功');
      queryClient.invalidateQueries({ queryKey: ['suppliers'] });
    },
    onError: () => {
      message.error('删除失败');
    },
  });

  const handleSearch = () => {
    setFilter((prev) => ({ ...prev, search: searchText || undefined, page: 1 }));
  };

  const handleFilterChange = (key: string, value?: unknown) => {
    setFilter((prev) => ({ ...prev, [key]: value ?? undefined, page: 1 }));
  };

  const handleTableChange = (pagination: TablePaginationConfig) => {
    setFilter((prev) => ({
      ...prev,
      page: pagination.current || 1,
      page_size: pagination.pageSize || 20,
    }));
  };

  const columns: ColumnsType<Supplier> = [
    { title: '供应商名称', dataIndex: 'name', key: 'name', width: 200 },
    { title: '联系人', dataIndex: 'contact_person', key: 'contact_person', width: 120 },
    { title: '电话', dataIndex: 'phone', key: 'phone', width: 140 },
    { title: '邮箱', dataIndex: 'email', key: 'email', width: 180, ellipsis: true },
    {
      title: '状态',
      dataIndex: 'is_active',
      key: 'is_active',
      width: 80,
      render: (v: boolean) => (v ? <Tag color="green">启用</Tag> : <Tag color="red">停用</Tag>),
    },
    {
      title: '操作',
      key: 'action',
      width: 140,
      fixed: 'right',
      render: (_, r) => (
        <Space size="small">
          <Button
            type="link"
            size="small"
            icon={<EditOutlined />}
            onClick={() => navigate(`/purchases/suppliers/${r.id}/edit`)}
          >
            编辑
          </Button>
          <Popconfirm
            title="确认删除"
            description="确定要删除此供应商吗？"
            onConfirm={() => deleteMutation.mutate(r.id)}
            okText="确定"
            cancelText="取消"
          >
            <Button type="link" size="small" danger icon={<DeleteOutlined />}>
              删除
            </Button>
          </Popconfirm>
        </Space>
      ),
    },
  ];

  return (
    <Card title="供应商管理" styles={{ body: { padding: 16 } }}>
      <Row gutter={[12, 12]} style={{ marginBottom: 16 }}>
        <Col xs={24} sm={12} md={6}>
          <Input
            placeholder="搜索供应商名称"
            prefix={<SearchOutlined />}
            value={searchText}
            onChange={(e) => setSearchText(e.target.value)}
            onPressEnter={handleSearch}
            allowClear
          />
        </Col>
        <Col xs={12} sm={6} md={4}>
          <Select
            placeholder="状态"
            style={{ width: '100%' }}
            value={filter.is_active}
            onChange={(v) => handleFilterChange('is_active', v)}
            allowClear
            options={[
              { label: '启用', value: true },
              { label: '停用', value: false },
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
              onClick={() => navigate('/purchases/suppliers/new')}
            >
              新增
            </Button>
          </Space>
        </Col>
      </Row>

      <Table
        columns={columns}
        dataSource={data?.data?.data}
        rowKey="id"
        loading={isLoading}
        locale={{ emptyText: '暂无数据' }}
        scroll={{ x: 860 }}
        pagination={{
          current: data?.data?.meta?.page || 1,
          pageSize: data?.data?.meta?.page_size || 20,
          total: data?.data?.meta?.total || 0,
          showSizeChanger: true,
          showQuickJumper: true,
          showTotal: (total) => `共 ${total} 条`,
        }}
        onChange={handleTableChange}
      />
    </Card>
  );
}
