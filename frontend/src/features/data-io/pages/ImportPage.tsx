import React, { useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import {
  Card,
  Table,
  Button,
  Space,
  Radio,
  Upload,
  Tabs,
  message,
  Alert,
  Typography,
  Divider,
  Result,
  Tag,
  Row,
  Col,
} from 'antd';
import {
  UploadOutlined,
  DownloadOutlined,
  InboxOutlined,
  ReloadOutlined,
} from '@ant-design/icons';
import type { UploadFile, RcFile } from 'antd/es/upload';
import type { ColumnsType } from 'antd/es/table';
import { dataIoApi, ImportStrategy, ImportPreviewRow, ImportResult, ImportError } from '../api/dataIoApi';

const { Dragger } = Upload;
const { Text } = Typography;

const VALID_EXTENSIONS = ['.xlsx', '.xls', '.csv'];

type ImportStep = 'select' | 'preview' | 'result';

const resultColumns: ColumnsType<ImportError> = [
  { title: '行号', dataIndex: 'row', key: 'row', width: 80 },
  { title: '失败原因', dataIndex: 'reason', key: 'reason', ellipsis: true },
];

export default function ImportPage() {
  const [activeTab, setActiveTab] = useState<'seamless' | 'screen'>('seamless');
  const [strategy, setStrategy] = useState<ImportStrategy>('skip');
  const [fileList, setFileList] = useState<UploadFile[]>([]);
  const [parsedFile, setParsedFile] = useState<RcFile | null>(null);
  const [step, setStep] = useState<ImportStep>('select');
  const [previewData, setPreviewData] = useState<ImportPreviewRow[]>([]);
  const [previewColumns, setPreviewColumns] = useState<string[]>([]);
  const [importResult, setImportResult] = useState<ImportResult | null>(null);

  const isSeamless = activeTab === 'seamless';

  const previewMutation = useMutation({
    mutationFn: (file: RcFile) =>
      isSeamless ? dataIoApi.previewSeamlessPipes(file) : dataIoApi.previewScreenPipes(file),
    onSuccess: (res) => {
      if (res.data.data) {
        setPreviewData(res.data.data.preview_rows);
        setPreviewColumns(res.data.data.columns);
        setStep('preview');
        message.success(`解析成功，共 ${res.data.data.total_rows} 行数据`);
      }
    },
    onError: () => {
      message.error('文件解析失败，请检查文件格式');
    },
  });

  const importMutation = useMutation({
    mutationFn: () => {
      if (!parsedFile) throw new Error('No file selected');
      return isSeamless
        ? dataIoApi.importSeamlessPipes(parsedFile, strategy)
        : dataIoApi.importScreenPipes(parsedFile, strategy);
    },
    onSuccess: (res) => {
      if (res.data.data) {
        setImportResult(res.data.data);
        setStep('result');
        if (res.data.data.failed_rows === 0) {
          message.success(`成功导入 ${res.data.data.success_rows} 条记录`);
        } else {
          message.warning(`导入完成，${res.data.data.success_rows} 成功，${res.data.data.failed_rows} 失败`);
        }
      }
    },
    onError: () => {
      message.error('导入失败，请稍后重试');
    },
  });

  const handleDownloadTemplate = () => {
    const downloadFn = isSeamless
      ? dataIoApi.downloadSeamlessTemplate()
      : dataIoApi.downloadScreenTemplate();

    downloadFn.then((res) => {
      const blob = new Blob([res.data], {
        type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
      });
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${isSeamless ? '无缝钢管' : '筛管'}导入模板.xlsx`;
      a.click();
      window.URL.revokeObjectURL(url);
      message.success('模板下载成功');
    }).catch(() => {
      message.error('模板下载失败');
    });
  };

  const beforeUpload = (file: RcFile): boolean => {
    const ext = '.' + file.name.split('.').pop()?.toLowerCase();
    if (!VALID_EXTENSIONS.includes(ext)) {
      message.error('仅支持 .xlsx / .xls / .csv 格式文件');
      return false;
    }
    setParsedFile(file);
    previewMutation.mutate(file);
    return false; // Prevent auto-upload
  };

  const handleReset = () => {
    setStep('select');
    setFileList([]);
    setParsedFile(null);
    setPreviewData([]);
    setPreviewColumns([]);
    setImportResult(null);
  };

  const handleTabChange = (key: string) => {
    setActiveTab(key as 'seamless' | 'screen');
    handleReset();
  };

  const buildPreviewColumns = (): ColumnsType<ImportPreviewRow> => {
    if (previewColumns.length === 0) {
      return Object.keys(previewData[0] || {}).map((key) => ({
        title: key,
        dataIndex: key,
        key,
        ellipsis: true,
      }));
    }
    return previewColumns.map((col) => ({
      title: col,
      dataIndex: col,
      key: col,
      ellipsis: true,
    }));
  };

  const renderFileSelect = () => (
    <Space direction="vertical" style={{ width: '100%' }} size="large">
      <Alert
        type="info"
        showIcon
        message="导入说明"
        description={
          <ul style={{ margin: 0, paddingLeft: 20 }}>
            <li>支持 .xlsx / .xls / .csv 格式文件</li>
            <li>请先下载导入模板，按照模板格式准备数据</li>
            <li>文件大小限制为 50MB</li>
            <li>首次上传后将自动解析并预览前 5 行数据</li>
          </ul>
        }
      />

      <div>
        <Text strong>导入配置</Text>
        <div style={{ marginTop: 8 }}>
          <Radio.Group
            value={strategy}
            onChange={(e) => setStrategy(e.target.value)}
            optionType="button"
            buttonStyle="solid"
          >
            <Radio.Button value="skip">跳过重复</Radio.Button>
            <Radio.Button value="overwrite">覆盖重复</Radio.Button>
            <Radio.Button value="auto_number">自动编号</Radio.Button>
          </Radio.Group>
        </div>
        <div style={{ marginTop: 8, color: '#888', fontSize: 13 }}>
          {strategy === 'skip' && '遇到已存在的管材编号时跳过该行'}
          {strategy === 'overwrite' && '遇到已存在的管材编号时覆盖原记录'}
          {strategy === 'auto_number' && '遇到重复时自动生成新编号'}
        </div>
      </div>

      <Dragger
        fileList={fileList}
        beforeUpload={beforeUpload}
        onChange={(info) => setFileList(info.fileList)}
        accept={VALID_EXTENSIONS.join(',')}
        maxCount={1}
        disabled={previewMutation.isPending}
      >
        <p className="ant-upload-drag-icon">
          <InboxOutlined />
        </p>
        <p className="ant-upload-text">点击或拖拽文件到此区域上传</p>
        <p className="ant-upload-hint">
          支持 .xlsx / .xls / .csv 格式，单次仅限一个文件
        </p>
      </Dragger>

      {previewMutation.isPending && (
        <Alert type="warning" showIcon message="正在解析文件，请稍候..." />
      )}
    </Space>
  );

  const renderPreview = () => (
    <Space direction="vertical" style={{ width: '100%' }} size="large">
      <Alert
        type="success"
        showIcon
        message="文件解析完成"
        description={`共解析 ${previewData.length} 行数据（预览前 ${Math.min(previewData.length, 5)} 行）`}
      />

      {previewData.length > 0 && (
        <Table
          columns={buildPreviewColumns()}
          dataSource={previewData.slice(0, 5)}
          rowKey={(_, index) => String(index)}
          pagination={false}
          size="small"
          scroll={{ x: 'max-content' }}
          locale={{ emptyText: '暂无预览数据' }}
        />
      )}

      <Row gutter={16}>
        <Col>
          <Button
            type="primary"
            size="large"
            icon={<UploadOutlined />}
            loading={importMutation.isPending}
            onClick={() => importMutation.mutate()}
          >
            确认导入
          </Button>
        </Col>
        <Col>
          <Button
            size="large"
            icon={<ReloadOutlined />}
            onClick={handleReset}
          >
            重新选择
          </Button>
        </Col>
      </Row>
    </Space>
  );

  const renderResult = () => {
    if (!importResult) return null;
    const allSuccess = importResult.failed_rows === 0;

    return (
      <Space direction="vertical" style={{ width: '100%' }} size="large">
        <Result
          status={allSuccess ? 'success' : 'warning'}
          title={allSuccess ? '导入完成' : '导入完成，部分数据失败'}
          subTitle={`共 ${importResult.total_rows} 行，成功 ${importResult.success_rows} 行${
            importResult.failed_rows > 0
              ? `，失败 ${importResult.failed_rows} 行`
              : ''
          }`}
          extra={
            <Space>
              {importResult.failed_rows > 0 && (
                <Button
                  icon={<DownloadOutlined />}
                  onClick={() => message.success('错误报告下载中...')}
                >
                  下载错误报告
                </Button>
              )}
              <Button type="primary" onClick={handleReset}>
                继续导入
              </Button>
            </Space>
          }
        />

        {importResult.errors.length > 0 && (
          <>
            <Divider orientation="left">
              <Tag color="error">失败详情 ({importResult.errors.length})</Tag>
            </Divider>
            <Table
              columns={resultColumns}
              dataSource={importResult.errors}
              rowKey="row"
              pagination={false}
              size="small"
              locale={{ emptyText: '暂无失败记录' }}
            />
          </>
        )}
      </Space>
    );
  };

  return (
    <Card title="数据导入" styles={{ body: { padding: 16 } }}>
      <Tabs
        activeKey={activeTab}
        onChange={handleTabChange}
        items={[
          { key: 'seamless', label: '导入无缝钢管' },
          { key: 'screen', label: '导入筛管' },
        ]}
        tabBarExtraContent={
          <Button
            icon={<DownloadOutlined />}
            onClick={handleDownloadTemplate}
            disabled={step !== 'select'}
          >
            下载导入模板
          </Button>
        }
      />

      <Divider />

      {step === 'select' && renderFileSelect()}
      {step === 'preview' && renderPreview()}
      {step === 'result' && renderResult()}
    </Card>
  );
}
