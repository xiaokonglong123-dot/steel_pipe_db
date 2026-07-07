// 质检证书新增/编辑表单 — 使用 PageLayout + 共享常量
import { useEffect } from 'react';
import { Form, Input, Select, DatePicker, InputNumber, Button, Space, message } from 'antd';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { PageLayout } from '@/shared/components/PageLayout';
import { DETAILED_PIPE_TYPES, API_5CT_GRADES } from '@/shared/constants';
import { useCert, useCreateCert, useUpdateCert } from '../hooks/useQuality';
import type { CreateQualityCertData } from '../types';

export default function CertFormPage() {
  const { t } = useTranslation();
  const { id } = useParams<{ id?: string }>();
  const navigate = useNavigate();
  const [form] = Form.useForm<CreateQualityCertData>();

  const isEdit = !!id;
  const certId = isEdit ? Number(id) : 0;

  const { data: cert, isLoading: loadingCert } = useCert(certId);
  const createMutation = useCreateCert();
  const updateMutation = useUpdateCert(certId);

  useEffect(() => {
    if (isEdit && cert) {
      form.setFieldsValue({
        cert_number: cert.cert_number,
        pipe_type: cert.pipe_type,
        pipe_id: cert.pipe_id,
        cert_date: cert.cert_date ?? undefined,
        result: cert.result,
        inspector: cert.inspector ?? undefined,
        inspection_body: cert.inspection_body ?? undefined,
        notes: cert.notes ?? undefined,
      });
    }
  }, [isEdit, cert, form]);

  const handleSubmit = async (values: CreateQualityCertData) => {
    try {
      if (isEdit) {
        await updateMutation.mutateAsync(values);
      } else {
        await createMutation.mutateAsync(values);
      }
      message.success(t('common.operate_success'));
      navigate('/quality/certs');
    } catch {
      message.error(t('common.operate_failed'));
    }
  };

  if (isEdit && loadingCert) {
    return <div>{t('common.loading')}</div>;
  }

  return (
    <PageLayout
      title={isEdit ? t('quality.edit_certificate') : t('quality.create_certificate')}
      onBack={() => navigate('/quality/certs')}
    >
      <Form
        form={form}
        layout="vertical"
        onFinish={handleSubmit}
        style={{ maxWidth: 800 }}
      >
        <Form.Item
          label={t('quality.cert_number')}
          name="cert_number"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <Input disabled={isEdit} />
        </Form.Item>

        <Form.Item label={t('quality.batch_number')} name="batch_number">
          <Input />
        </Form.Item>

        <Form.Item
          label={t('quality.pipe_type')}
          name="pipe_type"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <Select options={DETAILED_PIPE_TYPES.map((pt) => ({ label: t('pipe_type.' + pt), value: pt }))} />
        </Form.Item>

        <Form.Item
          label={t('quality.grade')}
          name="grade"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <Select showSearch options={API_5CT_GRADES.map((g) => ({ label: g, value: g }))} />
        </Form.Item>

        <Form.Item
          label={t('quality.od')}
          name="od"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <InputNumber style={{ width: '100%' }} min={0} step={0.001} />
        </Form.Item>

        <Form.Item
          label={t('quality.wt')}
          name="wt"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <InputNumber style={{ width: '100%' }} min={0} step={0.001} />
        </Form.Item>

        <Form.Item label={t('quality.length')} name="length">
          <InputNumber style={{ width: '100%' }} min={0} step={0.01} />
        </Form.Item>

        <Form.Item
          label={t('quality.quantity')}
          name="quantity"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <InputNumber style={{ width: '100%' }} min={0} step={1} />
        </Form.Item>

        <Form.Item label={t('quality.heat_number')} name="heat_number">
          <Input />
        </Form.Item>

        <Form.Item label={t('quality.manufacturer')} name="manufacturer">
          <Input />
        </Form.Item>

        <Form.Item label={t('quality.production_date')} name="production_date">
          <DatePicker style={{ width: '100%' }} />
        </Form.Item>

        <Form.Item label={t('quality.test_pressure')} name="test_pressure">
          <InputNumber style={{ width: '100%' }} min={0} step={0.1} />
        </Form.Item>

        <Form.Item label={t('quality.yield_strength')} name="yield_strength">
          <InputNumber style={{ width: '100%' }} min={0} step={0.1} />
        </Form.Item>

        <Form.Item label={t('quality.tensile_strength')} name="tensile_strength">
          <InputNumber style={{ width: '100%' }} min={0} step={0.1} />
        </Form.Item>

        <Form.Item label={t('quality.elongation')} name="elongation">
          <InputNumber style={{ width: '100%' }} min={0} step={0.1} />
        </Form.Item>

        <Form.Item label={t('quality.hardness')} name="hardness">
          <InputNumber style={{ width: '100%' }} min={0} step={0.1} />
        </Form.Item>

        <Form.Item label={t('quality.inspection_standard')} name="inspection_standard">
          <Input />
        </Form.Item>

        <Form.Item label={t('quality.inspector')} name="inspector">
          <Input />
        </Form.Item>

        <Form.Item label={t('quality.cert_date')} name="cert_date">
          <DatePicker style={{ width: '100%' }} />
        </Form.Item>

        <Form.Item
          label={t('quality.status')}
          name="result"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <Select>
            <Select.Option key="pass" value="pass">Pass</Select.Option>
            <Select.Option key="fail" value="fail">Fail</Select.Option>
          </Select>
        </Form.Item>

        <Form.Item label={t('quality.notes')} name="notes">
          <Input.TextArea rows={3} />
        </Form.Item>

        <Form.Item>
          <Space>
            <Button
              type="primary"
              htmlType="submit"
              loading={createMutation.isPending || updateMutation.isPending}
            >
              {t('common.save')}
            </Button>
            <Button onClick={() => navigate('/quality/certs')}>
              {t('common.cancel')}
            </Button>
          </Space>
        </Form.Item>
      </Form>
    </PageLayout>
  );
}
