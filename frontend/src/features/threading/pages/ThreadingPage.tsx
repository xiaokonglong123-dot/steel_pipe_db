import { useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import { Button, Card, Descriptions, Form, Input, Select, Space, Tag } from 'antd';
import { CalculatorOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { threadingApi, type CalcResult, type DesignCheckOutput } from '../api/threadingApi';
import { PageLayout } from '@/shared/components/PageLayout';

export default function ThreadingPage() {
  const { t } = useTranslation('threading');
  const [form] = Form.useForm();
  const [calcResult, setCalcResult] = useState<CalcResult | null>(null);
  const [designResult, setDesignResult] = useState<DesignCheckOutput | null>(null);

  const calc = useMutation({
    mutationFn: threadingApi.calc,
    onSuccess: (r: CalcResult) => setCalcResult(r),
  });
  const design = useMutation({
    mutationFn: threadingApi.designCheck,
    onSuccess: (r: DesignCheckOutput) => setDesignResult(r),
  });

  const handleCalc = async () => {
    const v = await form.validateFields();
    const payload = {
      od: Number(v.od), wt: Number(v.wt), grade: v.grade, connection_type: v.connection_type,
    };
    calc.mutate(payload);
    design.mutate({ ...payload, depth: Number(v.depth) || 500, fluid_density: 1025 });
  };

  return (
    <PageLayout title={t('title')}>
      <Card>
        <Form form={form} layout="inline" initialValues={{ grade: 'N80', connection_type: 'premium', od: 244.5, wt: 11.05, depth: 500 }}>
          <Form.Item name="od" label={t('od')}><Input type="number" style={{ width: 100 }} /></Form.Item>
          <Form.Item name="wt" label={t('wt')}><Input type="number" style={{ width: 100 }} /></Form.Item>
          <Form.Item name="grade" label={t('grade')}>
            <Select style={{ width: 110 }} options={['J55', 'N80', 'L80', 'P110', 'Q125'].map((v) => ({ value: v, label: v }))} />
          </Form.Item>
          <Form.Item name="connection_type" label={t('connectionType')}>
            <Select style={{ width: 120 }} options={['round', 'buttress', 'premium'].map((v) => ({ value: v, label: v }))} />
          </Form.Item>
          <Form.Item name="depth" label={t('depth')}><Input type="number" style={{ width: 100 }} /></Form.Item>
          <Button type="primary" icon={<CalculatorOutlined />} loading={calc.isPending} onClick={handleCalc}>
            {t('calc')}
          </Button>
        </Form>
      </Card>

      {calcResult && (
        <Card title={t('calcResult')} style={{ marginTop: 16 }}>
          <Descriptions column={2} size="small">
            <Descriptions.Item label={t('burst')}>{Math.round(calcResult.burst_pressure)} psi</Descriptions.Item>
            <Descriptions.Item label={t('collapse')}>{Math.round(calcResult.collapse_pressure)} psi</Descriptions.Item>
            <Descriptions.Item label={t('tension')}>{Math.round(calcResult.tension_capacity)} lbs</Descriptions.Item>
            <Descriptions.Item label={t('jointEff')}>{calcResult.joint_efficiency}</Descriptions.Item>
          </Descriptions>
        </Card>
      )}

      {designResult && (
        <Card title={t('designCheck')} style={{ marginTop: 16 }}>
          <Space direction="vertical">
            <Tag color={designResult.verdict === 'safe' ? 'green' : 'red'}>
              {designResult.verdict === 'safe' ? t('safe') : t('unsafe')}
            </Tag>
            <Descriptions column={2} size="small">
              <Descriptions.Item label={t('externalPressure')}>{Math.round(designResult.external_pressure_psi)} psi</Descriptions.Item>
              <Descriptions.Item label={`${t('burst')} SF`}>{designResult.burst_safety_factor.toFixed(2)}</Descriptions.Item>
              <Descriptions.Item label={`${t('collapse')} SF`}>{designResult.collapse_safety_factor.toFixed(2)}</Descriptions.Item>
              <Descriptions.Item label={`${t('tension')} SF`}>{designResult.tension_safety_factor.toFixed(2)}</Descriptions.Item>
            </Descriptions>
          </Space>
        </Card>
      )}
    </PageLayout>
  );
}
