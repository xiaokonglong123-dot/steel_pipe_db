// Supplier create/edit form — qualification info (supply grades, contact, tax ID, etc.)
import { useEffect } from 'react';
import { Form, Input, Select, Button, Space, message } from 'antd';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useSupplier, useCreateSupplier, useUpdateSupplier } from '../hooks/useSuppliers';
import type { CreateSupplierData } from '../types';

const API_5CT_GRADES = ['H40', 'J55', 'K55', 'N80', 'L80', 'C90', 'T95', 'P110', 'Q125'];

export default function SupplierFormPage() {
  const { t } = useTranslation();
  const { id } = useParams<{ id?: string }>();
  const navigate = useNavigate();
  const [form] = Form.useForm<CreateSupplierData>();

  const isEdit = !!id;
  const supplierId = isEdit ? Number(id) : 0;

  const { data: supplier, isLoading: loadingSupplier } = useSupplier(supplierId);
  const createMutation = useCreateSupplier();
  const updateMutation = useUpdateSupplier(supplierId);

  useEffect(() => {
    if (isEdit && supplier) {
      form.setFieldsValue({
        code: supplier.code,
        name: supplier.name,
        contact_person: supplier.contact_person,
        phone: supplier.phone,
        email: supplier.email,
        address: supplier.address,
        tax_id: supplier.tax_id,
        bank_info: supplier.bank_info,
        grade_supply: supplier.grade_supply,
        status: supplier.status,
        notes: supplier.notes,
      });
    }
  }, [isEdit, supplier, form]);

  const handleSubmit = async (values: CreateSupplierData) => {
    try {
      if (isEdit) {
        await updateMutation.mutateAsync(values);
      } else {
        await createMutation.mutateAsync(values);
      }
      message.success(t('common.operate_success'));
      navigate('/suppliers');
    } catch {
      message.error(t('common.operate_failed'));
    }
  };

  if (isEdit && loadingSupplier) {
    return <div>{t('common.loading')}</div>;
  }

  return (
    <div>
      <h2 style={{ marginBottom: 24 }}>
        {isEdit ? t('common.edit') : t('common.create')} {t('suppliers.name')}
      </h2>
      <Form
        form={form}
        layout="vertical"
        onFinish={handleSubmit}
        style={{ maxWidth: 800 }}
      >
        <Form.Item
          label={t('suppliers.code')}
          name="code"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <Input disabled={isEdit} placeholder={t('common.required')} />
        </Form.Item>

        <Form.Item
          label={t('suppliers.name')}
          name="name"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <Input />
        </Form.Item>

        <Form.Item label={t('suppliers.contact_person')} name="contact_person">
          <Input />
        </Form.Item>

        <Form.Item label={t('suppliers.phone')} name="phone">
          <Input />
        </Form.Item>

        <Form.Item label={t('suppliers.email')} name="email">
          <Input />
        </Form.Item>

        <Form.Item label={t('suppliers.address')} name="address">
          <Input.TextArea rows={2} />
        </Form.Item>

        <Form.Item label={t('suppliers.tax_id')} name="tax_id">
          <Input />
        </Form.Item>

        <Form.Item label={t('suppliers.bank_info')} name="bank_info">
          <Input />
        </Form.Item>

        <Form.Item label={t('suppliers.grade_supply')} name="grade_supply">
          <Select mode="tags" placeholder={t('suppliers.select_grades')}>
            {API_5CT_GRADES.map((g) => (
              <Select.Option key={g} value={g}>
                {g}
              </Select.Option>
            ))}
          </Select>
        </Form.Item>

        <Form.Item label={t('suppliers.status')} name="status">
          <Select>
            <Select.Option value="active">{t('suppliers.status_active')}</Select.Option>
            <Select.Option value="inactive">{t('suppliers.status_inactive')}</Select.Option>
          </Select>
        </Form.Item>

        <Form.Item label={t('suppliers.notes')} name="notes">
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
            <Button onClick={() => navigate('/suppliers')}>
              {t('common.cancel')}
            </Button>
          </Space>
        </Form.Item>
      </Form>
    </div>
  );
}
