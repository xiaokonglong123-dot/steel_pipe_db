import React, { useEffect, useCallback } from 'react';
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
import { qualityApi, QualityCert } from '../api/qualityApi';

const { TextArea } = Input;

const pipeTypeOptions = [
  { label: '套管', value: 'casing' },
  { label: '油管', value: 'tubing' },
  { label: '光管', value: 'plain_end' },
];

const resultOptions = [
  { label: '合格', value: 'pass' },
  { label: '不合格', value: 'fail' },
  { label: '待检', value: 'pending' },
];

async function fetchCertById(id: string): Promise<{ cert: QualityCert; pipeType: string }> {
  const types = ['casing', 'tubing', 'plain_end'];
  for (const pt of types) {
    try {
      const res = await qualityApi.getCert(pt, id);
      if (res.data?.data) {
        return { cert: res.data.data, pipeType: pt };
      }
    } catch {
      /* try next type */
    }
  }
  throw new Error('未找到质检证书');
}

export default function QualityFormPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [form] = Form.useForm();
  const isEdit = Boolean(id);
  const pipeType = Form.useWatch('pipe_type', form);

  const { data: editData, isLoading: loadingEdit } = useQuery({
    queryKey: ['quality-cert-edit', id],
    queryFn: () => fetchCertById(id!),
    enabled: isEdit,
  });

  useEffect(() => {
    if (editData) {
      const record = editData.cert;
      form.setFieldsValue({
        ...record,
        inspect_date: record.inspect_date ? dayjs(record.inspect_date) : undefined,
      });
    }
  }, [editData, form]);

  const createMutation = useMutation({
    mutationFn: (values: Record<string, unknown>) => {
      const payload = { ...values };
      if (payload.inspect_date) {
        payload.inspect_date = dayjs(payload.inspect_date as string).format('YYYY-MM-DD');
      }
      return qualityApi.createCert(
        payload.pipe_type as string,
        payload as Partial<QualityCert>
      );
    },
    onSuccess: () => {
      message.success('质检证书创建成功');
      queryClient.invalidateQueries({ queryKey: ['quality-certs'] });
      navigate('/quality/certs');
    },
    onError: () => {
      message.error('创建失败');
    },
  });

  const updateMutation = useMutation({
    mutationFn: (values: Record<string, unknown>) => {
      const payload = { ...values };
      if (payload.inspect_date) {
        payload.inspect_date = dayjs(payload.inspect_date as string).format('YYYY-MM-DD');
      }
      return qualityApi.updateCert(
        editData!.pipeType,
        id!,
        payload as Partial<QualityCert>
      );
    },
    onSuccess: () => {
      message.success('质检证书更新成功');
      queryClient.invalidateQueries({ queryKey: ['quality-certs'] });
      navigate('/quality/certs');
    },
    onError: () => {
      message.error('更新失败');
    },
  });

  const handleSubmit = useCallback(async () => {
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
  }, [form, isEdit, createMutation, updateMutation]);

  const isSubmitting = createMutation.isPending || updateMutation.isPending;

  return (
    <Card
      title={isEdit ? '编辑质检证书' : '新增质检证书'}
      extra={
        <Button onClick={() => navigate('/quality/certs')} type="default">
          返回列表
        </Button>
      }
      styles={{ body: { padding: 24 } }}
    >
      <Spin spinning={isEdit && loadingEdit}>
        <Form
          form={form}
          layout="vertical"
          initialValues={{ pipe_type: 'casing', result: 'pending' }}
          style={{ maxWidth: 800 }}
        >
          <Row gutter={24}>
            <Col xs={24} sm={12} md={8}>
              <Form.Item
                name="cert_no"
                label="证书编号"
                rules={[{ required: true, message: '请输入证书编号' }]}
              >
                <Input placeholder="例如: QC-2024-0001" />
              </Form.Item>
            </Col>
            <Col xs={24} sm={12} md={8}>
              <Form.Item
                name="pipe_type"
                label="管材类型"
                rules={[{ required: true, message: '请选择管材类型' }]}
              >
                <Select
                  placeholder="请选择管材类型"
                  options={pipeTypeOptions}
                  disabled={isEdit}
                />
              </Form.Item>
            </Col>
            <Col xs={24} sm={12} md={8}>
              <Form.Item
                name="pipe_id"
                label="管材ID"
                rules={[{ required: true, message: '请输入管材ID' }]}
              >
                <Input placeholder="关联管材的ID" />
              </Form.Item>
            </Col>
          </Row>

          <Row gutter={24}>
            <Col xs={24} sm={12} md={8}>
              <Form.Item
                name="inspect_date"
                label="检验日期"
                rules={[{ required: true, message: '请选择检验日期' }]}
              >
                <DatePicker style={{ width: '100%' }} />
              </Form.Item>
            </Col>
            <Col xs={24} sm={12} md={8}>
              <Form.Item
                name="inspector"
                label="检验人"
                rules={[{ required: true, message: '请输入检验人' }]}
              >
                <Input placeholder="检验员姓名" />
              </Form.Item>
            </Col>
            <Col xs={24} sm={12} md={8}>
              <Form.Item name="agency" label="检验机构">
                <Input placeholder="可选，第三方检验机构" />
              </Form.Item>
            </Col>
          </Row>

          <Row gutter={24}>
            <Col xs={24} sm={12} md={8}>
              <Form.Item
                name="result"
                label="检验结果"
                rules={[{ required: true, message: '请选择检验结果' }]}
              >
                <Select placeholder="请选择检验结果" options={resultOptions} />
              </Form.Item>
            </Col>
          </Row>

          <Row gutter={24}>
            <Col xs={24}>
              <Form.Item name="items_json" label="检验项目 (JSON)">
                <TextArea
                  rows={4}
                  placeholder='[{"item_no":1,"test_item":"抗拉强度","specification":"≥ 862 MPa","measured_value":"910","result":"pass"}]'
                />
              </Form.Item>
            </Col>
          </Row>

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
              <Button onClick={() => navigate('/quality/certs')}>取消</Button>
            </Space>
          </Form.Item>
        </Form>
      </Spin>
    </Card>
  );
}
