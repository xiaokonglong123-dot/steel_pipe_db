import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Button, Card, Input, Space, Table, Tag, message } from 'antd';
import { DollarOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { hrApi } from '../api/hrApi';
import { PageLayout } from '@/shared/components/PageLayout';

export default function SalaryPage() {
  const { t } = useTranslation('hr');
  const queryClient = useQueryClient();
  const [period, setPeriod] = useState('');

  const { data: salaries, isLoading } = useQuery({
    queryKey: ['hr-salaries', period],
    queryFn: () => hrApi.listSalaries(period || undefined),
  });

  const generate = useMutation({
    mutationFn: hrApi.generateSalaries,
    onSuccess: (items) => {
      message.success(`${t('generated')}: ${items.length}`);
      queryClient.invalidateQueries({ queryKey: ['hr-salaries'] });
    },
  });

  const columns = [
    { title: t('period'), dataIndex: 'period', key: 'period' },
    { title: t('employeeId'), dataIndex: 'employee_id', key: 'employee_id' },
    { title: t('baseSalary'), dataIndex: 'base_salary', key: 'base_salary' },
    { title: t('gross'), dataIndex: 'gross', key: 'gross' },
    { title: t('net'), dataIndex: 'net', key: 'net' },
    { title: t('status'), dataIndex: 'status', key: 'status', render: (v: string) => (
      <Tag color={v === 'paid' ? 'green' : 'default'}>{v}</Tag>
    ) },
  ];

  return (
    <PageLayout title={t('salaryTitle')}>
      <Card>
        <Space style={{ marginBottom: 16 }}>
          <Input
            placeholder={t('periodPlaceholder')}
            value={period}
            onChange={(e) => setPeriod(e.target.value)}
            style={{ width: 160 }}
          />
          <Button
            type="primary"
            icon={<DollarOutlined />}
            loading={generate.isPending}
            onClick={() => {
              const p = period || new Date().toISOString().slice(0, 7);
              setPeriod(p);
              generate.mutate(p);
            }}
          >
            {t('generate')}
          </Button>
        </Space>
        <Table rowKey="id" loading={isLoading} dataSource={salaries ?? []} columns={columns} pagination={false} />
      </Card>
    </PageLayout>
  );
}
