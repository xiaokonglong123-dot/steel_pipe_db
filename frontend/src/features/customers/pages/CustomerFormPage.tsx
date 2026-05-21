import { useEffect } from 'react';
import { Form, Input, Select, Button, Space, message } from 'antd';
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
        code: customer.code,
        name: customer.name,
        contact_person: customer.contact_person,
        phone: customer.phone,
        email: customer.email,
        address: customer.address,
        tax_id: customer.tax_id,
        bank_info: customer.bank_info,
        industry: customer.industry,
        status: customer.status,
        notes: customer.notes,
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
          name="code"
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

        <Form.Item label={t('customers.tax_id')} name="tax_id">
          <Input />
        </Form.Item>

        <Form.Item label={t('customers.bank_info')} name="bank_info">
          <Input />
        </Form.Item>

        <Form.Item label={t('customers.industry')} name="industry">
          <Input />
        </Form.Item>

        <Form.Item label={t('customers.status')} name="status">
          <Select>
            <Select.Option value="active">{t('customers.status_active')}</Select.Option>
            <Select.Option value="inactive">{t('customers.status_inactive')}</Select.Option>
          </Select>
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
