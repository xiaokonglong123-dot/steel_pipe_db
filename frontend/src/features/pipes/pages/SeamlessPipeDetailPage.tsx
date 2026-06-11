// 无缝钢管详情页 — 使用 PageLayout 共享组件
import { Button, Descriptions, Tag, Card } from 'antd';
import { EditOutlined } from '@ant-design/icons';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { PageLayout } from '@/shared/components/PageLayout';
import { useSeamlessPipe } from '../hooks/useSeamlessPipes';

const STATUS_COLORS: Record<string, string> = {
  in_stock: 'green',
  outbound: 'orange',
  scrapped: 'red',
};

export default function SeamlessPipeDetailPage() {
  const { t } = useTranslation();
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const pipeId = Number(id);

  const { data: pipe, isLoading } = useSeamlessPipe(pipeId);

  if (isLoading) {
    return <div>{t('common.loading')}</div>;
  }

  if (!pipe) {
    return <div>{t('common.no_data')}</div>;
  }

  return (
    <PageLayout
      title={`${t('nav.seamless_pipes')} — ${pipe.pipe_number}`}
      onBack={() => navigate('/pipes/seamless')}
      extra={
        <Button
          type="primary"
          icon={<EditOutlined />}
          onClick={() => navigate(`/pipes/seamless/${pipe.id}/edit`)}
        >
          {t('common.edit')}
        </Button>
      }
    >
      <Card>
        <Descriptions bordered column={{ xs: 1, sm: 2, lg: 3 }}>
          <Descriptions.Item label={t('pipes.pipe_number')}>{pipe.pipe_number}</Descriptions.Item>
          <Descriptions.Item label={t('pipes.batch_number')}>{pipe.batch_number || '-'}</Descriptions.Item>
          <Descriptions.Item label={t('pipes.pipe_type')}>{pipe.pipe_type}</Descriptions.Item>
          <Descriptions.Item label={t('pipes.grade')}>
            <Tag color="blue">{pipe.grade}</Tag>
          </Descriptions.Item>
          <Descriptions.Item label={t('pipes.od')}>{pipe.od}</Descriptions.Item>
          <Descriptions.Item label={t('pipes.wt')}>{pipe.wt}</Descriptions.Item>
          <Descriptions.Item label={t('pipes.length')}>{pipe.length ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('pipes.weight_per_unit')}>{pipe.weight_per_unit ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('pipes.end_type')}>{pipe.end_type ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('pipes.coupling_type')}>{pipe.coupling_type ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('pipes.coupling_od')}>{pipe.coupling_od ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('pipes.coupling_length')}>{pipe.coupling_length ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('pipes.heat_number')}>{pipe.heat_number ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('pipes.serial_number')}>{pipe.serial_number ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('pipes.manufacturer')}>{pipe.manufacturer ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('pipes.production_date')}>{pipe.production_date ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('pipes.cert_number')}>{pipe.cert_number ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('pipes.status')}>
            <Tag color={STATUS_COLORS[pipe.status] ?? 'default'}>{pipe.status}</Tag>
          </Descriptions.Item>
          <Descriptions.Item label={t('pipes.notes')} span={3}>{pipe.notes ?? '-'}</Descriptions.Item>
        </Descriptions>
      </Card>
    </PageLayout>
  );
}
