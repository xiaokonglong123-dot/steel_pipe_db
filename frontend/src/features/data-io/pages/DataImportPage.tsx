/**
 * DataImportPage — Upload Excel/CSV files for bulk data import.
 *
 * Workflow:
 * 1. Select entity type (seamless_pipes, screen_pipes, etc.)
 * 2. Optionally download the import template first
 * 3. Upload the filled file
 * 4. Review import results (success count, failure count, error details)
 */
import { useState } from 'react';
import {
  Card,
  Select,
  Upload,
  Button,
  Table,
  Tag,
  Space,
  Typography,
  message,
} from 'antd';
import {
  UploadOutlined,
  DownloadOutlined,
  InboxOutlined,
} from '@ant-design/icons';
import type { UploadFile } from 'antd/es/upload/interface';
import { dataIoApi, ENTITY_TYPES, type EntityType } from '../api/dataIoApi';

const { Title, Text } = Typography;
const { Dragger } = Upload;

export default function DataImportPage() {
  const [entityType, setEntityType] = useState<EntityType>('seamless_pipes');
  const [importing, setImporting] = useState(false);
  const [downloadingTemplate, setDownloadingTemplate] = useState(false);
  const [fileList, setFileList] = useState<UploadFile[]>([]);
  const [result, setResult] = useState<{
    imported_count: number;
    failed_count: number;
    errors: string[];
  } | null>(null);

  const handleDownloadTemplate = async () => {
    setDownloadingTemplate(true);
    try {
      const blob = await dataIoApi.getTemplate(entityType);
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${entityType}_template.xlsx`;
      a.click();
      window.URL.revokeObjectURL(url);
      message.success('模板下载成功');
    } catch {
      message.error('模板下载失败');
    } finally {
      setDownloadingTemplate(false);
    }
  };

  const handleImport = async () => {
    if (fileList.length === 0) {
      message.warning('请先选择要导入的文件');
      return;
    }
    const file = fileList[0].originFileObj;
    if (!file) return;

    setImporting(true);
    setResult(null);
    try {
      const res = await dataIoApi.importData(entityType, file);
      setResult(res);
      if (res.failed_count === 0) {
        message.success(`导入成功！共导入 ${res.imported_count} 条记录`);
      } else {
        message.warning(
          `导入完成：成功 ${res.imported_count} 条，失败 ${res.failed_count} 条`,
        );
      }
      setFileList([]);
    } catch {
      message.error('导入失败，请检查文件格式');
    } finally {
      setImporting(false);
    }
  };

  const errorColumns = [
    {
      title: '行号 / 错误信息',
      dataIndex: 'error',
      key: 'error',
      render: (text: string) => <Text type="danger">{text}</Text>,
    },
  ];

  return (
    <div style={{ padding: 24 }}>
      <Title level={3}>数据导入</Title>

      <Card style={{ marginBottom: 16 }}>
        <Space direction="vertical" size="middle" style={{ width: '100%' }}>
          <div>
            <Text strong>选择数据类型：</Text>
            <Select
              value={entityType}
              onChange={(v) => {
                setEntityType(v);
                setResult(null);
                setFileList([]);
              }}
              style={{ width: 240, marginLeft: 8 }}
              options={ENTITY_TYPES.map((t) => ({ value: t.value, label: t.label }))}
            />
          </div>

          <Button
            icon={<DownloadOutlined />}
            onClick={handleDownloadTemplate}
            loading={downloadingTemplate}
          >
            下载导入模板
          </Button>

          <Dragger
            accept=".xlsx,.xls,.csv"
            maxCount={1}
            fileList={fileList}
            onChange={({ fileList: fl }) => setFileList(fl)}
            beforeUpload={() => false}
          >
            <p className="ant-upload-drag-icon">
              <InboxOutlined />
            </p>
            <p className="ant-upload-text">点击或拖拽文件到此区域上传</p>
            <p className="ant-upload-hint">
              支持 .xlsx、.xls、.csv 格式
            </p>
          </Dragger>

          <Button
            type="primary"
            icon={<UploadOutlined />}
            onClick={handleImport}
            loading={importing}
            disabled={fileList.length === 0}
          >
            开始导入
          </Button>
        </Space>
      </Card>

      {result && (
        <Card title="导入结果">
          <Space size="large" style={{ marginBottom: 16 }}>
            <span>
              成功：<Tag color="green">{result.imported_count}</Tag> 条
            </span>
            <span>
              失败：<Tag color="red">{result.failed_count}</Tag> 条
            </span>
          </Space>
          {result.errors.length > 0 && (
            <Table
              columns={errorColumns}
              dataSource={result.errors.map((e, i) => ({ key: i, error: e }))}
              pagination={false}
              size="small"
              scroll={{ y: 300 }}
            />
          )}
        </Card>
      )}
    </div>
  );
}
