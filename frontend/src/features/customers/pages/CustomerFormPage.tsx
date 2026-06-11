// Customer create/edit form — basic info (contact, tax ID, bank info, industry, etc.)
import { useEffect } from 'react';
import { Form, Input, Button, Space, message } from 'antd';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useCustomer, useCreateCustomer, useUpdateCustomer } from '../hooks/useCustomers';
import type { CreateCustomerData } from '../types';

export default function CustomerFormPage() {
  const { t } = useTranslation();
  const { id } = useParams<{ id?: string }>();
  const navigate = useNavigate();
  const [form] = Form.useForm<CreateCustomerData>();

  const isEdit = !!id;
  const customerId = isEdit ? Number(id) : 0;

  const { data: customer, isLoading: loadingCustomer } = useCustomer(customerId);
  const createMutation = useCreateCustomer();
  const updateMutation = useUpdateCustomer(customerId);

  useEffect(() => {
    if (isEdit && customer) {
      form.setFieldsValue({
        customer_code: customer.customer_code,
        name: customer.name,
        contact_person: customer.contact_person ?? undefined,
        phone: customer.phone ?? undefined,
        email: customer.email ?? undefined,
        address: customer.address ?? undefined,
        notes: customer.notes ?? undefined,
      });
    }
  }, [isEdit, customer, form]);

  const handleSubmit = async (values: CreateCustomerData) => {
    try {
      if (isEdit) {
        await updateMutation.mutateAsync(values);
      } else {
        await createMutation.mutateAsync(values);
      }
      message.success(t('common.operate_success'));
      navigate('/customers');
    } catch {
      message.error(t('common.operate_failed'));
    }
  };

  if (isEdit && loadingCustomer) {
    return <div>{t('common.loading')}</div>;
  }

  return (
    <div>
      <h2 style={{ marginBottom: 24 }}>
        {isEdit ? t('common.edit') : t('common.create')} {t('customers.name')}
      </h2>
      <Form
        form={form}
        layout="vertical"
        onFinish={handleSubmit}
        style={{ maxWidth: 800 }}
      >
        <Form.Item
          label={t('customers.code')}
          name="customer_code"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <Input disabled={isEdit} placeholder={t('common.required')} />
        </Form.Item>

        <Form.Item
          label={t('customers.name')}
          name="name"
          rules={[{ required: true, message: t('common.required') }]}
        >
          <Input />
        </Form.Item>

        <Form.Item label={t('customers.contact_person')} name="contact_person">
          <Input />
        </Form.Item>

        <Form.Item label={t('customers.phone')} name="phone">
          <Input />
        </Form.Item>

        <Form.Item label={t('customers.email')} name="email">
          <Input />
        </Form.Item>

        <Form.Item label={t('customers.address')} name="address">
          <Input.TextArea rows={2} />
        </Form.Item>

        <Form.Item label={t('customers.notes')} name="notes">
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
            <Button onClick={() => navigate('/customers')}>
              {t('common.cancel')}
            </Button>
          </Space>
        </Form.Item>
      </Form>
    </div>
  );
}
