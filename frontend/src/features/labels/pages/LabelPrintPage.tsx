import React, { useState, useMemo } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  Card,
  Row,
  Col,
  Select,
  InputNumber,
  Button,
  Table,
  Tag,
  message,
  Typography,
  Space,
  Empty,
  Spin,
  Descriptions,
} from 'antd';
import { PrinterOutlined, HistoryOutlined, FileTextOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import { labelApi, LabelTemplate, PrintJob } from '../api/labelApi';
import { pipeApi, SeamlessPipe, ScreenPipe } from '../../pipes/api/pipeApi';

const { Text } = Typography;

const statusConfig: Record<string, { color: string; label: string }> = {
  in_stock: { color: 'green', label: '在库' },
  outbound: { color: 'blue', label: '已出库' },
  scrapped: { color: 'red', label: '报废' },
};

const printStatusConfig: Record<string, { color: string; label: string }> = {
  pending: { color: 'orange', label: '待打印' },
  processing: { color: 'blue', label: '打印中' },
  completed: { color: 'green', label: '已完成' },
  failed: { color: 'red', label: '失败' },
};

function getPipeLabel(pipe: SeamlessPipe | ScreenPipe): string {
  return `${pipe.pipe_number} | ${pipe.grade} ${pipe.od}×${pipe.wt} | ${pipe.length.toFixed(2)}m`;
}

export default function LabelPrintPage() {
  const queryClient = useQueryClient();

  const [selectedPipeIds, setSelectedPipeIds] = useState<string[]>([]);
  const [selectedTemplateId, setSelectedTemplateId] = useState<string | undefined>();
  const [copies, setCopies] = useState<number>(1);
  const [pipeSearchText, setPipeSearchText] = useState('');
  const [pipeTab, setPipeTab] = useState<'seamless' | 'screen'>('seamless');

  const {
    data: templatesData,
    isLoading: templatesLoading,
  } = useQuery({
    queryKey: ['label-templates'],
    queryFn: () => labelApi.listTemplates(),
  });

  const seamlessQuery = useQuery({
    queryKey: ['seamless-pipes', { search: pipeSearchText || undefined, page_size: 50 }],
    queryFn: () => pipeApi.listSeamless({ search: pipeSearchText || undefined, page_size: 50 }),
    enabled: pipeTab === 'seamless',
  });

  const screenQuery = useQuery({
    queryKey: ['screen-pipes', { search: pipeSearchText || undefined, page_size: 50 }],
    queryFn: () => pipeApi.listScreen({ search: pipeSearchText || undefined, page_size: 50 }),
    enabled: pipeTab === 'screen',
  });

  const selectedPipes = useMemo(() => {
    const allPipes = [
      ...(seamlessQuery.data?.data?.data || []),
      ...(screenQuery.data?.data?.data || []),
    ];
    return allPipes.filter((p) => selectedPipeIds.includes(p.id));
  }, [selectedPipeIds, seamlessQuery.data, screenQuery.data]);

  const printMutation = useMutation({
    mutationFn: () =>
      labelApi.printLabels({
        pipe_ids: selectedPipeIds,
        template_id: selectedTemplateId!,
        copies,
      }),
    onSuccess: () => {
      message.success('打印任务已提交');
      queryClient.invalidateQueries({ queryKey: ['print-jobs'] });
    },
    onError: () => {
      message.error('提交打印任务失败');
    },
  });

  const printJobsQuery = useQuery({
    queryKey: ['print-jobs'],
    queryFn: () => labelApi.listPrintJobs({ page: 1, page_size: 10 }),
  });

  const templateOptions = useMemo(() => {
    if (!templatesData?.data?.data) return [];
    return templatesData.data.data.map((t: LabelTemplate) => ({
      label: `${t.name} (${t.width_mm}×${t.height_mm}mm)${t.is_default ? ' (默认)' : ''}`,
      value: t.id,
    }));
  }, [templatesData]);

  const pipeOptions = useMemo(() => {
    const currentData = pipeTab === 'seamless' ? seamlessQuery.data : screenQuery.data;
    if (!currentData?.data?.data) return [];
    return currentData.data.data.map((p: SeamlessPipe | ScreenPipe) => ({
      label: getPipeLabel(p),
      value: p.id,
    }));
  }, [pipeTab, seamlessQuery.data, screenQuery.data]);

  React.useEffect(() => {
    if (!selectedTemplateId && templatesData?.data?.data) {
      const defaultTmpl = templatesData.data.data.find((t: LabelTemplate) => t.is_default);
      if (defaultTmpl) {
        setSelectedTemplateId(defaultTmpl.id);
      } else if (templatesData.data.data.length > 0) {
        setSelectedTemplateId(templatesData.data.data[0].id);
      }
    }
  }, [templatesData, selectedTemplateId]);

  const selectedTemplate = useMemo(() => {
    if (!selectedTemplateId || !templatesData?.data?.data) return null;
    return templatesData.data.data.find((t: LabelTemplate) => t.id === selectedTemplateId) || null;
  }, [selectedTemplateId, templatesData]);

  const isPrintDisabled = !selectedTemplateId || selectedPipeIds.length === 0 || printMutation.isPending;

  const printJobsColumns: ColumnsType<PrintJob> = [
    {
      title: '任务编号',
      dataIndex: 'job_id',
      key: 'job_id',
      width: 160,
      ellipsis: true,
    },
    {
      title: '模板',
      dataIndex: 'template_name',
      key: 'template_name',
      width: 140,
    },
    {
      title: '管材数量',
      dataIndex: 'pipe_count',
      key: 'pipe_count',
      width: 100,
      align: 'center',
    },
    {
      title: '份数',
      dataIndex: 'copies',
      key: 'copies',
      width: 80,
      align: 'center',
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      width: 100,
      render: (s: string) => {
        const cfg = printStatusConfig[s] || { color: 'default', label: s };
        return <Tag color={cfg.color}>{cfg.label}</Tag>;
      },
    },
    {
      title: '时间',
      dataIndex: 'created_at',
      key: 'created_at',
      width: 180,
    },
  ];

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
      <Card
        title={
          <Space>
            <PrinterOutlined />
            <span>标签打印</span>
          </Space>
        }
        styles={{ body: { padding: 24 } }}
      >
        <Row gutter={[24, 24]}>
          <Col xs={24} lg={12}>
            <div style={{ marginBottom: 12 }}>
              <Text strong style={{ fontSize: 14, display: 'block', marginBottom: 8 }}>
                选择管材
              </Text>
              <Space style={{ marginBottom: 8 }}>
                <Button
                  size="small"
                  type={pipeTab === 'seamless' ? 'primary' : 'default'}
                  onClick={() => setPipeTab('seamless')}
                >
                  无缝钢管
                </Button>
                <Button
                  size="small"
                  type={pipeTab === 'screen' ? 'primary' : 'default'}
                  onClick={() => setPipeTab('screen')}
                >
                  筛管
                </Button>
              </Space>
              <Select
                mode="multiple"
                style={{ width: '100%' }}
                placeholder="搜索并选择管材"
                value={selectedPipeIds}
                onChange={setSelectedPipeIds}
                options={pipeOptions}
                loading={pipeTab === 'seamless' ? seamlessQuery.isLoading : screenQuery.isLoading}
                showSearch
                filterOption={false}
                onSearch={setPipeSearchText}
                notFoundContent={
                  pipeTab === 'seamless' && seamlessQuery.isLoading ? (
                    <Spin size="small" />
                  ) : pipeTab === 'screen' && screenQuery.isLoading ? (
                    <Spin size="small" />
                  ) : (
                    <Empty description="未找到匹配管材" />
                  )
                }
                maxTagCount={5}
                maxTagTextLength={30}
              />
            </div>

            {selectedPipes.length > 0 && (
              <div style={{ marginTop: 8 }}>
                <Text strong style={{ fontSize: 14, display: 'block', marginBottom: 8 }}>
                  已选管材 ({selectedPipes.length})
                </Text>
                <div
                  style={{
                    display: 'flex',
                    flexDirection: 'column',
                    gap: 6,
                    maxHeight: 320,
                    overflowY: 'auto',
                    paddingRight: 4,
                  }}
                >
                  {selectedPipes.map((pipe) => {
                    const cfg = statusConfig[pipe.status] || { color: 'default', label: pipe.status };
                    return (
                      <Card
                        key={pipe.id}
                        size="small"
                        styles={{
                          body: {
                            padding: '8px 12px',
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'space-between',
                            flexWrap: 'wrap',
                            gap: 8,
                          },
                        }}
                        style={{
                          borderLeft: '3px solid #1B3A5C',
                          background: '#f8fafc',
                        }}
                      >
                        <Space size={12}>
                          <FileTextOutlined style={{ color: '#1B3A5C', fontSize: 16 }} />
                          <div>
                            <Text strong style={{ fontFamily: 'monospace' }}>
                              {pipe.pipe_number}
                            </Text>
                            <div>
                              <Text type="secondary" style={{ fontSize: 12 }}>
                                {pipe.grade} | {pipe.od}×{pipe.wt} | {pipe.length.toFixed(2)}m
                              </Text>
                            </div>
                          </div>
                        </Space>
                        <Space size={8}>
                          <Tag color={cfg.color}>{cfg.label}</Tag>
                          {pipe.location && (
                            <Text type="secondary" style={{ fontSize: 12 }}>
                              {pipe.location}
                            </Text>
                          )}
                        </Space>
                      </Card>
                    );
                  })}
                </div>
              </div>
            )}
          </Col>

          <Col xs={24} lg={12}>
            <div style={{ marginBottom: 20 }}>
              <Text strong style={{ fontSize: 14, display: 'block', marginBottom: 8 }}>
                标签模板
              </Text>
              <Select
                style={{ width: '100%' }}
                placeholder="选择标签模板"
                value={selectedTemplateId}
                onChange={setSelectedTemplateId}
                options={templateOptions}
                loading={templatesLoading}
                allowClear
              />
              {selectedTemplate && (
                <Descriptions size="small" style={{ marginTop: 12 }} column={2}>
                  <Descriptions.Item label="尺寸">
                    {selectedTemplate.width_mm} × {selectedTemplate.height_mm} mm
                  </Descriptions.Item>
                  <Descriptions.Item label="字段数">
                    {selectedTemplate.fields?.length || 0}
                  </Descriptions.Item>
                  <Descriptions.Item label="默认">
                    {selectedTemplate.is_default ? '是' : '否'}
                  </Descriptions.Item>
                </Descriptions>
              )}
            </div>

            <div style={{ marginBottom: 20 }}>
              <Text strong style={{ fontSize: 14, display: 'block', marginBottom: 8 }}>
                打印份数
              </Text>
              <InputNumber
                min={1}
                max={10}
                value={copies}
                onChange={(v) => setCopies(v || 1)}
                style={{ width: 120 }}
                addonAfter="份"
              />
            </div>

            <Button
              type="primary"
              icon={<PrinterOutlined />}
              size="large"
              disabled={isPrintDisabled}
              loading={printMutation.isPending}
              onClick={() => printMutation.mutate()}
              style={{ minWidth: 180 }}
            >
              开始打印
            </Button>

            {selectedPipeIds.length === 0 && selectedTemplateId && (
              <div style={{ marginTop: 12 }}>
                <Text type="secondary" style={{ fontSize: 12 }}>
                  请先选择需要打印标签的管材
                </Text>
              </div>
            )}
            {!selectedTemplateId && (
              <div style={{ marginTop: 12 }}>
                <Text type="secondary" style={{ fontSize: 12 }}>
                  请先选择标签模板
                </Text>
              </div>
            )}
          </Col>
        </Row>
      </Card>

      <Card
        size="small"
        title={
          <Space>
            <FileTextOutlined />
            <span>标签预览</span>
          </Space>
        }
        styles={{ body: { padding: 16 } }}
      >
        {selectedPipes.length > 0 && selectedTemplate ? (
          <div
            style={{
              display: 'flex',
              gap: 12,
              flexWrap: 'wrap',
            }}
          >
            {selectedPipes.slice(0, 5).map((pipe) => (
              <div
                key={pipe.id}
                style={{
                  width: 280,
                  border: '1px solid #d9d9d9',
                  borderRadius: 4,
                  padding: 12,
                  background: '#fff',
                  fontFamily: 'monospace',
                  fontSize: 11,
                  position: 'relative',
                }}
              >
                <div
                  style={{
                    borderBottom: '2px solid #1B3A5C',
                    paddingBottom: 6,
                    marginBottom: 8,
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                  }}
                >
                  <strong style={{ fontSize: 13, color: '#1B3A5C' }}>
                    {selectedTemplate.name}
                  </strong>
                  <Text style={{ fontSize: 10, color: '#999' }}>
                    {selectedTemplate.width_mm}×{selectedTemplate.height_mm}
                  </Text>
                </div>

                <div style={{ display: 'flex', flexDirection: 'column', gap: 3 }}>
                  <div>
                    <Text type="secondary" style={{ fontSize: 10 }}>
                      编号:{' '}
                    </Text>
                    <Text strong>{pipe.pipe_number}</Text>
                  </div>
                  <div>
                    <Text type="secondary" style={{ fontSize: 10 }}>
                      钢级/规格:{' '}
                    </Text>
                    <Text>
                      {pipe.grade} {pipe.od}×{pipe.wt}
                    </Text>
                  </div>
                  <div>
                    <Text type="secondary" style={{ fontSize: 10 }}>
                      长度:{' '}
                    </Text>
                    <Text>{pipe.length.toFixed(2)}m</Text>
                  </div>
                  <div>
                    <Text type="secondary" style={{ fontSize: 10 }}>
                      重量:{' '}
                    </Text>
                    <Text>{pipe.weight.toFixed(1)}kg</Text>
                  </div>
                  {'heat_number' in pipe && pipe.heat_number && (
                    <div>
                      <Text type="secondary" style={{ fontSize: 10 }}>
                        炉批号:{' '}
                      </Text>
                      <Text>{pipe.heat_number}</Text>
                    </div>
                  )}
                  {pipe.location && (
                    <div>
                      <Text type="secondary" style={{ fontSize: 10 }}>
                        库位:{' '}
                      </Text>
                      <Text>{pipe.location}</Text>
                    </div>
                  )}
                </div>

                {copies > 1 && (
                  <div
                    style={{
                      position: 'absolute',
                      top: -6,
                      right: -6,
                      background: '#1B3A5C',
                      color: '#fff',
                      borderRadius: 10,
                      padding: '0 6px',
                      fontSize: 10,
                      lineHeight: '18px',
                    }}
                  >
                    ×{copies}
                  </div>
                )}
              </div>
            ))}
            {selectedPipes.length > 5 && (
              <div
                style={{
                  width: 280,
                  border: '1px dashed #d9d9d9',
                  borderRadius: 4,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  color: '#999',
                  fontSize: 13,
                  minHeight: 140,
                }}
              >
                +{selectedPipes.length - 5} 更多
              </div>
            )}
          </div>
        ) : (
          <Empty
            description={
              selectedPipeIds.length === 0
                ? '请先选择管材和模板'
                : '请选择标签模板以预览'
            }
            image={Empty.PRESENTED_IMAGE_SIMPLE}
          />
        )}
      </Card>

      <Card
        size="small"
        title={
          <Space>
            <HistoryOutlined />
            <span>历史打印</span>
          </Space>
        }
        styles={{ body: { padding: 0 } }}
      >
        <Table<PrintJob>
          columns={printJobsColumns}
          dataSource={printJobsQuery.data?.data?.data}
          rowKey="id"
          loading={printJobsQuery.isLoading}
          locale={{ emptyText: '暂无打印记录' }}
          scroll={{ x: 760 }}
          pagination={false}
          size="small"
        />
      </Card>
    </div>
  );
}
