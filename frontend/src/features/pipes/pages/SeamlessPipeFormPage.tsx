import { useEffect } from 'react';
import { Form, Input, Select, DatePicker, InputNumber, Button, Space, message } from 'antd';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useSeamlessPipe, useCreateSeamlessPipe, useUpdateSeamlessPipe } from '../hooks/useSeamlessPipes';
import type { CreateSeamlessPipeData } from '../types';

const PIPE_TYPES = ['casing', 'tubing', 'coupling', 'accessory'];
const API_5CT_GRADES = ['H40', 'J55', 'K55', 'N80', 'L80', 'C90', 'T95', 'P110', 'Q125'];
const END_TYPES = ['plain_end', 'threaded', 'threaded_coupled', 'upset'];

export default function SeamlessPipeFormPage() {
  const { t } = useTranslation();
  const { id } = useParams<{ id?: string }>();
  const navigate = useNavigate();
  const [form] = Form.useForm<CreateSeamlessPipeData>();

  const isEdit = !!id;
  const pipeId = isEdit ? Number(id) : 0;

  const { data: pipe, isLoading: loadingPipe } = useSeamlessPipe(pipeId);
  const createMutation = useCreateSeamlessPipe();
  const updateMutation = useUpdateSeamlessPipe(pipeId);

  useEffect(() => {
    if (isEdit && pipe) {
      form.setFieldsValue({
        pipe_number: pipe.pipe_number,
        batch_number: pipe.batch_number,
        pipe_type: pipe.pipe_type,
        grade: pipe.grade,
        od: pipe.od,
        wt: pipe.wt,
        length: pipe.length,
        weight_per_unit: pipe.weight_per_unit,
        end_type: pipe.end_type,
        coupling_type: pipe.coupling_type,
        coupling_od: pipe.coupling_od,
        coupling_length: pipe.coupling_length,
        heat_number: pipe.heat_number,
        serial_number: pipe.serial_number,
        manufacturer: pipe.manufacturer,
        production_date: pipe.production_date,
        cert_number: pipe.cert_number,
        notes: pipe.notes,
      });
    }
  }, [isEdit, pipe, form]);

  const handleSubmit = async (values: CreateSeamlessPipeData) => {
    try {
      if (isEdit) {
        await updateMutation.mutateAsync(values);
      } else {
        await createMutation.mutateAsync(values);
      }
      message.success(t('common.operate_success'));
      navigate('/pipes/seamless');
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
        {isEdit ? t('common.edit') : t('common.create')} {t('nav.seamless_pipes')}
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
          label={t('pipes.pipe_type')}
          name="pipe_type"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <Select>
            {PIPE_TYPES.map((type) => (
              <Select.Option key={type} value={type}>
                {type}
              </Select.Option>
            ))}
          </Select>
        </Form.Item>

        <Form.Item
          label={t('pipes.grade')}
          name="grade"
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
          <Select allowClear>
            {END_TYPES.map((type) => (
              <Select.Option key={type} value={type}>
                {type}
              </Select.Option>
            ))}
          </Select>
        </Form.Item>

        <Form.Item label={t('pipes.coupling_type')} name="coupling_type">
          <Input />
        </Form.Item>

        <Form.Item label={t('pipes.coupling_od')} name="coupling_od">
          <InputNumber style={{ width: '100%' }} min={0} step={0.001} />
        </Form.Item>

        <Form.Item label={t('pipes.coupling_length')} name="coupling_length">
          <InputNumber style={{ width: '100%' }} min={0} step={0.01} />
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
            <Button onClick={() => navigate('/pipes/seamless')}>
              {t('common.cancel')}
            </Button>
          </Space>
        </Form.Item>
      </Form>
    </div>
  );
}
