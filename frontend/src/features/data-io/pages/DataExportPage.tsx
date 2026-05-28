/**
 * DataExportPage — Export data to Excel/CSV files.
 *
 * Workflow:
 * 1. Select entity type
 * 2. Select export format (xlsx/csv)
 * 3. Click export to download the file
 */
import { useState } from 'react';
import {
  Card,
  Select,
  Button,
  Radio,
  Space,
  Typography,
  message,
} from 'antd';
import { DownloadOutlined } from '@ant-design/icons';
import { dataIoApi, ENTITY_TYPES, type EntityType } from '../api/dataIoApi';

const { Title, Text } = Typography;

export default function DataExportPage() {
  const [entityType, setEntityType] = useState<EntityType>('seamless_pipes');
  const [format, setFormat] = useState<string>('xlsx');
  const [exporting, setExporting] = useState(false);

  const handleExport = async () => {
    setExporting(true);
    try {
      const blob = await dataIoApi.exportData(entityType, format);
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      const ext = format === 'csv' ? 'csv' : 'xlsx';
      a.download = `${entityType}_export.${ext}`;
      a.click();
      window.URL.revokeObjectURL(url);
      message.success('导出成功');
    } catch {
      message.error('导出失败');
    } finally {
      setExporting(false);
    }
  };

  return (
    <div style={{ padding: 24 }}>
      <Title level={3}>数据导出</Title>

      <Card>
        <Space direction="vertical" size="middle" style={{ width: '100%' }}>
          <div>
            <Text strong>选择数据类型：</Text>
            <Select
              value={entityType}
              onChange={setEntityType}
              style={{ width: 240, marginLeft: 8 }}
              options={ENTITY_TYPES.map((t) => ({ value: t.value, label: t.label }))}
            />
          </div>

          <div>
            <Text strong>导出格式：</Text>
            <Radio.Group
              value={format}
              onChange={(e) => setFormat(e.target.value)}
              style={{ marginLeft: 8 }}
            >
              <Radio value="xlsx">Excel (.xlsx)</Radio>
              <Radio value="csv">CSV (.csv)</Radio>
            </Radio.Group>
          </div>

          <Button
            type="primary"
            icon={<DownloadOutlined />}
            onClick={handleExport}
            loading={exporting}
            size="large"
          >
            导出数据
          </Button>
        </Space>
      </Card>
    </div>
  );
}
