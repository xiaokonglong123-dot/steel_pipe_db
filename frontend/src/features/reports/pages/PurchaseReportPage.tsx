import React, { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Card, Table, Button, DatePicker, Space, message } from 'antd';
import { DownloadOutlined, ReloadOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import dayjs from 'dayjs';
import { reportApi, OrderReport } from '../api/reportApi';
import { dataIoApi } from '../../data-io/api/dataIoApi';

export default function PurchaseReportPage() {
  const [dateRange, setDateRange] = useState<[dayjs.Dayjs | null, dayjs.Dayjs | null]>([null, null]);

  const filter = {
    date_from: dateRange[0]?.format('YYYY-MM-DD') || undefined,
    date_to: dateRange[1]?.format('YYYY-MM-DD') || undefined,
  };

  const { data, isLoading } = useQuery({
    queryKey: ['report-purchase', filter],
    queryFn: () => reportApi.getPurchaseSummary(filter),
  });

  const handleExport = async () => {
    try {
      const res = await dataIoApi.exportPipes({ ...filter, pipe_type: 'seamless' });
      const blob = new Blob([res.data as BlobPart], { type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' });
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `采购报表_${dayjs().format('YYYYMMDD')}.xlsx`;
      a.click();
      window.URL.revokeObjectURL(url);
      message.success('导出成功');
    } catch {
      message.error('导出失败');
    }
  };

  const columns: ColumnsType<OrderReport> = [
    { title: '月份', dataIndex: 'month', key: 'month' },
    { title: '采购订单数', dataIndex: 'po_count', key: 'po_count', render: (v) => v?.toLocaleString() },
    { title: '采购金额', dataIndex: 'po_amount', key: 'po_amount', render: (v) => `¥${(v ?? 0).toLocaleString()}` },
  ];

  return (
    <Card title="采购报表" styles={{ body: { padding: 16 } }}>
      <Space style={{ marginBottom: 16 }}>
        <DatePicker.RangePicker
          value={dateRange}
          onChange={(dates) => setDateRange(dates as [dayjs.Dayjs | null, dayjs.Dayjs | null])}
        />
        <Button icon={<ReloadOutlined />} onClick={() => setDateRange([null, null])}>重置</Button>
        <Button type="primary" icon={<DownloadOutlined />} onClick={handleExport}>导出</Button>
      </Space>
      <Table
        columns={columns}
        dataSource={data?.data?.data}
        rowKey="month"
        loading={isLoading}
        locale={{ emptyText: '暂无数据' }}
        scroll={{ x: 500 }}
        pagination={false}
      />
    </Card>
  );
}
