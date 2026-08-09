import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Button, Card, Form, Input, Modal, Select, Space, Table, Tabs, message } from 'antd';
import { PlusOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { financeApi, type Account, type FinanceInvoice } from '../api/financeApi';
import { financeQueryKeys } from '../queryKeys';
import { PageLayout } from '@/shared/components/PageLayout';

export default function FinancePage() {
  const { t } = useTranslation('finance');
  const queryClient = useQueryClient();
  const [form] = Form.useForm();
  const [creating, setCreating] = useState<'account' | 'entry' | 'invoice' | 'payment' | null>(null);

  const invalidate = () => {
    queryClient.invalidateQueries({ queryKey: financeQueryKeys.accounts });
    queryClient.invalidateQueries({ queryKey: financeQueryKeys.entries });
    queryClient.invalidateQueries({ queryKey: financeQueryKeys.invoices });
    queryClient.invalidateQueries({ queryKey: financeQueryKeys.payments });
    queryClient.invalidateQueries({ queryKey: financeQueryKeys.trial });
  };

  const { data: accounts } = useQuery({ queryKey: financeQueryKeys.accounts, queryFn: financeApi.listAccounts });
  const { data: entries } = useQuery({ queryKey: financeQueryKeys.entries, queryFn: financeApi.listEntries });
  const { data: invoices } = useQuery({ queryKey: financeQueryKeys.invoices, queryFn: () => financeApi.listInvoices() });
  const { data: payments } = useQuery({ queryKey: financeQueryKeys.payments, queryFn: financeApi.listPayments });
  const { data: trial } = useQuery({ queryKey: financeQueryKeys.trial, queryFn: financeApi.trialBalance });

  const createAccount = useMutation({
    mutationFn: financeApi.createAccount,
    onSuccess: () => { message.success(t('saved')); invalidate(); setCreating(null); form.resetFields(); },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
  const createEntry = useMutation({
    mutationFn: financeApi.createEntry,
    onSuccess: () => { message.success(t('entryPosted')); invalidate(); setCreating(null); form.resetFields(); },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
  const createInvoice = useMutation({
    mutationFn: financeApi.createInvoice,
    onSuccess: () => { message.success(t('saved')); invalidate(); setCreating(null); form.resetFields(); },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
  const createPayment = useMutation({
    mutationFn: financeApi.createPayment,
    onSuccess: () => { message.success(t('saved')); invalidate(); setCreating(null); form.resetFields(); },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });
  const confirmInvoice = useMutation({
    mutationFn: financeApi.confirmInvoice,
    onSuccess: () => { message.success(t('confirmed')); invalidate(); },
    onError: () => { message.error(t('common.operate_failed', '操作失败')); },
  });

  const handleCreate = async () => {
    const v = await form.validateFields();
    if (creating === 'account') createAccount.mutate({ code: v.code, name: v.name, account_type: v.account_type });
    if (creating === 'entry') {
      const account = accounts?.[0];
      createEntry.mutate({
        entry_date: v.entry_date ?? new Date().toISOString().slice(0, 10),
        description: v.description,
        details: [
          { account_id: account?.id ?? 1, debit: Number(v.debit) || 0 },
          { account_id: v.credit_account ?? (accounts?.[1]?.id ?? account?.id ?? 1), credit: Number(v.debit) || 0 },
        ],
      });
    }
    if (creating === 'invoice') createInvoice.mutate({ invoice_type: v.invoice_type, party_id: Number(v.party_id), amount: Number(v.amount), tax_amount: v.tax_amount ? Number(v.tax_amount) : undefined });
    if (creating === 'payment') createPayment.mutate({ invoice_id: v.invoice_id ? Number(v.invoice_id) : undefined, direction: v.direction, amount: Number(v.amount), method: v.method });
  };

  const accountColumns = [
    { title: t('code'), dataIndex: 'code', key: 'code' },
    { title: t('name'), dataIndex: 'name', key: 'name' },
    { title: t('type'), dataIndex: 'account_type', key: 'account_type' },
  ];
  const entryColumns = [
    { title: t('entryNo'), dataIndex: 'entry_no', key: 'entry_no' },
    { title: t('date'), dataIndex: 'entry_date', key: 'entry_date' },
    { title: t('description'), dataIndex: 'description', key: 'description', render: (v: string | null) => v ?? '-' },
    { title: t('status'), dataIndex: 'status', key: 'status' },
  ];
  const invoiceColumns = [
    { title: t('invoiceNo'), dataIndex: 'invoice_no', key: 'invoice_no' },
    { title: t('type'), dataIndex: 'invoice_type', key: 'invoice_type' },
    { title: t('partyId'), dataIndex: 'party_id', key: 'party_id' },
    { title: t('total'), dataIndex: 'total_amount', key: 'total_amount' },
    { title: t('status'), dataIndex: 'status', key: 'status' },
    {
      title: t('actions'), key: 'actions', render: (_: unknown, r: FinanceInvoice) =>
        r.status === 'draft' ? (
          <Button size="small" type="primary" onClick={() => confirmInvoice.mutate(r.id)}>{t('confirm')}</Button>
        ) : null,
    },
  ];
  const paymentColumns = [
    { title: t('paymentNo'), dataIndex: 'payment_no', key: 'payment_no' },
    { title: t('direction'), dataIndex: 'direction', key: 'direction' },
    { title: t('amount'), dataIndex: 'amount', key: 'amount' },
    { title: t('method'), dataIndex: 'method', key: 'method' },
  ];
  const trialColumns = [
    { title: t('code'), dataIndex: 'code', key: 'code' },
    { title: t('name'), dataIndex: 'name', key: 'name' },
    { title: t('debit'), dataIndex: 'debit', key: 'debit' },
    { title: t('credit'), dataIndex: 'credit', key: 'credit' },
  ];

  const tabs = [
    { key: 'accounts', label: t('accounts'), children: (
      <Table rowKey="id" dataSource={accounts ?? []} columns={accountColumns} pagination={false} size="small" />
    ) },
    { key: 'entries', label: t('journalEntries'), children: (
      <Table rowKey="id" dataSource={entries ?? []} columns={entryColumns} pagination={false} size="small" />
    ) },
    { key: 'invoices', label: t('invoices'), children: (
      <Table rowKey="id" dataSource={invoices ?? []} columns={invoiceColumns} pagination={false} size="small" />
    ) },
    { key: 'payments', label: t('payments'), children: (
      <Table rowKey="id" dataSource={payments ?? []} columns={paymentColumns} pagination={false} size="small" />
    ) },
    { key: 'trial', label: t('trialBalance'), children: (
      <Table rowKey="code" dataSource={trial ?? []} columns={trialColumns} pagination={false} size="small" />
    ) },
  ];

  return (
    <PageLayout title={t('title')}>
      <Card>
        <Space style={{ marginBottom: 16 }}>
          <Button type="primary" icon={<PlusOutlined />} onClick={() => setCreating('account')}>{t('newAccount')}</Button>
          <Button icon={<PlusOutlined />} onClick={() => setCreating('entry')}>{t('newEntry')}</Button>
          <Button icon={<PlusOutlined />} onClick={() => setCreating('invoice')}>{t('newInvoice')}</Button>
          <Button icon={<PlusOutlined />} onClick={() => setCreating('payment')}>{t('newPayment')}</Button>
        </Space>
        <Tabs items={tabs} />
      </Card>

      <Modal
        title={t(`new_${creating ?? 'account'}`)}
        open={!!creating}
        onCancel={() => setCreating(null)}
        onOk={handleCreate}
        okText={t('save')}
        cancelText={t('cancel')}
      >
        <Form form={form} layout="vertical">
          {creating === 'account' && (
            <>
              <Form.Item name="code" label={t('code')} rules={[{ required: true }]}><Input /></Form.Item>
              <Form.Item name="name" label={t('name')} rules={[{ required: true }]}><Input /></Form.Item>
              <Form.Item name="account_type" label={t('type')} initialValue="asset">
                <Select options={['asset', 'liability', 'equity', 'revenue', 'expense'].map((v) => ({ value: v, label: v }))} />
              </Form.Item>
            </>
          )}
          {creating === 'entry' && (
            <>
              <Form.Item name="entry_date" label={t('date')}><Input type="date" /></Form.Item>
              <Form.Item name="description" label={t('description')}><Input /></Form.Item>
              <Form.Item name="debit" label={t('debitAmount')} rules={[{ required: true }]}><Input type="number" /></Form.Item>
              <Form.Item name="credit_account" label={t('creditAccount')}>
                <Select options={(accounts ?? []).map((a: Account) => ({ value: a.id, label: `${a.code} ${a.name}` }))} />
              </Form.Item>
            </>
          )}
          {creating === 'invoice' && (
            <>
              <Form.Item name="invoice_type" label={t('type')} initialValue="sales">
                <Select options={[{ value: 'sales', label: t('sales') }, { value: 'purchase', label: t('purchase') }]} />
              </Form.Item>
              <Form.Item name="party_id" label={t('partyId')} rules={[{ required: true }]}><Input type="number" /></Form.Item>
              <Form.Item name="amount" label={t('amount')} rules={[{ required: true }]}><Input type="number" /></Form.Item>
              <Form.Item name="tax_amount" label={t('tax')}><Input type="number" /></Form.Item>
            </>
          )}
          {creating === 'payment' && (
            <>
              <Form.Item name="invoice_id" label={t('invoiceId')}><Input type="number" /></Form.Item>
              <Form.Item name="direction" label={t('direction')} initialValue="in">
                <Select options={[{ value: 'in', label: t('in') }, { value: 'out', label: t('out') }]} />
              </Form.Item>
              <Form.Item name="amount" label={t('amount')} rules={[{ required: true }]}><Input type="number" /></Form.Item>
              <Form.Item name="method" label={t('method')} initialValue="bank_transfer">
                <Select options={['bank_transfer', 'cash', 'check'].map((v) => ({ value: v, label: v }))} />
              </Form.Item>
            </>
          )}
        </Form>
      </Modal>
    </PageLayout>
  );
}
