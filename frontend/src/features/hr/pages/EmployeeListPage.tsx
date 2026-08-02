import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Button, Card, Form, Input, Modal, Popconfirm, Space, Table, Tag, message } from 'antd';
import { PlusOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { hrApi, type HrEmployee } from '../api/hrApi';
import { PageLayout } from '@/shared/components/PageLayout';
import { SearchBar } from '@/shared/components/SearchBar';

export default function EmployeeListPage() {
  const { t } = useTranslation('hr');
  const queryClient = useQueryClient();
  const [page, setPage] = useState(1);
  const [keyword, setKeyword] = useState('');
  const [creating, setCreating] = useState(false);
  const [form] = Form.useForm();

  const { data, isLoading } = useQuery({
    queryKey: ['hr-employees', page, keyword],
    queryFn: () => hrApi.listEmployees({ page, page_size: 20, q: keyword || undefined }),
  });

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ['hr-employees'] });

  const createEmp = useMutation({
    mutationFn: hrApi.createEmployee,
    onSuccess: () => {
      message.success(t('saved'));
      invalidate();
      setCreating(false);
      form.resetFields();
    },
  });

  const terminate = useMutation({
    mutationFn: ({ id, reason }: { id: number; reason?: string }) => hrApi.terminateEmployee(id, reason),
    onSuccess: () => {
      message.success(t('terminated'));
      invalidate();
    },
  });

  const handleSubmit = async () => {
    const values = await form.validateFields();
    createEmp.mutate({
      employee_no: values.employee_no,
      name: values.name,
      phone: values.phone,
      hire_date: values.hire_date.format?.('YYYY-MM-DD') ?? values.hire_date,
      base_salary: values.base_salary ? Number(values.base_salary) : undefined,
    });
  };

  const columns = [
    { title: t('employeeNo'), dataIndex: 'employee_no', key: 'employee_no' },
    { title: t('name'), dataIndex: 'name', key: 'name' },
    { title: t('phone'), dataIndex: 'phone', key: 'phone', render: (v: string | null) => v ?? '-' },
    { title: t('hireDate'), dataIndex: 'hire_date', key: 'hire_date' },
    { title: t('probationEnd'), dataIndex: 'probation_end', key: 'probation_end', render: (v: string | null) => v ?? '-' },
    { title: t('baseSalary'), dataIndex: 'base_salary', key: 'base_salary', render: (v: string | null) => v ?? '-' },
    { title: t('status'), dataIndex: 'status', key: 'status', render: (v: string) => (
      <Tag color={v === 'active' ? 'green' : v === 'on_leave' ? 'orange' : 'red'}>{v}</Tag>
    ) },
    { title: t('actions'), key: 'actions', render: (_: unknown, r: HrEmployee) => (
      <Popconfirm title={t('confirmTerminate')} onConfirm={() => terminate.mutate({ id: r.id })} disabled={r.status !== 'active'}>
        <Button size="small" danger disabled={r.status !== 'active'}>{t('terminate')}</Button>
      </Popconfirm>
    ) },
  ];

  return (
    <PageLayout title={t('title')}>
      <Card>
        <Space style={{ marginBottom: 16, width: '100%' }} direction="vertical">
          <Space style={{ display: 'flex', justifyContent: 'space-between' }}>
            <SearchBar onSearch={(v) => { setKeyword(v); setPage(1); }} placeholder={t('searchPlaceholder')} />
            <Button type="primary" icon={<PlusOutlined />} onClick={() => setCreating(true)}>
              {t('create')}
            </Button>
          </Space>
          <Table
            rowKey="id"
            loading={isLoading}
            dataSource={data?.items ?? []}
            columns={columns}
            pagination={{
              current: page,
              pageSize: 20,
              total: data?.total ?? 0,
              onChange: (p) => setPage(p),
              showTotal: (total) => `${t('total')}: ${total}`,
            }}
          />
        </Space>
      </Card>

      <Modal title={t('create')} open={creating} onCancel={() => setCreating(false)} footer={null}>
        <Form form={form} layout="vertical" onFinish={handleSubmit}>
          <Form.Item name="employee_no" label={t('employeeNo')} rules={[{ required: true, max: 50 }]}>
            <Input />
          </Form.Item>
          <Form.Item name="name" label={t('name')} rules={[{ required: true, max: 100 }]}>
            <Input />
          </Form.Item>
          <Form.Item name="phone" label={t('phone')}>
            <Input />
          </Form.Item>
          <Form.Item name="hire_date" label={t('hireDate')} rules={[{ required: true }]}>
            <Input type="date" />
          </Form.Item>
          <Form.Item name="base_salary" label={t('baseSalary')}>
            <Input type="number" />
          </Form.Item>
          <Space>
            <Button type="primary" htmlType="submit" loading={createEmp.isPending}>{t('save')}</Button>
            <Button onClick={() => setCreating(false)}>{t('cancel')}</Button>
          </Space>
        </Form>
      </Modal>
    </PageLayout>
  );
}
