import React, { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Card, Table, Button, DatePicker, Space, message } from 'antd';
import { DownloadOutlined, ReloadOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import dayjs from 'dayjs';
import { reportApi, InventoryReport } from '../api/reportApi';
import { dataIoApi } from '../../data-io/api/dataIoApi';

export default function OutboundReportPage() {
  const [dateRange, setDateRange] = useState<[dayjs.Dayjs | null, dayjs.Dayjs | null]>([null, null]);

  const filter = {
    date_from: dateRange[0]?.format('YYYY-MM-DD') || undefined,
    date_to: dateRange[1]?.format('YYYY-MM-DD') || undefined,
  };

  const { data, isLoading } = useQuery({
    queryKey: ['report-outbound', filter],
    queryFn: () => reportApi.getOutboundSummary(filter),
  });

  const handleExport = async () => {
    try {
      const res = await dataIoApi.exportOutbound(filter);
      const blob = new Blob([res.data as BlobPart], { type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' });
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `出库报表_${dayjs().format('YYYYMMDD')}.xlsx`;
      a.click();
      window.URL.revokeObjectURL(url);
      message.success('导出成功');
    } catch {
      message.error('导出失败');
    }
  };

  const columns: ColumnsType<InventoryReport> = [
    { title: '管材类型', dataIndex: 'pipe_type', key: 'pipe_type' },
    { title: '钢级', dataIndex: 'grade', key: 'grade' },
    { title: '出库数量', dataIndex: 'outbound', key: 'outbound', render: (v) => v?.toLocaleString() },
    { title: '报废数量', dataIndex: 'scrapped', key: 'scrapped', render: (v) => v?.toLocaleString() },
  ];

  return (
    <Card title="出库报表" styles={{ body: { padding: 16 } }}>
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
        rowKey={(r) => `${r.pipe_type}-${r.grade}`}
        loading={isLoading}
        locale={{ emptyText: '暂无数据' }}
        scroll={{ x: 600 }}
        pagination={false}
      />
    </Card>
  );
}
