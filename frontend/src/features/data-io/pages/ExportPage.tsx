import React, { useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import {
  Card,
  Button,
  Space,
  Radio,
  Checkbox,
  DatePicker,
  Select,
  message,
  Typography,
  Divider,
  Row,
  Col,
  Spin,
} from 'antd';
import { DownloadOutlined, FileExcelOutlined } from '@ant-design/icons';
import dayjs from 'dayjs';
import {
  dataIoApi,
  ExportType,
  ExportFormat,
  ExportFilter,
  exportFieldOptions,
} from '../api/dataIoApi';

const { RangePicker } = DatePicker;
const { Title, Text } = Typography;

const exportTypeLabels: Record<ExportType, string> = {
  inventory: '库存报表',
  inbound: '入库明细',
  outbound: '出库明细',
  pipes: '管材列表',
};

const pipeTypeOptions = [
  { label: '无缝钢管', value: 'seamless' },
  { label: '筛管', value: 'screen' },
];

const gradeOptions = [
  { label: 'J55', value: 'J55' },
  { label: 'K55', value: 'K55' },
  { label: 'N80', value: 'N80' },
  { label: 'L80', value: 'L80' },
  { label: 'P110', value: 'P110' },
];

const statusOptions = [
  { label: '在库', value: 'in_stock' },
  { label: '已出库', value: 'outbound' },
  { label: '报废', value: 'scrapped' },
];

export default function ExportPage() {
  const [exportType, setExportType] = useState<ExportType>('inventory');
  const [format, setFormat] = useState<ExportFormat>('xlsx');
  const [selectedFields, setSelectedFields] = useState<string[]>([]);
  const [dateRange, setDateRange] = useState<[dayjs.Dayjs | null, dayjs.Dayjs | null] | null>(null);
  const [filterGrade, setFilterGrade] = useState<string | undefined>();
  const [filterPipeType, setFilterPipeType] = useState<string | undefined>();
  const [filterStatus, setFilterStatus] = useState<string | undefined>();

  const currentFieldOptions = exportFieldOptions[exportType];

  const handleExportTypeChange = (type: ExportType) => {
    setExportType(type);
    setSelectedFields([]);
    setFilterGrade(undefined);
    setFilterPipeType(undefined);
    setFilterStatus(undefined);
    setDateRange(null);
  };

  const handleSelectAll = () => {
    if (selectedFields.length === currentFieldOptions.length) {
      setSelectedFields([]);
    } else {
      setSelectedFields(currentFieldOptions.map((opt) => opt.value));
    }
  };

  const exportMutation = useMutation({
    mutationFn: () => {
      const filter: ExportFilter = {
        format,
        fields: selectedFields.length > 0 ? selectedFields : undefined,
      };

      if (dateRange?.[0]) filter.date_from = dateRange[0].format('YYYY-MM-DD');
      if (dateRange?.[1]) filter.date_to = dateRange[1].format('YYYY-MM-DD');
      if (filterGrade) filter.grade = filterGrade;
      if (filterStatus) filter.status = filterStatus;

      let apiCall;
      switch (exportType) {
        case 'inventory':
          apiCall = dataIoApi.exportInventory(filter);
          break;
        case 'inbound':
          apiCall = dataIoApi.exportInbound(filter);
          break;
        case 'outbound':
          apiCall = dataIoApi.exportOutbound(filter);
          break;
        case 'pipes':
          apiCall = dataIoApi.exportPipes({
            ...filter,
            pipe_type: (filterPipeType as 'seamless' | 'screen') || 'seamless',
          });
          break;
        default:
          throw new Error(`Unknown export type: ${exportType}`);
      }
      return apiCall;
    },
    onSuccess: (res) => {
      const contentType = String(res.headers['content-type'] || '');
      const isCsv = format === 'csv' || contentType.includes('csv');
      const blob = new Blob([res.data], {
        type: isCsv
          ? 'text/csv;charset=utf-8'
          : 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
      });
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      const now = dayjs().format('YYYYMMDD_HHmmss');
      a.download = `${exportTypeLabels[exportType]}_${now}.${format === 'csv' ? 'csv' : 'xlsx'}`;
      a.click();
      window.URL.revokeObjectURL(url);
      message.success('导出成功');
    },
    onError: () => {
      message.error('导出失败，请稍后重试');
    },
  });

  const renderFilters = () => {
    const showDateRange = exportType !== 'pipes';
    const showGrade = exportType === 'inventory' || exportType === 'pipes';
    const showStatus = exportType === 'inventory' || exportType === 'pipes';
    const showPipeType = exportType === 'pipes';

    return (
      <Row gutter={[16, 16]}>
        {showDateRange && (
          <Col xs={24} sm={12} md={8}>
            <div>
              <Text style={{ display: 'block', marginBottom: 4 }}>日期范围</Text>
              <RangePicker
                style={{ width: '100%' }}
                value={dateRange}
                onChange={(dates) => setDateRange(dates as [dayjs.Dayjs | null, dayjs.Dayjs | null] | null)}
                allowClear
              />
            </div>
          </Col>
        )}
        {showPipeType && (
          <Col xs={12} sm={6} md={4}>
            <div>
              <Text style={{ display: 'block', marginBottom: 4 }}>管材类型</Text>
              <Select
                placeholder="全部"
                style={{ width: '100%' }}
                value={filterPipeType}
                onChange={setFilterPipeType}
                allowClear
                options={pipeTypeOptions}
              />
            </div>
          </Col>
        )}
        {showGrade && (
          <Col xs={12} sm={6} md={4}>
            <div>
              <Text style={{ display: 'block', marginBottom: 4 }}>钢级</Text>
              <Select
                placeholder="全部"
                style={{ width: '100%' }}
                value={filterGrade}
                onChange={setFilterGrade}
                allowClear
                options={gradeOptions}
              />
            </div>
          </Col>
        )}
        {showStatus && (
          <Col xs={12} sm={6} md={4}>
            <div>
              <Text style={{ display: 'block', marginBottom: 4 }}>状态</Text>
              <Select
                placeholder="全部"
                style={{ width: '100%' }}
                value={filterStatus}
                onChange={setFilterStatus}
                allowClear
                options={statusOptions}
              />
            </div>
          </Col>
        )}
      </Row>
    );
  };

  return (
    <Card title="数据导出" styles={{ body: { padding: 16 } }}>
      <Space direction="vertical" style={{ width: '100%' }} size="large">
        <div>
          <Title level={5}>导出类型</Title>
          <Radio.Group
            value={exportType}
            onChange={(e) => handleExportTypeChange(e.target.value)}
            optionType="button"
            buttonStyle="solid"
          >
            {Object.entries(exportTypeLabels).map(([key, label]) => (
              <Radio.Button key={key} value={key}>
                {label}
              </Radio.Button>
            ))}
          </Radio.Group>
        </div>

        <Divider style={{ margin: '12px 0' }} />

        <div>
          <Title level={5}>筛选条件</Title>
          {renderFilters()}
        </div>

        <Divider style={{ margin: '12px 0' }} />

        <div>
          <Title level={5}>导出字段</Title>
          <div style={{ marginBottom: 8 }}>
            <Button
              type="link"
              size="small"
              onClick={handleSelectAll}
              style={{ padding: 0 }}
            >
              {selectedFields.length === currentFieldOptions.length &&
              currentFieldOptions.length > 0
                ? '取消全选'
                : '全选'}
            </Button>
            <Text style={{ marginLeft: 8, color: '#888', fontSize: 13 }}>
              {selectedFields.length > 0
                ? `已选 ${selectedFields.length} 项`
                : '未选择将导出全部字段'}
            </Text>
          </div>
          <Checkbox.Group
            value={selectedFields}
            onChange={(values) => setSelectedFields(values as string[])}
          >
            <Row gutter={[16, 8]}>
              {currentFieldOptions.map((opt) => (
                <Col key={opt.value} xs={12} sm={8} md={6} lg={4}>
                  <Checkbox value={opt.value}>{opt.label}</Checkbox>
                </Col>
              ))}
            </Row>
          </Checkbox.Group>
        </div>

        <Divider style={{ margin: '12px 0' }} />

        <div>
          <Title level={5}>文件格式</Title>
          <Radio.Group
            value={format}
            onChange={(e) => setFormat(e.target.value)}
            optionType="button"
            buttonStyle="solid"
          >
            <Radio.Button value="xlsx">
              <FileExcelOutlined /> Excel (.xlsx)
            </Radio.Button>
            <Radio.Button value="csv">CSV (.csv)</Radio.Button>
          </Radio.Group>
        </div>

        <Divider style={{ margin: '12px 0' }} />

        <Spin spinning={exportMutation.isPending} tip="正在导出...">
          <Button
            type="primary"
            size="large"
            icon={<DownloadOutlined />}
            onClick={() => exportMutation.mutate()}
            loading={exportMutation.isPending}
          >
            导出数据
          </Button>
        </Spin>
      </Space>
    </Card>
  );
}
