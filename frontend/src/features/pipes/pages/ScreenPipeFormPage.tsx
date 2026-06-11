// 筛管新增/编辑表单页 — 支持筛管类型（绕丝/预填充/割缝等）及基管规格录入
import { useEffect } from 'react';
import { Form, Input, Select, DatePicker, InputNumber, Button, Space, message } from 'antd';
import dayjs from 'dayjs';
import type { Dayjs } from 'dayjs';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useScreenPipe, useCreateScreenPipe, useUpdateScreenPipe } from '../hooks/useScreenPipes';
import type { CreateScreenPipeData } from '../types';

const SCREEN_TYPES = ['wire_wrapped', 'pre_packed', 'slotted_liner', 'mesh'];
const API_5CT_GRADES = ['H40', 'J55', 'K55', 'N80', 'L80', 'C90', 'T95', 'P110', 'Q125'];
const END_TYPES = ['plain_end', 'threaded', 'threaded_coupled', 'upset'];

type ScreenPipeFormValues = Omit<CreateScreenPipeData, 'production_date'> & {
  production_date?: Dayjs;
};

export default function ScreenPipeFormPage() {
  const { t } = useTranslation();
  const { id } = useParams<{ id?: string }>();
  const navigate = useNavigate();
  const [form] = Form.useForm<ScreenPipeFormValues>();

  const isEdit = !!id;
  const pipeId = isEdit ? Number(id) : 0;

  const { data: pipe, isLoading: loadingPipe } = useScreenPipe(pipeId);
  const createMutation = useCreateScreenPipe();
  const updateMutation = useUpdateScreenPipe(pipeId);

  useEffect(() => {
    if (isEdit && pipe) {
      form.setFieldsValue({
        pipe_number: pipe.pipe_number,
        batch_number: pipe.batch_number ?? undefined,
        screen_type: pipe.screen_type,
        slot_size: pipe.slot_size ?? undefined,
        filtration_grade: pipe.filtration_grade ?? undefined,
        base_od: pipe.base_od,
        base_wt: pipe.base_wt,
        base_grade: pipe.base_grade,
        base_end_type: pipe.base_end_type ?? undefined,
        length: pipe.length ?? undefined,
        weight_per_unit: pipe.weight_per_unit ?? undefined,
        heat_number: pipe.heat_number ?? undefined,
        serial_number: pipe.serial_number ?? undefined,
        manufacturer: pipe.manufacturer ?? undefined,
        production_date: pipe.production_date ? dayjs(pipe.production_date) : undefined,
        cert_number: pipe.cert_number ?? undefined,
        notes: pipe.notes ?? undefined,
      });
    }
  }, [isEdit, pipe, form]);

  const handleSubmit = async (values: ScreenPipeFormValues) => {
    const payload: CreateScreenPipeData = {
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
      navigate('/pipes/screen');
    } catch {
      message.error(t('common.operate_failed'));
    }
  };

  if (isEdit && loadingPipe) {
    return <div>{t('common.loading')}</div>;
  }

  return (
    <div>
      <h2 style={{ marginBottom: 24 }}>
        {isEdit ? t('common.edit') : t('common.create')} {t('nav.screen_pipes')}
      </h2>
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
          label={t('screen_pipes.screen_type')}
          name="screen_type"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <Select>
            {SCREEN_TYPES.map((type) => (
              <Select.Option key={type} value={type}>
                {type}
              </Select.Option>
            ))}
          </Select>
        </Form.Item>

        <Form.Item label={t('screen_pipes.slot_size')} name="slot_size">
          <InputNumber style={{ width: '100%' }} min={0} step={0.001} />
        </Form.Item>

        <Form.Item label={t('screen_pipes.filtration_grade')} name="filtration_grade">
          <Input />
        </Form.Item>

        <Form.Item
          label={t('pipes.od')}
          name="base_od"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <InputNumber style={{ width: '100%' }} min={0} step={0.001} />
        </Form.Item>

        <Form.Item
          label={t('pipes.wt')}
          name="base_wt"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <InputNumber style={{ width: '100%' }} min={0} step={0.001} />
        </Form.Item>

        <Form.Item
          label={t('screen_pipes.base_grade')}
          name="base_grade"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <Select showSearch>
            {API_5CT_GRADES.map((grade) => (
              <Select.Option key={grade} value={grade}>
                {grade}
              </Select.Option>
            ))}
          </Select>
        </Form.Item>

        <Form.Item label={t('screen_pipes.base_end_type')} name="base_end_type">
          <Select allowClear>
            {END_TYPES.map((type) => (
              <Select.Option key={type} value={type}>
                {type}
              </Select.Option>
            ))}
          </Select>
        </Form.Item>

        <Form.Item label={t('pipes.length')} name="length">
          <InputNumber style={{ width: '100%' }} min={0} step={0.01} />
        </Form.Item>

        <Form.Item label={t('pipes.weight_per_unit')} name="weight_per_unit">
          <InputNumber style={{ width: '100%' }} min={0} step={0.001} />
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

        <Form.Item label={t('common.notes')} name="notes">
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
            <Button onClick={() => navigate('/pipes/screen')}>
              {t('common.cancel')}
            </Button>
          </Space>
        </Form.Item>
      </Form>
    </div>
  );
}
