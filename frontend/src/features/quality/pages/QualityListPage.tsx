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
  DatePicker,
  Row,
  Col,
} from 'antd';
import {
  PlusOutlined,
  SearchOutlined,
  EyeOutlined,
  EditOutlined,
  DeleteOutlined,
} from '@ant-design/icons';
import dayjs from 'dayjs';
import type { TablePaginationConfig, ColumnsType } from 'antd/es/table';
import { qualityApi, QualityCert } from '../api/qualityApi';
import type { QualityCertListParams } from '../api/qualityApi';

const { RangePicker } = DatePicker;

const resultConfig: Record<string, { color: string; label: string }> = {
  pass: { color: 'green', label: '合格' },
  fail: { color: 'red', label: '不合格' },
  pending: { color: 'orange', label: '待检' },
};

const pipeTypeOptions = [
  { label: '套管', value: 'casing' },
  { label: '油管', value: 'tubing' },
  { label: '光管', value: 'plain_end' },
];

const resultOptions = [
  { label: '合格', value: 'pass' },
  { label: '不合格', value: 'fail' },
  { label: '待检', value: 'pending' },
];

const defaultFilter: QualityCertListParams = {
  page: 1,
  page_size: 20,
};

export default function QualityListPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [filter, setFilter] = useState<QualityCertListParams>(defaultFilter);
  const [searchText, setSearchText] = useState('');
  const [dateRange, setDateRange] = useState<[dayjs.Dayjs | null, dayjs.Dayjs | null] | null>(null);

  const { data, isLoading } = useQuery({
    queryKey: ['quality-certs', filter],
    queryFn: () => qualityApi.listCert('all', filter),
  });

  const deleteMutation = useMutation({
    mutationFn: (record: QualityCert) =>
      qualityApi.deleteCert(record.pipe_type, record.id),
    onSuccess: () => {
      message.success('删除成功');
      queryClient.invalidateQueries({ queryKey: ['quality-certs'] });
    },
    onError: () => {
      message.error('删除失败');
    },
  });

  const handleSearch = () => {
    setFilter((prev) => ({
      ...prev,
      cert_no: searchText || undefined,
      page: 1,
    }));
  };

  const handleFilterChange = (key: string, value?: string) => {
    setFilter((prev) => ({ ...prev, [key]: value || undefined, page: 1 }));
  };

  const handleDateRangeChange = (_: unknown, dateStrings: [string, string]) => {
    if (dateStrings[0] && dateStrings[1]) {
      setFilter((prev) => ({
        ...prev,
        date_from: dateStrings[0],
        date_to: dateStrings[1],
        page: 1,
      }));
    } else {
      setFilter((prev) => ({
        ...prev,
        date_from: undefined,
        date_to: undefined,
        page: 1,
      }));
    }
  };

  const handleTableChange = (pagination: TablePaginationConfig) => {
    setFilter((prev) => ({
      ...prev,
      page: pagination.current || 1,
      page_size: pagination.pageSize || 20,
    }));
  };

  const renderResult = (result: string) => {
    const cfg = resultConfig[result] || { color: 'default', label: result };
    return <Tag color={cfg.color}>{cfg.label}</Tag>;
  };

  const renderPipeType = (pipeType: string) => {
    const map: Record<string, string> = {
      casing: '套管',
      tubing: '油管',
      plain_end: '光管',
    };
    return map[pipeType] || pipeType;
  };

  const columns: ColumnsType<QualityCert> = [
    {
      title: '证书编号',
      dataIndex: 'cert_no',
      key: 'cert_no',
      width: 180,
      fixed: 'left',
    },
    {
      title: '管材类型',
      dataIndex: 'pipe_type',
      key: 'pipe_type',
      width: 100,
      render: (v: string) => renderPipeType(v),
    },
    {
      title: '管材编号',
      dataIndex: 'pipe_number',
      key: 'pipe_number',
      width: 200,
      ellipsis: true,
    },
    {
      title: '检验日期',
      dataIndex: 'inspect_date',
      key: 'inspect_date',
      width: 120,
    },
    {
      title: '检验人',
      dataIndex: 'inspector',
      key: 'inspector',
      width: 100,
    },
    {
      title: '检验结果',
      dataIndex: 'result',
      key: 'result',
      width: 100,
      render: (v: string) => renderResult(v),
    },
    {
      title: '操作',
      key: 'action',
      width: 200,
      fixed: 'right',
      render: (_, record) => (
        <Space size="small">
          <Button
            type="link"
            size="small"
            icon={<EyeOutlined />}
            onClick={() => navigate(`/quality/certs/${record.id}`)}
          >
            查看
          </Button>
          <Button
            type="link"
            size="small"
            icon={<EditOutlined />}
            onClick={() => navigate(`/quality/certs/${record.id}/edit`)}
          >
            编辑
          </Button>
          <Popconfirm
            title="确认删除"
            description="确定要删除这份质检证书吗？"
            onConfirm={() => deleteMutation.mutate(record)}
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
    <Card title="质检证书管理" styles={{ body: { padding: 16 } }}>
      <Row gutter={[12, 12]} style={{ marginBottom: 16 }}>
        <Col xs={24} sm={12} md={6}>
          <Input
            placeholder="搜索证书编号"
            prefix={<SearchOutlined />}
            value={searchText}
            onChange={(e) => setSearchText(e.target.value)}
            onPressEnter={handleSearch}
            allowClear
          />
        </Col>
        <Col xs={12} sm={6} md={4}>
          <Select
            placeholder="管材类型"
            style={{ width: '100%' }}
            value={filter.pipe_type}
            onChange={(v) => handleFilterChange('pipe_type', v)}
            allowClear
            options={pipeTypeOptions}
          />
        </Col>
        <Col xs={12} sm={6} md={4}>
          <Select
            placeholder="检验结果"
            style={{ width: '100%' }}
            value={filter.result}
            onChange={(v) => handleFilterChange('result', v)}
            allowClear
            options={resultOptions}
          />
        </Col>
        <Col xs={24} sm={12} md={6}>
          <RangePicker
            style={{ width: '100%' }}
            value={dateRange}
            onChange={(dates, dateStrings) => {
              setDateRange(dates as [dayjs.Dayjs | null, dayjs.Dayjs | null] | null);
              handleDateRangeChange(null, dateStrings);
            }}
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
              onClick={() => navigate('/quality/certs/new')}
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
        scroll={{ x: 960 }}
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
