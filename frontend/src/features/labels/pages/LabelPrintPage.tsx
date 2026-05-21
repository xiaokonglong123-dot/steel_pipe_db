// 标签打印页 — 钢管标签/质检标签/发货标签/批量标签生成与打印
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
    message.success(t('labels.pipe_label_ready'));
  };

  const handleBatchPrint = () => {
    const ids = batchPipeIds
      .split(',')
      .map((s) => s.trim())
      .filter(Boolean)
      .map(Number);
    if (ids.length === 0) {
      message.warning(t('labels.please_enter_pipe_ids'));
      return;
    }
    batchMutation.mutate(
      { pipe_ids: ids, pipe_type: batchPipeType },
      { onSuccess: () => message.success(t('labels.batch_labels_created')) },
    );
  };

  const handlePrintQualityLabel = () => {
    if (!qualityLabel) return;
    message.success(t('labels.quality_label_ready'));
  };

  const handlePrintShippingLabel = () => {
    if (!orderId) {
      message.warning(t('labels.please_enter_order_id'));
      return;
    }
    shippingMutation.mutate(
      { order_type: orderType, order_id: orderId },
      { onSuccess: () => message.success(t('labels.shipping_label_created')) },
    );
  };

  return (
    <div>
      <Title level={3}>{t('labels.page_title')}</Title>
      <Row gutter={[16, 16]}>
        <Col xs={24} lg={12}>
          <Card title={<><BarcodeOutlined /> {t('labels.pipe_label')}</>}>
            <Form layout="vertical">
              <Form.Item label={t('labels.pipe_type')}>
                <Select value={pipeType} onChange={setPipeType}>
                  <Option value="seamless">{t('labels.pipe_type.seamless')}</Option>
                  <Option value="screen">{t('labels.pipe_type.screen')}</Option>
                </Select>
              </Form.Item>
              <Form.Item label={t('labels.pipe_id')}>
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
                {t('labels.print_label')}
              </Button>
            </Form>
          </Card>
        </Col>
        <Col xs={24} lg={12}>
          <Card title={<><BarcodeOutlined /> {t('labels.batch_labels')}</>}>
            <Form layout="vertical">
              <Form.Item label={t('labels.pipe_type')}>
                <Select value={batchPipeType} onChange={setBatchPipeType}>
                  <Option value="seamless">{t('labels.pipe_type.seamless')}</Option>
                  <Option value="screen">{t('labels.pipe_type.screen')}</Option>
                </Select>
              </Form.Item>
              <Form.Item label={t('labels.pipe_ids_placeholder')}>
                <Input
                  value={batchPipeIds}
                  onChange={(e) => setBatchPipeIds(e.target.value)}
                  placeholder={t('labels.pipe_ids_example')}
                />
              </Form.Item>
              <Button
                type="primary"
                icon={<PrinterOutlined />}
                loading={batchMutation.isPending}
                onClick={handleBatchPrint}
              >
                {t('labels.batch_print')}
              </Button>
            </Form>
          </Card>
        </Col>
      </Row>
      <Divider />
      <Row gutter={[16, 16]}>
        <Col xs={24} lg={12}>
          <Card title={<><FileTextOutlined /> {t('labels.quality_label')}</>}>
            <Form layout="vertical">
              <Form.Item label={t('labels.certificate_id')}>
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
                {t('labels.print_label')}
              </Button>
            </Form>
          </Card>
        </Col>
        <Col xs={24} lg={12}>
          <Card title={<><FileTextOutlined /> {t('labels.shipping_label')}</>}>
            <Form layout="vertical">
              <Form.Item label={t('labels.order_type')}>
                <Select value={orderType} onChange={setOrderType}>
                  <Option value="purchase">{t('labels.order_type.purchase')}</Option>
                  <Option value="sales">{t('labels.order_type.sales')}</Option>
                  <Option value="return">{t('labels.order_type.return')}</Option>
                  <Option value="transfer">{t('labels.order_type.transfer')}</Option>
                </Select>
              </Form.Item>
              <Form.Item label={t('labels.order_id')}>
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
                {t('labels.print_label')}
              </Button>
            </Form>
          </Card>
        </Col>
      </Row>
    </div>
  );
}
