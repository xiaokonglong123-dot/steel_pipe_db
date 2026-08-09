// 合同新增/编辑表单 — 使用 PageLayout，行项按商品(SKU)选择
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
import { PlusOutlined, DeleteOutlined } from '@ant-design/icons';
import { useNavigate, useParams } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { PageLayout, ItemPicker } from '@/shared/components';
import type { ItemOption } from '@/shared/components';
import { useContract, useCreateContract, useUpdateContract } from '../hooks/useContracts';
import type { ContractItem, CreateContractItemData } from '../types';

interface FormValues {
  title: string;
  contract_type: 'purchase' | 'sales';
  party_a: string;
  party_b: string;
  sign_date?: string;
  start_date?: string;
  end_date?: string;
  total_amount: number;
  notes?: string;
  items: CreateContractItemData[];
}

export default function ContractFormPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { id } = useParams<{ id: string }>();
  const isEdit = !!id;
  const [form] = Form.useForm<FormValues>();
  const [items, setItems] = useState<CreateContractItemData[]>([]);
  const [itemModalOpen, setItemModalOpen] = useState(false);

  const { data: contract } = useContract(Number(id));
  const createMutation = useCreateContract();
  const updateMutation = useUpdateContract(Number(id));

  useEffect(() => {
    if (contract && isEdit) {
      form.setFieldsValue({
        title: contract.title,
        contract_type: contract.contract_type,
        party_a: contract.party_a,
        party_b: contract.party_b,
        sign_date: contract.sign_date ?? undefined,
        start_date: contract.start_date ?? undefined,
        end_date: contract.end_date ?? undefined,
        total_amount: contract.total_amount ?? 0,
        notes: contract.notes ?? undefined,
      });
      if (contract.items && contract.items.length > 0) {
        setItems(
          contract.items.map((item: ContractItem) => ({
            item_id: item.item_id,
            sku: item.sku,
            name: item.name,
            quantity: item.quantity,
            unit_price: item.unit_price ?? 0,
            notes: item.notes ?? undefined,
          })),
        );
      }
    }
  }, [contract, isEdit, form]);

  const handleFinish = async (values: FormValues) => {
    if (items.length === 0) {
      message.warning(t('contracts.items_required', '请至少添加一个商品'));
      return;
    }
    try {
      const payload = {
        ...values,
        items: items.map((it) => ({
          item_id: it.item_id,
          quantity: it.quantity,
          unit_price: it.unit_price ?? 0,
          notes: it.notes ?? null,
        })),
      };
      if (isEdit) {
        await updateMutation.mutateAsync(payload as Parameters<typeof updateMutation.mutateAsync>[0]);
        message.success(t('common.updateSuccess'));
      } else {
        await createMutation.mutateAsync(payload as Parameters<typeof createMutation.mutateAsync>[0]);
        message.success(t('common.createSuccess'));
      }
      navigate('/contracts');
    } catch {
      message.error(t('common.operationFailed'));
    }
  };

  const addItems = (picked: ItemOption[]) => {
    const additions: CreateContractItemData[] = picked.map((it) => ({
      item_id: it.id,
      sku: it.sku,
      name: it.name,
      quantity: 1,
      unit_price: 0,
      notes: undefined,
    }));
    setItems((prev) => [...prev, ...additions]);
    setItemModalOpen(false);
  };

  const removeItem = (index: number) => {
    setItems((prev) => prev.filter((_, i) => i !== index));
  };

  const updateItem = (index: number, field: keyof CreateContractItemData, value: unknown) => {
    setItems((prev) => {
      const next = [...prev];
      next[index] = { ...next[index], [field]: value };
      return next;
    });
  };

  const itemColumns = [
    {
      title: t('contracts.item', '商品'),
      key: 'item',
      render: (_: unknown, record: CreateContractItemData) =>
        record.name ? `${record.sku ?? ''} — ${record.name}` : record.sku || `#${record.item_id}`,
    },
    {
      title: t('contracts.quantity'),
      dataIndex: 'quantity',
      key: 'quantity',
      width: 90,
      render: (_: unknown, record: CreateContractItemData, index: number) => (
        <InputNumber
          value={record.quantity}
          min={1}
          onChange={(v) => updateItem(index, 'quantity', v ?? 1)}
          style={{ width: 80 }}
        />
      ),
    },
    {
      title: t('contracts.unit_price'),
      dataIndex: 'unit_price',
      key: 'unit_price',
      width: 110,
      render: (_: unknown, record: CreateContractItemData, index: number) => (
        <InputNumber
          value={record.unit_price}
          min={0}
          step={0.01}
          onChange={(v) => updateItem(index, 'unit_price', v ?? 0)}
          style={{ width: 110 }}
        />
      ),
    },
    {
      title: t('contracts.total_price'),
      key: 'total_price',
      width: 110,
      render: (_: unknown, record: CreateContractItemData) =>
        ((record.quantity ?? 0) * (record.unit_price ?? 0)).toLocaleString(),
    },
    {
      title: t('contracts.notes'),
      dataIndex: 'notes',
      key: 'notes',
      width: 150,
      render: (_: unknown, record: CreateContractItemData, index: number) => (
        <Input
          value={record.notes ?? ''}
          onChange={(e) => updateItem(index, 'notes', e.target.value)}
        />
      ),
    },
    {
      key: 'actions',
      width: 50,
      render: (_: unknown, __: unknown, index: number) => (
        <Popconfirm title={t('common.confirm_delete')} onConfirm={() => removeItem(index)}>
          <DeleteOutlined style={{ color: '#ff4d4f' }} />
        </Popconfirm>
      ),
    },
  ];

  return (
    <PageLayout
      title={isEdit ? t('contracts.edit_contract') : t('contracts.create_contract')}
      onBack={() => navigate('/contracts')}
    >
      <Form
        form={form}
        layout="vertical"
        onFinish={handleFinish}
        initialValues={{ contract_type: 'purchase' }}
        style={{ maxWidth: 960 }}
      >
        <Space style={{ display: 'flex' }} wrap>
            {isEdit && (
              <Form.Item label={t('contracts.contract_number')}>
                <Input value={contract?.contract_no} disabled />
              </Form.Item>
            )}
            <Form.Item
              label={t('contracts.contract_name')}
              name="title"
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

        <Card
          title={t('contracts.contract_items')}
          extra={
            <Button type="dashed" icon={<PlusOutlined />} onClick={() => setItemModalOpen(true)}>
              {t('contracts.add_item')}
            </Button>
          }
          style={{ marginBottom: 16 }}
        >
          <Table
            columns={itemColumns}
            dataSource={items.map((item, i) => ({ ...item, key: i }))}
            rowKey="key"
            pagination={false}
            bordered
            size="small"
          />
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

      <ItemPicker
        open={itemModalOpen}
        onCancel={() => setItemModalOpen(false)}
        onSelect={addItems}
      />
    </PageLayout>
  );
}
