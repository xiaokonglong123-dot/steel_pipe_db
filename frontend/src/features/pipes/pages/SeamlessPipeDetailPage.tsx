import { Button, Descriptions, Space, Tag, Card } from 'antd';
import { EditOutlined, ArrowLeftOutlined } from '@ant-design/icons';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
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
    <div>
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          marginBottom: 24,
        }}
      >
        <h2 style={{ margin: 0 }}>{t('nav.seamless_pipes')} — {pipe.pipe_number}</h2>
        <Space>
          <Button
            type="primary"
            icon={<EditOutlined />}
            onClick={() => navigate(`/pipes/seamless/${pipe.id}/edit`)}
          >
            {t('common.edit')}
          </Button>
          <Button
            icon={<ArrowLeftOutlined />}
            onClick={() => navigate('/pipes/seamless')}
          >
            {t('common.back')}
          </Button>
        </Space>
      </div>

      <Card>
        <Descriptions bordered column={{ xs: 1, sm: 2, lg: 3 }}>
          <Descriptions.Item label="Pipe Number">{pipe.pipe_number}</Descriptions.Item>
          <Descriptions.Item label="Batch Number">{pipe.batch_number || '-'}</Descriptions.Item>
          <Descriptions.Item label="Pipe Type">{pipe.pipe_type}</Descriptions.Item>
          <Descriptions.Item label="Grade">
            <Tag color="blue">{pipe.grade}</Tag>
          </Descriptions.Item>
          <Descriptions.Item label="OD (in)">{pipe.od}</Descriptions.Item>
          <Descriptions.Item label="WT (in)">{pipe.wt}</Descriptions.Item>
          <Descriptions.Item label="Length (m)">{pipe.length ?? '-'}</Descriptions.Item>
          <Descriptions.Item label="Weight per Unit">{pipe.weight_per_unit ?? '-'}</Descriptions.Item>
          <Descriptions.Item label="End Type">{pipe.end_type ?? '-'}</Descriptions.Item>
          <Descriptions.Item label="Coupling Type">{pipe.coupling_type ?? '-'}</Descriptions.Item>
          <Descriptions.Item label="Coupling OD">{pipe.coupling_od ?? '-'}</Descriptions.Item>
          <Descriptions.Item label="Coupling Length">{pipe.coupling_length ?? '-'}</Descriptions.Item>
          <Descriptions.Item label="Heat Number">{pipe.heat_number ?? '-'}</Descriptions.Item>
          <Descriptions.Item label="Serial Number">{pipe.serial_number ?? '-'}</Descriptions.Item>
          <Descriptions.Item label="Manufacturer">{pipe.manufacturer ?? '-'}</Descriptions.Item>
          <Descriptions.Item label="Production Date">{pipe.production_date ?? '-'}</Descriptions.Item>
          <Descriptions.Item label="Cert Number">{pipe.cert_number ?? '-'}</Descriptions.Item>
          <Descriptions.Item label="Status">
            <Tag color={STATUS_COLORS[pipe.status] ?? 'default'}>{pipe.status}</Tag>
          </Descriptions.Item>
          <Descriptions.Item label="Notes" span={3}>{pipe.notes ?? '-'}</Descriptions.Item>
        </Descriptions>
      </Card>
    </div>
  );
}
