import React, { useEffect } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  Card,
  Form,
  Input,
  Select,
  DatePicker,
  Button,
  Space,
  Row,
  Col,
  message,
  Spin,
} from 'antd';
import dayjs from 'dayjs';
import { pipeApi } from '../api/pipeApi';
import type { SeamlessPipe, ScreenPipe } from '../api/pipeApi';

const { TextArea } = Input;

const statusOptions = [
  { label: '在库', value: 'in_stock' },
  { label: '已出库', value: 'outbound' },
  { label: '报废', value: 'scrapped' },
];

const gradeOptions = [
  { label: 'J55', value: 'J55' },
  { label: 'K55', value: 'K55' },
  { label: 'N80', value: 'N80' },
  { label: 'L80', value: 'L80' },
  { label: 'P110', value: 'P110' },
];

const screenTypeOptions = [
  { label: '割缝筛管', value: 'slotted' },
  { label: '绕丝筛管', value: 'wire_wrapped' },
  { label: '预充填筛管', value: 'prepacked' },
];

export default function PipeFormPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [form] = Form.useForm();
  const isEdit = Boolean(id);
  const pipeType = Form.useWatch('pipeType', form) || 'seamless';

  const { data: editData, isLoading: loadingEdit } = useQuery({
    queryKey: ['pipe-edit', id],
    queryFn: () =>
      pipeApi.getSeamless(id!).catch(() => pipeApi.getScreen(id!)),
    enabled: isEdit,
  });

  useEffect(() => {
    if (editData?.data?.data) {
      const record = editData.data.data;
      const isScreen = 'screen_type' in record;
      form.setFieldsValue({
        ...record,
        pipeType: isScreen ? 'screen' : 'seamless',
        production_date: record.production_date ? dayjs(record.production_date) : undefined,
      });
    }
  }, [editData, form]);

  const createMutation = useMutation({
    mutationFn: (values: Record<string, unknown>) => {
      const payload = { ...values };
      delete payload.pipeType;
      if (payload.production_date) {
        payload.production_date = dayjs(payload.production_date as string).format('YYYY-MM-DD');
      }
      return pipeType === 'seamless'
        ? pipeApi.createSeamless(payload as Partial<SeamlessPipe>)
        : pipeApi.createScreen(payload as Partial<ScreenPipe>);
    },
    onSuccess: () => {
      message.success('管材创建成功');
      queryClient.invalidateQueries({ queryKey: ['seamless-pipes'] });
      queryClient.invalidateQueries({ queryKey: ['screen-pipes'] });
      navigate('/pipes');
    },
    onError: () => {
      message.error('创建失败');
    },
  });

  const updateMutation = useMutation({
    mutationFn: (values: Record<string, unknown>) => {
      const payload = { ...values };
      delete payload.pipeType;
      if (payload.production_date) {
        payload.production_date = dayjs(payload.production_date as string).format('YYYY-MM-DD');
      }
      return pipeType === 'seamless'
        ? pipeApi.updateSeamless(id!, payload as Partial<SeamlessPipe>)
        : pipeApi.updateScreen(id!, payload as Partial<ScreenPipe>);
    },
    onSuccess: () => {
      message.success('管材更新成功');
      queryClient.invalidateQueries({ queryKey: ['seamless-pipes'] });
      queryClient.invalidateQueries({ queryKey: ['screen-pipes'] });
      navigate('/pipes');
    },
    onError: () => {
      message.error('更新失败');
    },
  });

  const handleSubmit = async () => {
    try {
      const values = await form.validateFields();
      if (isEdit) {
        updateMutation.mutate(values);
      } else {
        createMutation.mutate(values);
      }
    } catch {
      /* empty - validation errors shown by antd */
    }
  };

  const isSubmitting = createMutation.isPending || updateMutation.isPending;

  return (
    <Card
      title={isEdit ? '编辑管材' : '新增管材'}
      extra={
        <Button onClick={() => navigate('/pipes')} type="default">
          返回列表
        </Button>
      }
      styles={{ body: { padding: 24 } }}
    >
      <Spin spinning={isEdit && loadingEdit}>
        <Form
          form={form}
          layout="vertical"
          initialValues={{ pipeType: 'seamless', status: 'in_stock' }}
          style={{ maxWidth: 800 }}
        >
          {!isEdit && (
            <Row gutter={24}>
              <Col xs={24} sm={12} md={8}>
                <Form.Item
                  name="pipeType"
                  label="管材类型"
                  rules={[{ required: true, message: '请选择管材类型' }]}
                >
                  <Select
                    options={[
                      { label: '无缝钢管', value: 'seamless' },
                      { label: '筛管', value: 'screen' },
                    ]}
                  />
                </Form.Item>
              </Col>
            </Row>
          )}

          <Row gutter={24}>
            <Col xs={24} sm={12} md={8}>
              <Form.Item
                name="pipe_number"
                label="管材编号"
                rules={[{ required: true, message: '请输入管材编号' }]}
              >
                <Input placeholder="例如: J55 4.500in×11.60lb SC-H2405-000001" />
              </Form.Item>
            </Col>
            <Col xs={24} sm={12} md={8}>
              <Form.Item
                name="grade"
                label="钢级"
                rules={[{ required: true, message: '请选择钢级' }]}
              >
                <Select placeholder="请选择钢级" options={gradeOptions} />
              </Form.Item>
            </Col>
            <Col xs={24} sm={12} md={8}>
              <Form.Item
                name="status"
                label="状态"
                rules={[{ required: true, message: '请选择状态' }]}
              >
                <Select placeholder="请选择状态" options={statusOptions} />
              </Form.Item>
            </Col>
          </Row>

          <Row gutter={24}>
            <Col xs={12} sm={8} md={6}>
              <Form.Item
                name="od"
                label="外径 (in)"
                rules={[{ required: true, message: '请输入外径' }]}
              >
                <Input type="number" step="0.001" placeholder="4.500" />
              </Form.Item>
            </Col>
            <Col xs={12} sm={8} md={6}>
              <Form.Item
                name="wt"
                label="壁厚 (in)"
                rules={[{ required: true, message: '请输入壁厚' }]}
              >
                <Input type="number" step="0.001" placeholder="0.250" />
              </Form.Item>
            </Col>
            <Col xs={12} sm={8} md={6}>
              <Form.Item
                name="length"
                label="长度 (m)"
                rules={[{ required: true, message: '请输入长度' }]}
              >
                <Input type="number" step="0.01" placeholder="10.00" />
              </Form.Item>
            </Col>
            <Col xs={12} sm={8} md={6}>
              <Form.Item
                name="weight"
                label="重量 (kg)"
                rules={[{ required: true, message: '请输入重量' }]}
              >
                <Input type="number" step="0.1" placeholder="500.0" />
              </Form.Item>
            </Col>
          </Row>

          <Row gutter={24}>
            <Col xs={12} sm={8} md={6}>
              <Form.Item name="connection_type" label="接箍类型">
                <Input placeholder="API 5CT" />
              </Form.Item>
            </Col>
            <Col xs={12} sm={8} md={6}>
              <Form.Item name="heat_number" label="炉号">
                <Input placeholder="例如: H2405" />
              </Form.Item>
            </Col>
            <Col xs={12} sm={8} md={6}>
              <Form.Item name="production_date" label="生产日期">
                <DatePicker style={{ width: '100%' }} />
              </Form.Item>
            </Col>
            <Col xs={12} sm={8} md={6}>
              <Form.Item name="location" label="库位">
                <Input placeholder="例如: A区-01排" />
              </Form.Item>
            </Col>
          </Row>

          {pipeType === 'screen' && (
            <Row gutter={24}>
              <Col xs={24} sm={12} md={8}>
                <Form.Item
                  name="screen_type"
                  label="筛管类型"
                  rules={[{ required: true, message: '请选择筛管类型' }]}
                >
                  <Select placeholder="请选择筛管类型" options={screenTypeOptions} />
                </Form.Item>
              </Col>
              <Col xs={12} sm={8} md={6}>
                <Form.Item name="slot_width" label="缝宽 (mm)">
                  <Input type="number" step="0.1" placeholder="0.5" />
                </Form.Item>
              </Col>
              <Col xs={12} sm={8} md={6}>
                <Form.Item name="open_area" label="开口面积 (%)">
                  <Input type="number" step="0.1" placeholder="15.0" />
                </Form.Item>
              </Col>
            </Row>
          )}

          <Row gutter={24}>
            <Col xs={24}>
              <Form.Item name="notes" label="备注">
                <TextArea rows={3} placeholder="可选备注信息" />
              </Form.Item>
            </Col>
          </Row>

          <Form.Item>
            <Space>
              <Button type="primary" onClick={handleSubmit} loading={isSubmitting}>
                {isEdit ? '保存修改' : '创建'}
              </Button>
              <Button onClick={() => navigate('/pipes')}>取消</Button>
            </Space>
          </Form.Item>
        </Form>
      </Spin>
    </Card>
  );
}
