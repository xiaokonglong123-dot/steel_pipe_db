// Contract detail page — overview + items table + payment records + status transitions
import { useState } from 'react';
import {
  Card,
  Descriptions,
  Table,
  Tag,
  Button,
  Space,
  Select,
  Modal,
  InputNumber,
  DatePicker,
  Form,
  Input,
  Popconfirm,
  message,
} from 'antd';
import { PlusOutlined } from '@ant-design/icons';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import {
  useContract,
  useUpdateContractStatus,
  useDeleteContractItem,
  useCreateContractPayment,
  useDeleteContractPayment,
} from '../hooks/useContracts';
import type { ContractItem, ContractPayment } from '../types';

const statusColors: Record<string, string> = {
  draft: 'default',
  active: 'processing',
  completed: 'success',
  terminated: 'error',
};

const nextStatuses: Record<string, string[]> = {
  draft: ['active'],
  active: ['completed', 'terminated'],
  completed: [],
  terminated: [],
};

export default function ContractDetailPage() {
  const { t } = useTranslation();
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const contractId = Number(id);

  const { data: contract, isLoading } = useContract(contractId);
  const updateStatusMutation = useUpdateContractStatus(contractId);
  const deleteItemMutation = useDeleteContractItem(contractId);
  const createPaymentMutation = useCreateContractPayment(contractId);
  const deletePaymentMutation = useDeleteContractPayment(contractId);

  const [paymentModalOpen, setPaymentModalOpen] = useState(false);
  const [paymentForm] = Form.useForm();

  const handleStatusUpdate = async (status: string) => {
    try {
      await updateStatusMutation.mutateAsync(status);
      message.success(t('common.updateSuccess'));
    } catch {
      message.error(t('common.operationFailed'));
    }
  };

  const handleAddPayment = async (values: {
    payment_date: string;
    amount: number;
    payment_method?: string;
    reference_number?: string;
    notes?: string;
  }) => {
    try {
      await createPaymentMutation.mutateAsync(values);
      message.success(t('common.createSuccess'));
      setPaymentModalOpen(false);
      paymentForm.resetFields();
    } catch {
      message.error(t('common.operationFailed'));
    }
  };

  const itemColumns = [
    { title: t('contracts.pipe_type'), dataIndex: 'pipe_type', key: 'pipe_type', render: (v: string) => <Tag>{v}</Tag> },
    { title: t('contracts.grade'), dataIndex: 'grade', key: 'grade' },
    { title: t('contracts.od'), dataIndex: 'od', key: 'od' },
    { title: t('contracts.wt'), dataIndex: 'wt', key: 'wt' },
    { title: t('contracts.quantity'), dataIndex: 'quantity', key: 'quantity' },
    { title: t('contracts.unit_price'), dataIndex: 'unit_price', key: 'unit_price', render: (v: number) => v?.toLocaleString() },
    { title: t('contracts.total_price'), dataIndex: 'total_price', key: 'total_price', render: (v: number) => v?.toLocaleString() },
    {
      key: 'actions',
      render: (_: unknown, record: ContractItem) => (
        <Popconfirm title={t('common.confirm_delete')} onConfirm={() => deleteItemMutation.mutate(record.id)}>
          <Button type="link" danger size="small">{t('common.delete')}</Button>
        </Popconfirm>
      ),
    },
  ];

  const paymentColumns = [
    { title: t('contracts.payment_date'), dataIndex: 'payment_date', key: 'payment_date' },
    { title: t('contracts.amount'), dataIndex: 'amount', key: 'amount', render: (v: number) => v?.toLocaleString() },
    { title: t('contracts.payment_method'), dataIndex: 'payment_method', key: 'payment_method' },
    { title: t('contracts.reference_number'), dataIndex: 'reference_number', key: 'reference_number' },
    {
      key: 'actions',
      render: (_: unknown, record: ContractPayment) => (
        <Popconfirm title={t('common.confirm_delete')} onConfirm={() => deletePaymentMutation.mutate(record.id)}>
          <Button type="link" danger size="small">{t('common.delete')}</Button>
        </Popconfirm>
      ),
    },
  ];

  if (isLoading) return null;

  if (!contract) return <div>{t('common.notFound')}</div>;

  const availableStatuses = nextStatuses[contract.status] || [];

  return (
    <div>
      <Card
        title={`${t('contracts.contract')} #${contract.contract_number}`}
        extra={
          <Space>
            <Button onClick={() => navigate(`/contracts/${contract.id}/edit`)}>
              {t('common.edit')}
            </Button>
            {availableStatuses.length > 0 && (
              <Select
                placeholder={t('contracts.update_status')}
                style={{ width: 150 }}
                onChange={handleStatusUpdate}
                options={availableStatuses.map((s) => ({ label: s, value: s }))}
              />
            )}
          </Space>
        }
        style={{ marginBottom: 16 }}
      >
        <Descriptions bordered column={2}>
          <Descriptions.Item label={t('contracts.contract_name')}>{contract.contract_name}</Descriptions.Item>
          <Descriptions.Item label={t('contracts.contract_type')}>
            <Tag color={contract.contract_type === 'purchase' ? 'blue' : 'green'}>{contract.contract_type}</Tag>
          </Descriptions.Item>
          <Descriptions.Item label={t('contracts.party_a')}>{contract.party_a}</Descriptions.Item>
          <Descriptions.Item label={t('contracts.party_b')}>{contract.party_b}</Descriptions.Item>
          <Descriptions.Item label={t('contracts.sign_date')}>{contract.sign_date}</Descriptions.Item>
          <Descriptions.Item label={t('contracts.status')}>
            <Tag color={statusColors[contract.status]}>{contract.status}</Tag>
          </Descriptions.Item>
          <Descriptions.Item label={t('contracts.start_date')}>{contract.start_date}</Descriptions.Item>
          <Descriptions.Item label={t('contracts.end_date')}>{contract.end_date}</Descriptions.Item>
          <Descriptions.Item label={t('contracts.total_amount')} span={2}>
            {contract.total_amount?.toLocaleString()}
          </Descriptions.Item>
          <Descriptions.Item label={t('contracts.paid_amount')} span={2}>
            {contract.paid_amount?.toLocaleString()}
          </Descriptions.Item>
          <Descriptions.Item label={t('common.notes')} span={2}>
            {contract.notes}
          </Descriptions.Item>
        </Descriptions>
      </Card>

      <Card
        title={t('contracts.contract_items')}
        style={{ marginBottom: 16 }}
      >
        <Table
          columns={itemColumns}
          dataSource={contract.items}
          rowKey="id"
          pagination={false}
          bordered
          size="small"
        />
      </Card>

      <Card
        title={t('contracts.payments')}
        extra={
          <Button
            type="primary"
            icon={<PlusOutlined />}
            onClick={() => setPaymentModalOpen(true)}
          >
            {t('contracts.add_payment')}
          </Button>
        }
      >
        <Table
          columns={paymentColumns}
          dataSource={contract.payments}
          rowKey="id"
          pagination={false}
          bordered
          size="small"
        />
      </Card>

      <Modal
        title={t('contracts.add_payment')}
        open={paymentModalOpen}
        onCancel={() => setPaymentModalOpen(false)}
        onOk={() => paymentForm.submit()}
        confirmLoading={createPaymentMutation.isPending}
      >
        <Form
          form={paymentForm}
          layout="vertical"
          onFinish={handleAddPayment}
          initialValues={{ payment_method: 'bank_transfer' }}
        >
          <Form.Item
            label={t('contracts.payment_date')}
            name="payment_date"
            rules={[{ required: true }]}
          >
            <DatePicker style={{ width: '100%' }} />
          </Form.Item>
          <Form.Item
            label={t('contracts.amount')}
            name="amount"
            rules={[{ required: true }]}
          >
            <InputNumber min={0} step={0.01} style={{ width: '100%' }} />
          </Form.Item>
          <Form.Item label={t('contracts.payment_method')} name="payment_method">
            <Select
              options={[
                { label: t('contracts.bank_transfer'), value: 'bank_transfer' },
                { label: t('contracts.cash'), value: 'cash' },
                { label: t('contracts.check'), value: 'check' },
                { label: t('contracts.other'), value: 'other' },
              ]}
            />
          </Form.Item>
          <Form.Item label={t('contracts.reference_number')} name="reference_number">
            <Input />
          </Form.Item>
          <Form.Item label={t('common.notes')} name="notes">
            <Input.TextArea rows={2} />
          </Form.Item>
        </Form>
      </Modal>
    </div>
  );
}
