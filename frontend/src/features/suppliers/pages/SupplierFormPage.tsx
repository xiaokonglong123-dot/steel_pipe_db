// Supplier create/edit form — qualification info (supply grades, contact, tax ID, etc.)
import { useEffect } from 'react';
import { Form, Input, Button, Space, message } from 'antd';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useSupplier, useCreateSupplier, useUpdateSupplier } from '../hooks/useSuppliers';
import type { CreateSupplierData } from '../types';

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
        supplier_code: supplier.supplier_code,
        name: supplier.name,
        contact_person: supplier.contact_person ?? undefined,
        phone: supplier.phone ?? undefined,
        email: supplier.email ?? undefined,
        address: supplier.address ?? undefined,
        notes: supplier.notes ?? undefined,
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
          name="supplier_code"
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
