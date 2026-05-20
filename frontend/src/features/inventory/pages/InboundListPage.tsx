import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import {
  Card,
  Table,
  Button,
  Space,
  Input,
  Select,
  DatePicker,
  Tag,
  Row,
  Col,
} from 'antd';
import { PlusOutlined, SearchOutlined } from '@ant-design/icons';
import type { TablePaginationConfig, ColumnsType } from 'antd/es/table';
import dayjs from 'dayjs';
import { inventoryApi, InboundRecord, InboundFilter } from '../api/inventoryApi';

const { RangePicker } = DatePicker;

const inboundTypeConfig: Record<string, { color: string; label: string }> = {
  purchase: { color: 'blue', label: '采购入库' },
  return: { color: 'orange', label: '退货入库' },
  transfer: { color: 'purple', label: '调拨入库' },
};

const defaultFilter: InboundFilter = {
  page: 1,
  page_size: 20,
};

export default function InboundListPage() {
  const navigate = useNavigate();
  const [filter, setFilter] = useState<InboundFilter>(defaultFilter);
  const [searchNo, setSearchNo] = useState('');
  const [searchType, setSearchType] = useState<string | undefined>();
  const [dateRange, setDateRange] = useState<[dayjs.Dayjs | null, dayjs.Dayjs | null] | null>(null);

  const { data, isLoading } = useQuery({
    queryKey: ['inbound', filter],
    queryFn: () => inventoryApi.listInbound(filter),
  });

  const handleSearch = () => {
    setFilter((prev) => ({
      ...prev,
      inbound_no: searchNo || undefined,
      inbound_type: searchType,
      date_from: dateRange?.[0]?.format('YYYY-MM-DD'),
      date_to: dateRange?.[1]?.format('YYYY-MM-DD'),
      page: 1,
    }));
  };

  const handleReset = () => {
    setSearchNo('');
    setSearchType(undefined);
    setDateRange(null);
    setFilter(defaultFilter);
  };

  const handleTableChange = (pagination: TablePaginationConfig) => {
    setFilter((prev) => ({
      ...prev,
      page: pagination.current || 1,
      page_size: pagination.pageSize || 20,
    }));
  };

  const renderType = (type: string) => {
    const cfg = inboundTypeConfig[type] || { color: 'default', label: type };
    return <Tag color={cfg.color}>{cfg.label}</Tag>;
  };

  const columns: ColumnsType<InboundRecord> = [
    { title: '入库单号', dataIndex: 'inbound_no', key: 'inbound_no', width: 180 },
    {
      title: '类型',
      dataIndex: 'inbound_type',
      key: 'inbound_type',
      width: 110,
      render: (t: string) => renderType(t),
    },
    { title: '入库日期', dataIndex: 'created_at', key: 'created_at', width: 170 },
    { title: '管材数量', dataIndex: 'total_items', key: 'total_items', width: 100, align: 'right' },
    { title: '操作人', dataIndex: 'operator_id', key: 'operator_id', width: 120, ellipsis: true },
    {
      title: '备注',
      dataIndex: 'notes',
      key: 'notes',
      width: 200,
      ellipsis: true,
      render: (v?: string) => v || '-',
    },
  ];

  const listData = data?.data;

  return (
    <Card
      title="入库管理"
      extra={
        <Button
          type="primary"
          icon={<PlusOutlined />}
          onClick={() => navigate('/inventory/inbound/new')}
        >
          新增入库
        </Button>
      }
      styles={{ body: { padding: 16 } }}
    >
      <Row gutter={[12, 12]} style={{ marginBottom: 16 }}>
        <Col xs={24} sm={12} md={6}>
          <Input
            placeholder="搜索入库单号"
            prefix={<SearchOutlined />}
            value={searchNo}
            onChange={(e) => setSearchNo(e.target.value)}
            onPressEnter={handleSearch}
            allowClear
          />
        </Col>
        <Col xs={12} sm={8} md={4}>
          <Select
            placeholder="入库类型"
            style={{ width: '100%' }}
            value={searchType}
            onChange={(v) => setSearchType(v)}
            allowClear
            options={[
              { label: '采购入库', value: 'purchase' },
              { label: '退货入库', value: 'return' },
              { label: '调拨入库', value: 'transfer' },
            ]}
          />
        </Col>
        <Col xs={24} sm={12} md={6}>
          <RangePicker
            style={{ width: '100%' }}
            value={dateRange}
            onChange={(dates) => setDateRange(dates as [dayjs.Dayjs | null, dayjs.Dayjs | null] | null)}
          />
        </Col>
        <Col xs={24} sm={12} md={4}>
          <Space>
            <Button type="primary" icon={<SearchOutlined />} onClick={handleSearch}>
              搜索
            </Button>
            <Button onClick={handleReset}>重置</Button>
          </Space>
        </Col>
      </Row>

      <Table
        columns={columns}
        dataSource={listData?.data}
        rowKey="id"
        loading={isLoading}
        locale={{ emptyText: '暂无入库记录' }}
        scroll={{ x: 860 }}
        pagination={{
          current: listData?.meta?.page || 1,
          pageSize: listData?.meta?.page_size || 20,
          total: listData?.meta?.total || 0,
          showSizeChanger: true,
          showQuickJumper: true,
          showTotal: (total) => `共 ${total} 条`,
        }}
        onChange={handleTableChange}
      />
    </Card>
  );
}
