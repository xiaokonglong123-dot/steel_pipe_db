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
  DatePicker,
} from 'antd';
import { PlusOutlined, SearchOutlined, EditOutlined, DeleteOutlined, EyeOutlined } from '@ant-design/icons';
import type { TablePaginationConfig, ColumnsType } from 'antd/es/table';
import dayjs from 'dayjs';
import { contractApi, Contract, ContractFilter } from '../api/contractApi';

const { RangePicker } = DatePicker;

const contractTypeOptions = [
  { label: '销售合同', value: 'sales' },
  { label: '采购合同', value: 'purchase' },
];

const statusConfig: Record<string, { color: string; label: string }> = {
  draft: { color: 'default', label: '草稿' },
  active: { color: 'blue', label: '生效中' },
  completed: { color: 'green', label: '已完成' },
  terminated: { color: 'orange', label: '已终止' },
  cancelled: { color: 'red', label: '已取消' },
};

const statusOptions = [
  { label: '草稿', value: 'draft' },
  { label: '生效中', value: 'active' },
  { label: '已完成', value: 'completed' },
  { label: '已终止', value: 'terminated' },
  { label: '已取消', value: 'cancelled' },
];

const defaultFilter: ContractFilter = {
  page: 1,
  page_size: 20,
};

export default function ContractListPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [filter, setFilter] = useState<ContractFilter>(defaultFilter);
  const [searchText, setSearchText] = useState('');
  const [dateRange, setDateRange] = useState<[dayjs.Dayjs | null, dayjs.Dayjs | null] | null>(null);

  const { data: listData, isLoading } = useQuery({
    queryKey: ['contracts', filter],
    queryFn: () => contractApi.listContracts(filter),
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => contractApi.deleteContract(id),
    onSuccess: () => {
      message.success('删除成功');
      queryClient.invalidateQueries({ queryKey: ['contracts'] });
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

  const handleDateChange = (dates: unknown) => {
    const range = dates as [dayjs.Dayjs | null, dayjs.Dayjs | null] | null;
    setDateRange(range);
    setFilter((prev) => ({
      ...prev,
      date_from: range?.[0]?.format('YYYY-MM-DD') || undefined,
      date_to: range?.[1]?.format('YYYY-MM-DD') || undefined,
      page: 1,
    }));
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

  const renderContractType = (type: string) => {
    const opt = contractTypeOptions.find((o) => o.value === type);
    return opt?.label || type;
  };

  const renderActions = (record: Contract) => (
    <Space size="small">
      <Button
        type="link"
        size="small"
        icon={<EyeOutlined />}
        onClick={() => navigate(`/contracts/${record.id}`)}
      >
        查看
      </Button>
      <Button
        type="link"
        size="small"
        icon={<EditOutlined />}
        onClick={() => navigate(`/contracts/${record.id}/edit`)}
      >
        编辑
      </Button>
      <Popconfirm
        title="确认删除"
        description="确定要删除这份合同吗？"
        onConfirm={() => deleteMutation.mutate(record.id)}
        okText="确定"
        cancelText="取消"
      >
        <Button type="link" size="small" danger icon={<DeleteOutlined />}>
          删除
        </Button>
      </Popconfirm>
    </Space>
  );

  const columns: ColumnsType<Contract> = [
    { title: '合同编号', dataIndex: 'contract_no', key: 'contract_no', width: 180, fixed: 'left' },
    {
      title: '合同类型',
      dataIndex: 'contract_type',
      key: 'contract_type',
      width: 100,
      render: (t: string) => renderContractType(t),
    },
    { title: '对方名称', dataIndex: 'party_name', key: 'party_name', width: 180, ellipsis: true },
    {
      title: '总金额',
      dataIndex: 'total_amount',
      key: 'total_amount',
      width: 130,
      align: 'right',
      render: (v: number) => `¥${v.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`,
    },
    { title: '状态', dataIndex: 'status', key: 'status', width: 100, render: (s: string) => renderStatus(s) },
    {
      title: '签订日期',
      dataIndex: 'sign_date',
      key: 'sign_date',
      width: 110,
      render: (v?: string) => (v ? dayjs(v).format('YYYY-MM-DD') : '-'),
    },
    {
      title: '生效日期',
      dataIndex: 'effective_date',
      key: 'effective_date',
      width: 110,
      render: (v?: string) => (v ? dayjs(v).format('YYYY-MM-DD') : '-'),
    },
    { title: '操作', key: 'action', width: 200, fixed: 'right', render: (_, r) => renderActions(r) },
  ];

  return (
    <Card title="合同管理" styles={{ body: { padding: 16 } }}>
      <Row gutter={[12, 12]} style={{ marginBottom: 16 }}>
        <Col xs={24} sm={12} md={6}>
          <Input
            placeholder="搜索合同编号"
            prefix={<SearchOutlined />}
            value={searchText}
            onChange={(e) => setSearchText(e.target.value)}
            onPressEnter={handleSearch}
            allowClear
          />
        </Col>
        <Col xs={12} sm={6} md={4}>
          <Select
            placeholder="合同类型"
            style={{ width: '100%' }}
            value={filter.contract_type}
            onChange={(v) => handleFilterChange('contract_type', v)}
            allowClear
            options={contractTypeOptions}
          />
        </Col>
        <Col xs={12} sm={6} md={4}>
          <Select
            placeholder="状态"
            style={{ width: '100%' }}
            value={filter.status}
            onChange={(v) => handleFilterChange('status', v)}
            allowClear
            options={statusOptions}
          />
        </Col>
        <Col xs={24} sm={12} md={6}>
          <RangePicker
            style={{ width: '100%' }}
            value={dateRange}
            onChange={handleDateChange}
            placeholder={['开始日期', '结束日期']}
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
              onClick={() => navigate('/contracts/new')}
            >
              新增
            </Button>
          </Space>
        </Col>
      </Row>

      <Table
        columns={columns}
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
