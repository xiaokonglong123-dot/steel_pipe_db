import React, { useState, useMemo } from 'react';
import { useQuery } from '@tanstack/react-query';
import {
  Card,
  Table,
  Button,
  Space,
  Row,
  Col,
  Tabs,
  Statistic,
  DatePicker,
  message,
} from 'antd';
import { DownloadOutlined, SearchOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import dayjs from 'dayjs';
import {
  reportApi,
  InventoryReport,
  OrderReport,
  ReportFilter,
} from '../api/reportApi';
import { dataIoApi } from '../../data-io/api/dataIoApi';

type TabKey = 'inventory' | 'orders';

export default function ReportDashboardPage() {
  const [activeTab, setActiveTab] = useState<TabKey>('inventory');
  const [draftRange, setDraftRange] = useState<[dayjs.Dayjs | null, dayjs.Dayjs | null]>([null, null]);
  const [appliedRange, setAppliedRange] = useState<[dayjs.Dayjs | null, dayjs.Dayjs | null]>([null, null]);

  const filter: ReportFilter = useMemo(() => {
    const [from, to] = appliedRange;
    return {
      date_from: from ? from.format('YYYY-MM-DD') : undefined,
      date_to: to ? to.format('YYYY-MM-DD') : undefined,
    };
  }, [appliedRange]);

  const { data: inventoryData, isLoading: inventoryLoading } = useQuery({
    queryKey: ['reports', 'inventory', filter],
    queryFn: () => reportApi.getStockSummary(filter),
    enabled: activeTab === 'inventory',
  });

  const { data: orderData, isLoading: orderLoading } = useQuery({
    queryKey: ['reports', 'orders', filter],
    queryFn: () => reportApi.getMonthlyFlow(filter),
    enabled: activeTab === 'orders',
  });

  const inventorySummary = useMemo(() => {
    const data = inventoryData?.data?.data ?? [];
    return {
      total: data.reduce((sum, r) => sum + r.total, 0),
      in_stock: data.reduce((sum, r) => sum + r.in_stock, 0),
      outbound: data.reduce((sum, r) => sum + r.outbound, 0),
      scrapped: data.reduce((sum, r) => sum + r.scrapped, 0),
    };
  }, [inventoryData]);

  const orderSummary = useMemo(() => {
    const data = orderData?.data?.data ?? [];
    return {
      po_count: data.reduce((sum, r) => sum + r.po_count, 0),
      so_count: data.reduce((sum, r) => sum + r.so_count, 0),
      po_amount: data.reduce((sum, r) => sum + r.po_amount, 0),
      so_amount: data.reduce((sum, r) => sum + r.so_amount, 0),
    };
  }, [orderData]);

  const handleSearch = () => {
    setAppliedRange(draftRange);
  };

  const handleExport = async () => {
    try {
      const exportApi = activeTab === 'inventory' ? dataIoApi.exportInventory : dataIoApi.exportOutbound;
      const response = await exportApi(filter);
      const blob = new Blob([response.data as Blob], {
        type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
      });
      const url = window.URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `${activeTab}_report.xlsx`;
      link.click();
      window.URL.revokeObjectURL(url);
      message.success('导出成功');
    } catch {
      message.error('导出失败');
    }
  };

  const inventoryColumns: ColumnsType<InventoryReport> = [
    { title: '管材类型', dataIndex: 'pipe_type', key: 'pipe_type', width: 120 },
    { title: '钢级', dataIndex: 'grade', key: 'grade', width: 100 },
    { title: '合计', dataIndex: 'total', key: 'total', width: 80, align: 'right' },
    { title: '在库', dataIndex: 'in_stock', key: 'in_stock', width: 80, align: 'right' },
    { title: '已出库', dataIndex: 'outbound', key: 'outbound', width: 80, align: 'right' },
    { title: '报废', dataIndex: 'scrapped', key: 'scrapped', width: 80, align: 'right' },
  ];

  const orderColumns: ColumnsType<OrderReport> = [
    { title: '月份', dataIndex: 'month', key: 'month', width: 120 },
    { title: '采购订单数', dataIndex: 'po_count', key: 'po_count', width: 120, align: 'right' },
    { title: '销售订单数', dataIndex: 'so_count', key: 'so_count', width: 120, align: 'right' },
    {
      title: '采购金额', dataIndex: 'po_amount', key: 'po_amount', width: 140, align: 'right',
      render: (v: number) => `¥${v.toLocaleString()}`,
    },
    {
      title: '销售金额', dataIndex: 'so_amount', key: 'so_amount', width: 140, align: 'right',
      render: (v: number) => `¥${v.toLocaleString()}`,
    },
  ];

  return (
    <Card title="报表看板" styles={{ body: { padding: 16 } }}>
      <Row gutter={[12, 12]} style={{ marginBottom: 16 }}>
        <Col xs={24} sm={12} md={8}>
          <DatePicker.RangePicker
            style={{ width: '100%' }}
            value={draftRange}
            onChange={(dates) =>
              setDraftRange((dates ?? [null, null]) as [dayjs.Dayjs | null, dayjs.Dayjs | null])
            }
            allowClear
          />
        </Col>
        <Col xs={12} sm={6} md={4}>
          <Space>
            <Button type="primary" icon={<SearchOutlined />} onClick={handleSearch}>
              查询
            </Button>
            <Button icon={<DownloadOutlined />} onClick={handleExport}>
              导出
            </Button>
          </Space>
        </Col>
      </Row>

      <Tabs
        activeKey={activeTab}
        onChange={(key) => setActiveTab(key as TabKey)}
        items={[
          { key: 'inventory', label: '库存报表' },
          { key: 'orders', label: '订单报表' },
        ]}
      />

      {activeTab === 'inventory' && (
        <>
          <Row gutter={[12, 12]} style={{ marginBottom: 16 }}>
            <Col xs={12} sm={6} md={4}>
              <Card size="small">
                <Statistic title="管材总量" value={inventorySummary.total} />
              </Card>
            </Col>
            <Col xs={12} sm={6} md={4}>
              <Card size="small">
                <Statistic title="在库" value={inventorySummary.in_stock} valueStyle={{ color: '#3f8600' }} />
              </Card>
            </Col>
            <Col xs={12} sm={6} md={4}>
              <Card size="small">
                <Statistic title="已出库" value={inventorySummary.outbound} valueStyle={{ color: '#1677ff' }} />
              </Card>
            </Col>
            <Col xs={12} sm={6} md={4}>
              <Card size="small">
                <Statistic title="报废" value={inventorySummary.scrapped} valueStyle={{ color: '#cf1322' }} />
              </Card>
            </Col>
          </Row>
          <Table
            columns={inventoryColumns}
            dataSource={inventoryData?.data?.data}
            rowKey={(r) => `${r.pipe_type}-${r.grade}`}
            loading={inventoryLoading}
            locale={{ emptyText: '暂无数据' }}
            scroll={{ x: 560 }}
            pagination={false}
          />
        </>
      )}

      {activeTab === 'orders' && (
        <>
          <Row gutter={[12, 12]} style={{ marginBottom: 16 }}>
            <Col xs={12} sm={6} md={4}>
              <Card size="small">
                <Statistic title="采购订单数" value={orderSummary.po_count} />
              </Card>
            </Col>
            <Col xs={12} sm={6} md={4}>
              <Card size="small">
                <Statistic title="销售订单数" value={orderSummary.so_count} />
              </Card>
            </Col>
            <Col xs={12} sm={6} md={4}>
              <Card size="small">
                <Statistic
                  title="采购金额"
                  prefix="¥"
                  value={orderSummary.po_amount}
                  precision={2}
                />
              </Card>
            </Col>
            <Col xs={12} sm={6} md={4}>
              <Card size="small">
                <Statistic
                  title="销售金额"
                  prefix="¥"
                  value={orderSummary.so_amount}
                  precision={2}
                  valueStyle={{ color: '#3f8600' }}
                />
              </Card>
            </Col>
          </Row>
          <Table
            columns={orderColumns}
            dataSource={orderData?.data?.data}
            rowKey="month"
            loading={orderLoading}
            locale={{ emptyText: '暂无数据' }}
            scroll={{ x: 640 }}
            pagination={false}
          />
        </>
      )}
    </Card>
  );
}
