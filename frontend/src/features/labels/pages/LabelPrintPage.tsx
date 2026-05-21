import { useState } from 'react';
import { Card, Form, Input, InputNumber, Select, Button, Row, Col, Typography, Divider, message } from 'antd';
import { PrinterOutlined, BarcodeOutlined, FileTextOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { usePipeLabel, useCreateBatchLabels, useQualityLabel, useCreateShippingLabel } from '../hooks/useLabels';

const { Title } = Typography;
const { Option } = Select;

export default function LabelPrintPage() {
  const { t } = useTranslation();
  const [pipeType, setPipeType] = useState<string>('seamless');
  const [pipeId, setPipeId] = useState<number | undefined>();
  const [certId, setCertId] = useState<number | undefined>();
  const [batchPipeIds, setBatchPipeIds] = useState<string>('');
  const [batchPipeType, setBatchPipeType] = useState<string>('seamless');
  const [orderType, setOrderType] = useState<string>('purchase');
  const [orderId, setOrderId] = useState<number | undefined>();

  const { data: pipeLabel, isLoading: pipeLabelLoading } = usePipeLabel(pipeType, pipeId ?? 0);
  const batchMutation = useCreateBatchLabels();
  const { data: qualityLabel, isLoading: qualityLabelLoading } = useQualityLabel(certId ?? 0);
  const shippingMutation = useCreateShippingLabel();

  const handlePrintPipeLabel = () => {
    if (!pipeLabel) return;
    message.success(t('Pipe label ready for printing'));
  };

  const handleBatchPrint = () => {
    const ids = batchPipeIds
      .split(',')
      .map((s) => s.trim())
      .filter(Boolean)
      .map(Number);
    if (ids.length === 0) {
      message.warning(t('Please enter pipe IDs'));
      return;
    }
    batchMutation.mutate(
      { pipe_ids: ids, pipe_type: batchPipeType },
      { onSuccess: () => message.success(t('Batch labels created')) },
    );
  };

  const handlePrintQualityLabel = () => {
    if (!qualityLabel) return;
    message.success(t('Quality label ready for printing'));
  };

  const handlePrintShippingLabel = () => {
    if (!orderId) {
      message.warning(t('Please enter order ID'));
      return;
    }
    shippingMutation.mutate(
      { order_type: orderType, order_id: orderId },
      { onSuccess: () => message.success(t('Shipping label created')) },
    );
  };

  return (
    <div>
      <Title level={3}>{t('Label Printing')}</Title>
      <Row gutter={[16, 16]}>
        <Col xs={24} lg={12}>
          <Card title={<><BarcodeOutlined /> {t('Pipe Label')}</>}>
            <Form layout="vertical">
              <Form.Item label={t('Pipe Type')}>
                <Select value={pipeType} onChange={setPipeType}>
                  <Option value="seamless">Seamless</Option>
                  <Option value="screen">Screen</Option>
                </Select>
              </Form.Item>
              <Form.Item label={t('Pipe ID')}>
                <InputNumber
                  style={{ width: '100%' }}
                  value={pipeId}
                  onChange={(v) => setPipeId(v ?? undefined)}
                  min={1}
                />
              </Form.Item>
              <Button
                type="primary"
                icon={<PrinterOutlined />}
                loading={pipeLabelLoading}
                disabled={!pipeId}
                onClick={handlePrintPipeLabel}
              >
                {t('Print Label')}
              </Button>
            </Form>
          </Card>
        </Col>
        <Col xs={24} lg={12}>
          <Card title={<><BarcodeOutlined /> {t('Batch Labels')}</>}>
            <Form layout="vertical">
              <Form.Item label={t('Pipe Type')}>
                <Select value={batchPipeType} onChange={setBatchPipeType}>
                  <Option value="seamless">Seamless</Option>
                  <Option value="screen">Screen</Option>
                </Select>
              </Form.Item>
              <Form.Item label={t('Pipe IDs (comma separated)')}>
                <Input
                  value={batchPipeIds}
                  onChange={(e) => setBatchPipeIds(e.target.value)}
                  placeholder="1, 2, 3, ..."
                />
              </Form.Item>
              <Button
                type="primary"
                icon={<PrinterOutlined />}
                loading={batchMutation.isPending}
                onClick={handleBatchPrint}
              >
                {t('Batch Print')}
              </Button>
            </Form>
          </Card>
        </Col>
      </Row>
      <Divider />
      <Row gutter={[16, 16]}>
        <Col xs={24} lg={12}>
          <Card title={<><FileTextOutlined /> {t('Quality Certificate Label')}</>}>
            <Form layout="vertical">
              <Form.Item label={t('Certificate ID')}>
                <InputNumber
                  style={{ width: '100%' }}
                  value={certId}
                  onChange={(v) => setCertId(v ?? undefined)}
                  min={1}
                />
              </Form.Item>
              <Button
                type="primary"
                icon={<PrinterOutlined />}
                loading={qualityLabelLoading}
                disabled={!certId}
                onClick={handlePrintQualityLabel}
              >
                {t('Print Label')}
              </Button>
            </Form>
          </Card>
        </Col>
        <Col xs={24} lg={12}>
          <Card title={<><FileTextOutlined /> {t('Shipping Label')}</>}>
            <Form layout="vertical">
              <Form.Item label={t('Order Type')}>
                <Select value={orderType} onChange={setOrderType}>
                  <Option value="purchase">Purchase</Option>
                  <Option value="sales">Sales</Option>
                  <Option value="return">Return</Option>
                  <Option value="transfer">Transfer</Option>
                </Select>
              </Form.Item>
              <Form.Item label={t('Order ID')}>
                <InputNumber
                  style={{ width: '100%' }}
                  value={orderId}
                  onChange={(v) => setOrderId(v ?? undefined)}
                  min={1}
                />
              </Form.Item>
              <Button
                type="primary"
                icon={<PrinterOutlined />}
                loading={shippingMutation.isPending}
                disabled={!orderId}
                onClick={handlePrintShippingLabel}
              >
                {t('Print Label')}
              </Button>
            </Form>
          </Card>
        </Col>
      </Row>
    </div>
  );
}
