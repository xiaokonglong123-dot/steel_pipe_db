import { Button, Descriptions, Space, Tag, Card, Table, Upload, Popconfirm, message } from 'antd';
import { EditOutlined, ArrowLeftOutlined, UploadOutlined, LinkOutlined } from '@ant-design/icons';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useCert, useAttachments, useCreateAttachment, useDeleteAttachment } from '../hooks/useQuality';
import type { Attachment } from '../types';

const STATUS_COLORS: Record<string, string> = {
  draft: 'default',
  active: 'green',
  void: 'red',
};

export default function CertDetailPage() {
  const { t } = useTranslation();
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const certId = Number(id);

  const { data: cert, isLoading } = useCert(certId);
  const { data: attachments } = useAttachments(certId);
  const createAttachmentMutation = useCreateAttachment();
  const deleteAttachmentMutation = useDeleteAttachment();

  const handleUpload = (file: File) => {
    const formData = new FormData();
    formData.append('file', file);
    formData.append('cert_id', String(certId));
    createAttachmentMutation.mutate(formData, {
      onSuccess: () => message.success(t('common.operate_success')),
      onError: () => message.error(t('common.operate_failed')),
    });
    return false;
  };

  const attachmentColumns = [
    {
      title: t('quality.file_name'),
      dataIndex: 'file_name',
      key: 'file_name',
      render: (name: string, record: Attachment) => (
        <a href={record.file_url} target="_blank" rel="noopener noreferrer">
          <LinkOutlined /> {name}
        </a>
      ),
    },
    {
      title: t('quality.file_type'),
      dataIndex: 'file_type',
      key: 'file_type',
    },
    {
      title: t('quality.uploaded_at'),
      dataIndex: 'uploaded_at',
      key: 'uploaded_at',
    },
    {
      title: t('common.actions'),
      key: 'actions',
      render: (_: unknown, record: Attachment) => (
        <Popconfirm
          title={t('common.confirm_delete')}
          onConfirm={() => deleteAttachmentMutation.mutate(record.id)}
        >
          <Button type="link" danger>
            {t('common.delete')}
          </Button>
        </Popconfirm>
      ),
    },
  ];

  if (isLoading) {
    return <div>{t('common.loading')}</div>;
  }

  if (!cert) {
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
        <h2 style={{ margin: 0 }}>{t('quality.certificate')} — {cert.cert_number}</h2>
        <Space>
          <Button
            type="primary"
            icon={<EditOutlined />}
            onClick={() => navigate(`/quality/certs/${cert.id}/edit`)}
          >
            {t('common.edit')}
          </Button>
          <Button
            icon={<ArrowLeftOutlined />}
            onClick={() => navigate('/quality/certs')}
          >
            {t('common.back')}
          </Button>
        </Space>
      </div>

      <Card title={t('quality.basic_info')} style={{ marginBottom: 24 }}>
        <Descriptions bordered column={{ xs: 1, sm: 2, lg: 3 }}>
          <Descriptions.Item label={t('quality.cert_number')}>{cert.cert_number}</Descriptions.Item>
          <Descriptions.Item label={t('quality.batch_number')}>{cert.batch_number || '-'}</Descriptions.Item>
          <Descriptions.Item label={t('quality.pipe_type')}>{cert.pipe_type}</Descriptions.Item>
          <Descriptions.Item label={t('quality.grade')}>
            <Tag color="blue">{cert.grade}</Tag>
          </Descriptions.Item>
          <Descriptions.Item label={t('quality.od')}>{cert.od}</Descriptions.Item>
          <Descriptions.Item label={t('quality.wt')}>{cert.wt}</Descriptions.Item>
          <Descriptions.Item label={t('quality.length')}>{cert.length ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('quality.quantity')}>{cert.quantity}</Descriptions.Item>
          <Descriptions.Item label={t('quality.heat_number')}>{cert.heat_number ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('quality.manufacturer')}>{cert.manufacturer ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('quality.production_date')}>{cert.production_date ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('quality.status')}>
            <Tag color={STATUS_COLORS[cert.status] ?? 'default'}>{cert.status}</Tag>
          </Descriptions.Item>
        </Descriptions>
      </Card>

      <Card title={t('quality.mechanical_properties')} style={{ marginBottom: 24 }}>
        <Descriptions bordered column={{ xs: 1, sm: 2, lg: 3 }}>
          <Descriptions.Item label={t('quality.test_pressure')}>{cert.test_pressure ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('quality.yield_strength')}>{cert.yield_strength ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('quality.tensile_strength')}>{cert.tensile_strength ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('quality.elongation')}>{cert.elongation ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('quality.hardness')}>{cert.hardness ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('quality.inspection_standard')}>{cert.inspection_standard ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('quality.inspector')}>{cert.inspector ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('quality.cert_date')}>{cert.cert_date ?? '-'}</Descriptions.Item>
        </Descriptions>
      </Card>

      <Card title={t('quality.notes')} style={{ marginBottom: 24 }}>
        <p>{cert.notes || '-'}</p>
      </Card>

      <Card
        title={t('quality.attachments')}
        extra={
          <Upload
            beforeUpload={handleUpload}
            showUploadList={false}
            accept=".pdf,.jpg,.png,.doc,.docx"
          >
            <Button icon={<UploadOutlined />}>{t('quality.upload')}</Button>
          </Upload>
        }
      >
        <Table
          columns={attachmentColumns}
          dataSource={attachments}
          rowKey="id"
          pagination={false}
          locale={{ emptyText: t('common.no_data') }}
        />
      </Card>
    </div>
  );
}
