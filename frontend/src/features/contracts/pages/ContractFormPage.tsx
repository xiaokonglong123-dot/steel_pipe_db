// Contract create/edit form — contract fields + dynamic line items + payment milestones
import { useEffect, useState } from 'react';
import {
  Form,
  Input,
  InputNumber,
  Select,
  DatePicker,
  Button,
  Table,
  Space,
  Popconfirm,
  Card,
  message,
} from 'antd';
import { PlusOutlined, MinusCircleOutlined } from '@ant-design/icons';
import { useNavigate, useParams } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useContract, useCreateContract, useUpdateContract } from '../hooks/useContracts';
import type { ContractItem } from '../types';

interface ItemFormValue {
  pipe_type: 'seamless' | 'screen';
  grade: string;
  od: number;
  wt: number;
  length?: number;
  quantity: number;
  unit_price: number;
  total_price: number;
  delivery_date?: string;
  notes?: string;
}

interface FormValues {
  contract_name: string;
  contract_type: 'purchase' | 'sales';
  party_a: string;
  party_b: string;
  sign_date?: string;
  start_date?: string;
  end_date?: string;
  total_amount: number;
  notes?: string;
  items: ItemFormValue[];
}

export default function ContractFormPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { id } = useParams<{ id: string }>();
  const isEdit = !!id;
  const [form] = Form.useForm<FormValues>();
  const [items, setItems] = useState<ItemFormValue[]>([]);

  const { data: contract } = useContract(Number(id));
  const createMutation = useCreateContract();
  const updateMutation = useUpdateContract(Number(id));

  useEffect(() => {
    if (contract && isEdit) {
      form.setFieldsValue({
        contract_name: contract.contract_name,
        contract_type: contract.contract_type,
        party_a: contract.party_a,
        party_b: contract.party_b,
        sign_date: contract.sign_date,
        start_date: contract.start_date,
        end_date: contract.end_date,
        total_amount: contract.total_amount,
        notes: contract.notes,
      });
      if (contract.items) {
        setItems(
          contract.items.map((item: ContractItem) => ({
            pipe_type: item.pipe_type,
            grade: item.grade,
            od: item.od,
            wt: item.wt,
            length: item.length,
            quantity: item.quantity,
            unit_price: item.unit_price,
            total_price: item.total_price,
            delivery_date: item.delivery_date,
            notes: item.notes,
          })),
        );
      }
    }
  }, [contract, isEdit, form]);

  const handleFinish = async (values: FormValues) => {
    try {
      const payload = { ...values, items };
      if (isEdit) {
        await updateMutation.mutateAsync(payload);
        message.success(t('common.updateSuccess'));
      } else {
        await createMutation.mutateAsync(payload);
        message.success(t('common.createSuccess'));
      }
      navigate('/contracts');
    } catch {
      message.error(t('common.operationFailed'));
    }
  };

  const addItem = () => {
    setItems((prev) => [
      ...prev,
      {
        pipe_type: 'seamless',
        grade: '',
        od: 0,
        wt: 0,
        quantity: 1,
        unit_price: 0,
        total_price: 0,
      },
    ]);
  };

  const removeItem = (index: number) => {
    setItems((prev) => prev.filter((_, i) => i !== index));
  };

  const updateItem = (index: number, field: keyof ItemFormValue, value: unknown) => {
    setItems((prev) => {
      const next = [...prev];
      next[index] = { ...next[index], [field]: value };
      const item = next[index];
      if (field === 'quantity' || field === 'unit_price') {
        item.total_price = item.quantity * item.unit_price;
      }
      return next;
    });
  };

  const itemColumns = [
    {
      title: t('contracts.pipe_type'),
      dataIndex: 'pipe_type',
      key: 'pipe_type',
      width: 100,
      render: (_: unknown, __: unknown, index: number) => (
        <Select
          value={items[index]?.pipe_type}
          onChange={(v) => updateItem(index, 'pipe_type', v)}
          style={{ width: 100 }}
          options={[
            { label: t('labels.pipe_type.seamless'), value: 'seamless' },
            { label: t('labels.pipe_type.screen'), value: 'screen' },
          ]}
        />
      ),
    },
    {
      title: t('contracts.grade'),
      dataIndex: 'grade',
      key: 'grade',
      width: 100,
      render: (_: unknown, __: unknown, index: number) => (
        <Input
          value={items[index]?.grade}
          onChange={(e) => updateItem(index, 'grade', e.target.value)}
          style={{ width: 80 }}
        />
      ),
    },
    {
      title: t('contracts.od'),
      dataIndex: 'od',
      key: 'od',
      width: 80,
      render: (_: unknown, __: unknown, index: number) => (
        <InputNumber
          value={items[index]?.od}
          onChange={(v) => updateItem(index, 'od', v ?? 0)}
          style={{ width: 80 }}
        />
      ),
    },
    {
      title: t('contracts.wt'),
      dataIndex: 'wt',
      key: 'wt',
      width: 80,
      render: (_: unknown, __: unknown, index: number) => (
        <InputNumber
          value={items[index]?.wt}
          onChange={(v) => updateItem(index, 'wt', v ?? 0)}
          style={{ width: 80 }}
        />
      ),
    },
    {
      title: t('contracts.quantity'),
      dataIndex: 'quantity',
      key: 'quantity',
      width: 80,
      render: (_: unknown, __: unknown, index: number) => (
        <InputNumber
          value={items[index]?.quantity}
          min={1}
          onChange={(v) => updateItem(index, 'quantity', v ?? 1)}
          style={{ width: 70 }}
        />
      ),
    },
    {
      title: t('contracts.unit_price'),
      dataIndex: 'unit_price',
      key: 'unit_price',
      width: 100,
      render: (_: unknown, __: unknown, index: number) => (
        <InputNumber
          value={items[index]?.unit_price}
          min={0}
          step={0.01}
          onChange={(v) => updateItem(index, 'unit_price', v ?? 0)}
          style={{ width: 100 }}
        />
      ),
    },
    {
      title: t('contracts.total_price'),
      dataIndex: 'total_price',
      key: 'total_price',
      width: 100,
      render: (_: unknown, __: unknown, index: number) => (
        <InputNumber value={items[index]?.total_price} disabled style={{ width: 100 }} />
      ),
    },
    {
      key: 'actions',
      width: 50,
      render: (_: unknown, __: unknown, index: number) => (
        <Popconfirm title={t('common.confirm_delete')} onConfirm={() => removeItem(index)}>
          <MinusCircleOutlined style={{ color: '#ff4d4f' }} />
        </Popconfirm>
      ),
    },
  ];

  return (
    <div style={{ maxWidth: 960 }}>
      <Form
        form={form}
        layout="vertical"
        onFinish={handleFinish}
        initialValues={{ contract_type: 'purchase' }}
      >
        <Card title={isEdit ? t('contracts.edit_contract') : t('contracts.create_contract')} style={{ marginBottom: 16 }}>
          <Space style={{ display: 'flex' }} wrap>
            {isEdit && (
              <Form.Item label={t('contracts.contract_number')}>
                <Input value={contract?.contract_number} disabled />
              </Form.Item>
            )}
            <Form.Item
              label={t('contracts.contract_name')}
              name="contract_name"
              rules={[{ required: true }]}
            >
              <Input style={{ width: 250 }} />
            </Form.Item>
            <Form.Item
              label={t('contracts.contract_type')}
              name="contract_type"
              rules={[{ required: true }]}
            >
              <Select
                style={{ width: 140 }}
                options={[
                  { label: t('contracts.purchase'), value: 'purchase' },
                  { label: t('contracts.sales'), value: 'sales' },
                ]}
              />
            </Form.Item>
            <Form.Item
              label={t('contracts.party_a')}
              name="party_a"
              rules={[{ required: true }]}
            >
              <Input style={{ width: 200 }} />
            </Form.Item>
            <Form.Item
              label={t('contracts.party_b')}
              name="party_b"
              rules={[{ required: true }]}
            >
              <Input style={{ width: 200 }} />
            </Form.Item>
            <Form.Item label={t('contracts.sign_date')} name="sign_date">
              <DatePicker />
            </Form.Item>
            <Form.Item label={t('contracts.start_date')} name="start_date">
              <DatePicker />
            </Form.Item>
            <Form.Item label={t('contracts.end_date')} name="end_date">
              <DatePicker />
            </Form.Item>
            <Form.Item
              label={t('contracts.total_amount')}
              name="total_amount"
              rules={[{ required: true }]}
            >
              <InputNumber min={0} step={0.01} style={{ width: 200 }} />
            </Form.Item>
            <Form.Item label={t('common.notes')} name="notes">
              <Input.TextArea rows={2} style={{ width: 300 }} />
            </Form.Item>
          </Space>
        </Card>

        <Card title={t('contracts.contract_items')} style={{ marginBottom: 16 }}>
          <Table
            columns={itemColumns}
            dataSource={items.map((_, i) => ({ key: i }))}
            rowKey="key"
            pagination={false}
            bordered
            size="small"
          />
          <Button
            type="dashed"
            icon={<PlusOutlined />}
            onClick={addItem}
            style={{ width: '100%', marginTop: 8 }}
          >
            {t('contracts.add_item')}
          </Button>
        </Card>

        <Form.Item>
          <Space>
            <Button type="primary" htmlType="submit" loading={createMutation.isPending || updateMutation.isPending}>
              {isEdit ? t('common.save') : t('common.create')}
            </Button>
            <Button onClick={() => navigate('/contracts')}>
              {t('common.cancel')}
            </Button>
          </Space>
        </Form.Item>
      </Form>
    </div>
  );
}
