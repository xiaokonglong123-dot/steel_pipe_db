// 焊接钢管新增/编辑表单页 — 使用 PageLayout + 共享常量
import { useEffect } from 'react';
import { Form, Input, Select, DatePicker, InputNumber, Button, Space, message } from 'antd';
import dayjs from 'dayjs';
import type { Dayjs } from 'dayjs';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { PageLayout } from '@/shared/components/PageLayout';
import { API_5CT_GRADES, WELDED_PIPE_TYPES, END_TYPES } from '@/shared/constants';
import { useWeldedPipe, useCreateWeldedPipe, useUpdateWeldedPipe } from '../hooks/useWeldedPipes';
import type { CreateWeldedPipeData } from '../types';

type WeldedPipeFormValues = Omit<CreateWeldedPipeData, 'production_date'> & {
  production_date?: Dayjs;
};

export default function WeldedPipeFormPage() {
  const { t } = useTranslation();
  const { id } = useParams<{ id?: string }>();
  const navigate = useNavigate();
  const [form] = Form.useForm<WeldedPipeFormValues>();

  const isEdit = !!id;
  const pipeId = isEdit ? Number(id) : 0;

  const { data: pipe, isLoading: loadingPipe } = useWeldedPipe(pipeId);
  const createMutation = useCreateWeldedPipe();
  const updateMutation = useUpdateWeldedPipe(pipeId);

  useEffect(() => {
    if (isEdit && pipe) {
      form.setFieldsValue({
        pipe_number: pipe.pipe_number,
        batch_number: pipe.batch_number ?? undefined,
        pipe_type: pipe.pipe_type,
        grade: pipe.grade,
        od: pipe.od,
        wt: pipe.wt,
        length: pipe.length ?? undefined,
        weight_per_unit: pipe.weight_per_unit ?? undefined,
        end_type: pipe.end_type ?? undefined,
        seam_type: pipe.seam_type ?? undefined,
        heat_number: pipe.heat_number ?? undefined,
        serial_number: pipe.serial_number ?? undefined,
        manufacturer: pipe.manufacturer ?? undefined,
        production_date: pipe.production_date ? dayjs(pipe.production_date) : undefined,
        cert_number: pipe.cert_number ?? undefined,
        notes: pipe.notes ?? undefined,
      });
    }
  }, [isEdit, pipe, form]);

  const handleSubmit = async (values: WeldedPipeFormValues) => {
    const payload: CreateWeldedPipeData = {
      ...values,
      production_date: values.production_date?.format('YYYY-MM-DD'),
    };

    try {
      if (isEdit) {
        await updateMutation.mutateAsync(payload);
      } else {
        await createMutation.mutateAsync(payload);
      }
      message.success(t('common.operate_success'));
      navigate('/pipes/welded');
    } catch {
      message.error(t('common.operate_failed'));
    }
  };

  if (isEdit && loadingPipe) {
    return <div>{t('common.loading')}</div>;
  }

  return (
    <PageLayout
      title={`${isEdit ? t('common.edit') : t('common.create')} ${t('pipes.welded_pipes')}`}
      onBack={() => navigate('/pipes/welded')}
    >
      <Form
        form={form}
        layout="vertical"
        onFinish={handleSubmit}
        style={{ maxWidth: 800 }}
      >
        <Form.Item
          label={t('pipes.pipe_number')}
          name="pipe_number"
          rules={isEdit ? [] : [{ required: true, message: t('common.required') }]}
        >
          <Input disabled={isEdit} placeholder={t('common.required')} />
        </Form.Item>

        <Form.Item label={t('pipes.batch_number')} name="batch_number">
          <Input />
        </Form.Item>

        <Form.Item
          label={t('pipes.pipe_type')}
          name="pipe_type"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <Select options={WELDED_PIPE_TYPES.map((pt) => ({ label: t('pipe_type.' + pt), value: pt }))} />
        </Form.Item>

        <Form.Item
          label={t('pipes.grade')}
          name="grade"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <Select showSearch options={API_5CT_GRADES.map((g) => ({ label: g, value: g }))} />
        </Form.Item>

        <Form.Item
          label={t('pipes.od')}
          name="od"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <InputNumber style={{ width: '100%' }} min={0} step={0.001} />
        </Form.Item>

        <Form.Item
          label={t('pipes.wt')}
          name="wt"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <InputNumber style={{ width: '100%' }} min={0} step={0.001} />
        </Form.Item>

        <Form.Item label={t('pipes.length')} name="length">
          <InputNumber style={{ width: '100%' }} min={0} step={0.01} />
        </Form.Item>

        <Form.Item label={t('pipes.weight_per_unit')} name="weight_per_unit">
          <InputNumber style={{ width: '100%' }} min={0} step={0.001} />
        </Form.Item>

        <Form.Item label={t('pipes.end_type')} name="end_type">
          <Select allowClear options={END_TYPES.map((et) => ({ label: t('pipe_type.' + et), value: et }))} />
        </Form.Item>

        <Form.Item label={t('pipes.seam_type')} name="seam_type">
          <Input />
        </Form.Item>

        <Form.Item label={t('pipes.heat_number')} name="heat_number">
          <Input />
        </Form.Item>

        <Form.Item label={t('pipes.serial_number')} name="serial_number">
          <Input />
        </Form.Item>

        <Form.Item label={t('pipes.manufacturer')} name="manufacturer">
          <Input />
        </Form.Item>

        <Form.Item label={t('pipes.production_date')} name="production_date">
          <DatePicker style={{ width: '100%' }} />
        </Form.Item>

        <Form.Item label={t('pipes.cert_number')} name="cert_number">
          <Input />
        </Form.Item>

        <Form.Item label={t('pipes.notes')} name="notes">
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
            <Button onClick={() => navigate('/pipes/welded')}>
              {t('common.cancel')}
            </Button>
          </Space>
        </Form.Item>
      </Form>
    </PageLayout>
  );
}
