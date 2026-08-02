// 入库单新增/编辑表单页 — 使用 PageLayout + 共享常量
import { useEffect, useState } from 'react';
import {
  Form,
  Input,
  Select,
  InputNumber,
  Button,
  Space,
  message,
  Table,
  Modal,
  Popconfirm,
} from 'antd';
import { PlusOutlined, DeleteOutlined, SearchOutlined } from '@ant-design/icons';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { PageLayout } from '@/shared/components/PageLayout';
import { INBOUND_TYPES, DETAILED_PIPE_TYPES } from '@/shared/constants';
import { parsePipeIds } from '@/shared/utils/pipeIds';
import { useCreateInbound, useUpdateInbound, useInboundRecord } from '../hooks/useInventory';
import { pipeSearchApi } from '../api/inventoryApi';
import type { PipeSearchResult, CreateInboundData, InboundItem } from '../api/inventoryApi';

interface PipeFormRow {
  pipe_type?: string;
  pipe_id?: number;
  pipe_number?: string;
  grade?: string;
  od?: number;
  wt?: number;
}

function pipeRowKey(pipeType: string | undefined, pipeId: number | undefined): string | null {
  return pipeType && typeof pipeId === 'number' ? `${pipeType}:${pipeId}` : null;
}

export default function InboundFormPage() {
  const { t } = useTranslation();
  const { id } = useParams<{ id?: string }>();
  const navigate = useNavigate();
  const [form] = Form.useForm();

  const isEdit = !!id;
  const orderId = isEdit ? Number(id) : 0;

  const { data: inboundRecord, isLoading: loadingRecord } = useInboundRecord(orderId);
  const createMutation = useCreateInbound();
  const updateMutation = useUpdateInbound(orderId);

  const [searchModalOpen, setSearchModalOpen] = useState(false);
  const [searchText, setSearchText] = useState('');
  const [searchResults, setSearchResults] = useState<PipeSearchResult[]>([]);
  const [searchLoading, setSearchLoading] = useState(false);
  const [batchModalOpen, setBatchModalOpen] = useState(false);
  const [batchPipeType, setBatchPipeType] = useState('casing');
  const [batchPipeIds, setBatchPipeIds] = useState('');

  useEffect(() => {
    if (isEdit && inboundRecord) {
      form.setFieldsValue({
        inbound_type: inboundRecord.record.inbound_type,
        order_id: inboundRecord.record.order_id,
        supplier_id: inboundRecord.record.supplier_id,
        notes: inboundRecord.record.notes,
        pipes: inboundRecord.items.map((item: InboundItem) => ({
          pipe_type: item.pipe_type,
          pipe_id: item.pipe_id,
        })),
      });
    }
  }, [isEdit, inboundRecord, form]);

  const handlePipeSearch = async () => {
    setSearchLoading(true);
    try {
      const results = await pipeSearchApi.search({ q: searchText || undefined, limit: 50 });
      setSearchResults(results);
    } catch (err) {
      console.error('pipe search failed', err);
      message.error(t('common.operate_failed'));
    } finally {
      setSearchLoading(false);
    }
  };

  const handleSelectPipe = (pipe: PipeSearchResult) => {
    const pipes = (form.getFieldValue('pipes') || []) as PipeFormRow[];
    const nextKey = pipeRowKey(pipe.pipe_type, pipe.id);
    const exists = pipes.some((p) => pipeRowKey(p.pipe_type, p.pipe_id) === nextKey);
    if (exists) {
      message.warning(t('inbound.pipe_already_added', 'This pipe has already been added to the list'));
      return;
    }
    form.setFieldsValue({
      pipes: [...pipes, { pipe_type: pipe.pipe_type, pipe_id: pipe.id, pipe_number: pipe.pipe_number, grade: pipe.grade, od: pipe.od, wt: pipe.wt }],
    });
    setSearchModalOpen(false);
  };

  const handleBatchAddPipes = () => {
    const ids = parsePipeIds(batchPipeIds);
    if (ids.length === 0) {
      message.error(t('common.required'));
      return;
    }

    const currentPipes = (form.getFieldValue('pipes') || []) as PipeFormRow[];
    const existingKeys = new Set(
      currentPipes
        .map((pipe) => pipeRowKey(pipe.pipe_type, pipe.pipe_id))
        .filter((key): key is string => key !== null),
    );
    const rowsToAdd = ids
      .filter((pipeId) => !existingKeys.has(pipeRowKey(batchPipeType, pipeId) ?? ''))
      .map((pipeId) => ({ pipe_type: batchPipeType, pipe_id: pipeId }));

    if (rowsToAdd.length === 0) {
      message.warning(t('inbound.pipe_already_added', 'This pipe has already been added to the list'));
      return;
    }

    form.setFieldsValue({ pipes: [...currentPipes, ...rowsToAdd] });
    setBatchPipeIds('');
    setBatchModalOpen(false);
    message.success(t('common.operate_success'));
  };

  const handleSubmit = async (values: Record<string, unknown>) => {
    try {
      const pipes = Array.isArray(values.pipes)
        ? values.pipes.map((p: unknown) => {
            const item = p as Record<string, unknown>;
            return { pipe_type: String(item.pipe_type ?? ''), pipe_id: Number(item.pipe_id) };
          })
        : [];

      const cleanValues: CreateInboundData = {
        inbound_type: String(values.inbound_type ?? ''),
        order_id: values.order_id != null ? Number(values.order_id) : undefined,
        supplier_id: values.supplier_id != null ? Number(values.supplier_id) : undefined,
        notes: values.notes != null ? String(values.notes) : undefined,
        pipes: isEdit ? [] : pipes,
      };

      if (!cleanValues.inbound_type || cleanValues.pipes.length === 0) {
        message.error(t('common.required'));
        return;
      }
      if (cleanValues.pipes.some((p) => !p.pipe_type || !p.pipe_id)) {
        message.error(t('common.required'));
        return;
      }

      if (isEdit) {
        await updateMutation.mutateAsync(cleanValues);
      } else {
        await createMutation.mutateAsync(cleanValues);
      }
      message.success(t('common.operate_success'));
      navigate('/inventory/inbound');
    } catch (err) {
      console.error('submit inbound failed', err);
      message.error(t('common.operate_failed'));
    }
  };

  if (isEdit && loadingRecord) {
    return <div>{t('common.loading')}</div>;
  }

  const searchColumns = [
    {
      title: t('inbound.pipe_id_placeholder'),
      dataIndex: 'id',
      key: 'id',
      width: 80,
    },
    {
      title: t('stock.pipe_type'),
      dataIndex: 'pipe_type',
      key: 'pipe_type',
      width: 100,
      render: (val: string) => t(`pipe_type.${val}`, val),
    },
    {
      title: t('pipes.pipe_number'),
      dataIndex: 'pipe_number',
      key: 'pipe_number',
      width: 120,
    },
    {
      title: t('pipes.grade'),
      dataIndex: 'grade',
      key: 'grade',
      width: 80,
    },
    {
      title: t('pipes.od'),
      dataIndex: 'od',
      key: 'od',
      width: 80,
      render: (val: number) => (val != null ? val : '-'),
    },
    {
      title: t('pipes.wt'),
      dataIndex: 'wt',
      key: 'wt',
      width: 80,
      render: (val: number) => (val != null ? val : '-'),
    },
    {
      title: t('common.actions'),
      key: 'actions',
      width: 80,
      render: (_: unknown, record: PipeSearchResult) => (
        <Button type="link" onClick={() => handleSelectPipe(record)}>
          {t('common.select')}
        </Button>
      ),
    },
  ];

  const itemColumns = [
    {
      title: t('stock.pipe_type'),
      dataIndex: 'pipe_type',
      key: 'pipe_type',
      width: 120,
      render: (_: unknown, __: unknown, index: number) => (
        <Form.Item
          name={['pipes', index, 'pipe_type']}
          rules={[{ required: true, message: t('common.required') }]}
          style={{ margin: 0 }}
        >
          <Select style={{ width: 120 }} disabled={isEdit}>
            {DETAILED_PIPE_TYPES.map((type) => (
              <Select.Option key={type} value={type}>
                {t(`pipe_type.${type}`, type)}
              </Select.Option>
            ))}
          </Select>
        </Form.Item>
      ),
    },
    {
      title: t('inbound.pipe_id_placeholder'),
      dataIndex: 'pipe_id',
      key: 'pipe_id',
      width: 120,
      render: (_: unknown, __: unknown, index: number) => (
        <Form.Item
          name={['pipes', index, 'pipe_id']}
          rules={[{ required: true, message: t('common.required') }]}
          style={{ margin: 0 }}
        >
          <InputNumber min={1} style={{ width: '100%' }} disabled={isEdit} />
        </Form.Item>
      ),
    },
    {
      title: t('pipes.pipe_number'),
      dataIndex: 'pipe_number',
      key: 'pipe_number',
      width: 120,
      render: (val: string) => <span>{val || '-'}</span>,
    },
    {
      title: t('pipes.grade'),
      dataIndex: 'grade',
      key: 'grade',
      width: 80,
      render: (val: string) => <span>{val || '-'}</span>,
    },
    {
      title: t('pipes.od'),
      dataIndex: 'od',
      key: 'od',
      width: 90,
      render: (val: number) => <span>{val != null ? val : '-'}</span>,
    },
    {
      title: t('pipes.wt'),
      dataIndex: 'wt',
      key: 'wt',
      width: 90,
      render: (val: number) => <span>{val != null ? val : '-'}</span>,
    },
    {
      title: t('common.actions'),
      key: 'actions',
      width: 80,
      render: (_: unknown, __: unknown, index: number) =>
        isEdit ? null : (
          <Popconfirm
            title={t('common.confirm_delete')}
            onConfirm={() => {
              const pipes = form.getFieldValue('pipes') || [];
              pipes.splice(index, 1);
              form.setFieldsValue({ pipes: [...pipes] });
            }}
          >
            <Button type="link" danger icon={<DeleteOutlined />} />
          </Popconfirm>
        ),
    },
  ];

  return (
    <PageLayout
      title={isEdit ? t('common.edit') : t('inbound.create_inbound')}
      onBack={() => navigate('/inventory/inbound')}
    >
      <Form
        form={form}
        layout="vertical"
        onFinish={handleSubmit}
        style={{ maxWidth: 1000 }}
      >
        <Form.Item
          label={t('inbound.inbound_type')}
          name="inbound_type"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <Select style={{ width: 200 }}>
            {INBOUND_TYPES.map((type) => (
              <Select.Option key={type} value={type}>
                {t(`inbound.type.${type}`)}
              </Select.Option>
            ))}
          </Select>
        </Form.Item>

        <Form.Item label={t('inbound.order_id')} name="order_id">
          <InputNumber min={1} style={{ width: 200 }} />
        </Form.Item>

        <Form.Item label={t('inbound.supplier_id')} name="supplier_id">
          <InputNumber min={1} style={{ width: 200 }} />
        </Form.Item>

        <Form.Item label={t('inbound.notes')} name="notes">
          <Input.TextArea rows={3} style={{ maxWidth: 600 }} />
        </Form.Item>

        <h3 style={{ marginBottom: 16 }}>
          <Space>
            <span>{t('inbound.pipes')}</span>
            {!isEdit && (
              <>
                <Button
                  type="primary"
                  ghost
                  size="small"
                  icon={<SearchOutlined />}
                  onClick={() => {
                    setSearchText('');
                    setSearchResults([]);
                    setSearchModalOpen(true);
                  }}
                >
                  {t('common.search')}
                </Button>
                <Button
                  type="primary"
                  ghost
                  size="small"
                  icon={<PlusOutlined />}
                  onClick={() => {
                    setBatchPipeIds('');
                    setBatchModalOpen(true);
                  }}
                >
                  {t('inbound.batch_add_pipes', '批量添加')}
                </Button>
              </>
            )}
          </Space>
        </h3>

        <Form.List name="pipes" initialValue={[]}>
          {(fields, { add }) => (
            <>
              <Table
                columns={itemColumns}
                dataSource={fields.map((field) => ({ ...field }))}
                rowKey="key"
                pagination={false}
                footer={() =>
                  isEdit ? null : (
                    <Button
                      type="dashed"
                      onClick={() =>
                        add({
                          pipe_type: 'casing',
                          pipe_id: undefined,
                        })
                      }
                      icon={<PlusOutlined />}
                      style={{ width: '100%' }}
                    >
                      {t('inbound.add_pipe')}
                    </Button>
                  )
                }
              />
            </>
          )}
        </Form.List>

        <Form.Item style={{ marginTop: 24 }}>
          <Space>
            <Button
              type="primary"
              htmlType="submit"
              loading={isEdit ? updateMutation.isPending : createMutation.isPending}
            >
              {t('common.save')}
            </Button>
            <Button onClick={() => navigate('/inventory/inbound')}>
              {t('common.cancel')}
            </Button>
          </Space>
        </Form.Item>
      </Form>

      <Modal
        title={t('common.search')}
        open={searchModalOpen}
        onCancel={() => setSearchModalOpen(false)}
        footer={null}
        width={700}
      >
        <Space style={{ marginBottom: 16 }}>
          <Input.Search
            placeholder={t('inbound.pipe_id_placeholder')}
            value={searchText}
            onChange={(e) => setSearchText(e.target.value)}
            onSearch={handlePipeSearch}
            enterButton={t('common.search')}
            loading={searchLoading}
            style={{ width: 300 }}
          />
        </Space>
        <Table
          columns={searchColumns}
          dataSource={searchResults}
          rowKey="id"
          pagination={false}
          locale={{ emptyText: t('common.no_data') }}
        />
      </Modal>

      <Modal
        title={t('inbound.batch_add_pipes', '批量添加管材')}
        open={batchModalOpen}
        onOk={handleBatchAddPipes}
        onCancel={() => setBatchModalOpen(false)}
        destroyOnClose
      >
        <Space direction="vertical" style={{ width: '100%' }}>
          <Select
            value={batchPipeType}
            onChange={setBatchPipeType}
            style={{ width: 200 }}
            options={DETAILED_PIPE_TYPES.map((pt) => ({ label: t(`pipe_type.${pt}`, pt), value: pt }))}
          />
          <Input.TextArea
            rows={6}
            value={batchPipeIds}
            onChange={(event) => setBatchPipeIds(event.target.value)}
            placeholder={t('inbound.batch_pipe_ids_placeholder', '例如：1001,1002,1003 或 1001-1010；支持空格、换行、逗号分隔')}
          />
        </Space>
      </Modal>
    </PageLayout>
  );
}
