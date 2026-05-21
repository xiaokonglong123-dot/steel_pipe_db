import { useEffect } from 'react';
import { Form, Input, Select, DatePicker, InputNumber, Button, Space, message } from 'antd';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useCert, useCreateCert, useUpdateCert } from '../hooks/useQuality';
import type { CreateQualityCertData } from '../types';

const PIPE_TYPES = ['casing', 'tubing', 'coupling', 'accessory'];
const API_5CT_GRADES = ['H40', 'J55', 'K55', 'N80', 'L80', 'C90', 'T95', 'P110', 'Q125'];
const CERT_STATUSES = ['draft', 'active', 'void'];

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
        batch_number: cert.batch_number,
        pipe_type: cert.pipe_type,
        grade: cert.grade,
        od: cert.od,
        wt: cert.wt,
        length: cert.length,
        quantity: cert.quantity,
        heat_number: cert.heat_number,
        manufacturer: cert.manufacturer,
        production_date: cert.production_date,
        test_pressure: cert.test_pressure,
        yield_strength: cert.yield_strength,
        tensile_strength: cert.tensile_strength,
        elongation: cert.elongation,
        hardness: cert.hardness,
        inspection_standard: cert.inspection_standard,
        inspector: cert.inspector,
        cert_date: cert.cert_date,
        status: cert.status,
        notes: cert.notes,
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
    <div>
      <h2 style={{ marginBottom: 24 }}>
        {isEdit ? t('common.edit') : t('common.create')} Certificate
      </h2>
      <Form
        form={form}
        layout="vertical"
        onFinish={handleSubmit}
        style={{ maxWidth: 800 }}
      >
        <Form.Item
          label="Cert Number"
          name="cert_number"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <Input disabled={isEdit} />
        </Form.Item>

        <Form.Item label="Batch Number" name="batch_number">
          <Input />
        </Form.Item>

        <Form.Item
          label="Pipe Type"
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
          label="Grade"
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
          label="OD (in)"
          name="od"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <InputNumber style={{ width: '100%' }} min={0} step={0.001} />
        </Form.Item>

        <Form.Item
          label="WT (in)"
          name="wt"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <InputNumber style={{ width: '100%' }} min={0} step={0.001} />
        </Form.Item>

        <Form.Item label="Length (m)" name="length">
          <InputNumber style={{ width: '100%' }} min={0} step={0.01} />
        </Form.Item>

        <Form.Item
          label="Quantity"
          name="quantity"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <InputNumber style={{ width: '100%' }} min={0} step={1} />
        </Form.Item>

        <Form.Item label="Heat Number" name="heat_number">
          <Input />
        </Form.Item>

        <Form.Item label="Manufacturer" name="manufacturer">
          <Input />
        </Form.Item>

        <Form.Item label="Production Date" name="production_date">
          <DatePicker style={{ width: '100%' }} />
        </Form.Item>

        <Form.Item label="Test Pressure" name="test_pressure">
          <InputNumber style={{ width: '100%' }} min={0} step={0.1} />
        </Form.Item>

        <Form.Item label="Yield Strength" name="yield_strength">
          <InputNumber style={{ width: '100%' }} min={0} step={0.1} />
        </Form.Item>

        <Form.Item label="Tensile Strength" name="tensile_strength">
          <InputNumber style={{ width: '100%' }} min={0} step={0.1} />
        </Form.Item>

        <Form.Item label="Elongation (%)" name="elongation">
          <InputNumber style={{ width: '100%' }} min={0} step={0.1} />
        </Form.Item>

        <Form.Item label="Hardness" name="hardness">
          <InputNumber style={{ width: '100%' }} min={0} step={0.1} />
        </Form.Item>

        <Form.Item label="Inspection Standard" name="inspection_standard">
          <Input />
        </Form.Item>

        <Form.Item label="Inspector" name="inspector">
          <Input />
        </Form.Item>

        <Form.Item label="Cert Date" name="cert_date">
          <DatePicker style={{ width: '100%' }} />
        </Form.Item>

        <Form.Item
          label="Status"
          name="status"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <Select>
            {CERT_STATUSES.map((s) => (
              <Select.Option key={s} value={s}>
                {s}
              </Select.Option>
            ))}
          </Select>
        </Form.Item>

        <Form.Item label="Notes" name="notes">
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
    </div>
  );
}
