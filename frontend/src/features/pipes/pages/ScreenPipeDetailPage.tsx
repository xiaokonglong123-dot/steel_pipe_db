import { Button, Descriptions, Space, Tag, Card } from 'antd';
import { EditOutlined, ArrowLeftOutlined } from '@ant-design/icons';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useScreenPipe } from '../hooks/useScreenPipes';

const STATUS_COLORS: Record<string, string> = {
  in_stock: 'green',
  outbound: 'orange',
  scrapped: 'red',
};

export default function ScreenPipeDetailPage() {
  const { t } = useTranslation();
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const pipeId = Number(id);

  const { data: pipe, isLoading } = useScreenPipe(pipeId);

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
        <h2 style={{ margin: 0 }}>{t('nav.screen_pipes')} — {pipe.pipe_number}</h2>
        <Space>
          <Button
            type="primary"
            icon={<EditOutlined />}
            onClick={() => navigate(`/pipes/screen/${pipe.id}/edit`)}
          >
            {t('common.edit')}
          </Button>
          <Button
            icon={<ArrowLeftOutlined />}
            onClick={() => navigate('/pipes/screen')}
          >
            {t('common.back')}
          </Button>
        </Space>
      </div>

      <Card>
        <Descriptions bordered column={{ xs: 1, sm: 2, lg: 3 }}>
          <Descriptions.Item label="Pipe Number">{pipe.pipe_number}</Descriptions.Item>
          <Descriptions.Item label="Batch Number">{pipe.batch_number || '-'}</Descriptions.Item>
          <Descriptions.Item label="Screen Type">
            <Tag color="cyan">{pipe.screen_type}</Tag>
          </Descriptions.Item>
          <Descriptions.Item label="Slot Size">{pipe.slot_size ?? '-'}</Descriptions.Item>
          <Descriptions.Item label="Filtration Grade">{pipe.filtration_grade ?? '-'}</Descriptions.Item>
          <Descriptions.Item label="Base OD (in)">{pipe.base_od}</Descriptions.Item>
          <Descriptions.Item label="Base WT (in)">{pipe.base_wt}</Descriptions.Item>
          <Descriptions.Item label="Base Grade">
            <Tag color="blue">{pipe.base_grade}</Tag>
          </Descriptions.Item>
          <Descriptions.Item label="Base End Type">{pipe.base_end_type ?? '-'}</Descriptions.Item>
          <Descriptions.Item label="Length (m)">{pipe.length ?? '-'}</Descriptions.Item>
          <Descriptions.Item label="Weight per Unit">{pipe.weight_per_unit ?? '-'}</Descriptions.Item>
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
