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
      onSuccess: () => message.success('Upload success'),
      onError: () => message.error('Upload failed'),
    });
    return false;
  };

  const attachmentColumns = [
    {
      title: 'File Name',
      dataIndex: 'file_name',
      key: 'file_name',
      render: (name: string, record: Attachment) => (
        <a href={record.file_url} target="_blank" rel="noopener noreferrer">
          <LinkOutlined /> {name}
        </a>
      ),
    },
    {
      title: 'File Type',
      dataIndex: 'file_type',
      key: 'file_type',
    },
    {
      title: 'Uploaded At',
      dataIndex: 'uploaded_at',
      key: 'uploaded_at',
    },
    {
      title: t('common.actions'),
      key: 'actions',
      render: (_: unknown, record: Attachment) => (
        <Popconfirm
          title="确认删除?"
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
        <h2 style={{ margin: 0 }}>Certificate — {cert.cert_number}</h2>
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

      <Card title="Basic Info" style={{ marginBottom: 24 }}>
        <Descriptions bordered column={{ xs: 1, sm: 2, lg: 3 }}>
          <Descriptions.Item label="Cert Number">{cert.cert_number}</Descriptions.Item>
          <Descriptions.Item label="Batch Number">{cert.batch_number || '-'}</Descriptions.Item>
          <Descriptions.Item label="Pipe Type">{cert.pipe_type}</Descriptions.Item>
          <Descriptions.Item label="Grade">
            <Tag color="blue">{cert.grade}</Tag>
          </Descriptions.Item>
          <Descriptions.Item label="OD (in)">{cert.od}</Descriptions.Item>
          <Descriptions.Item label="WT (in)">{cert.wt}</Descriptions.Item>
          <Descriptions.Item label="Length (m)">{cert.length ?? '-'}</Descriptions.Item>
          <Descriptions.Item label="Quantity">{cert.quantity}</Descriptions.Item>
          <Descriptions.Item label="Heat Number">{cert.heat_number ?? '-'}</Descriptions.Item>
          <Descriptions.Item label="Manufacturer">{cert.manufacturer ?? '-'}</Descriptions.Item>
          <Descriptions.Item label="Production Date">{cert.production_date ?? '-'}</Descriptions.Item>
          <Descriptions.Item label="Status">
            <Tag color={STATUS_COLORS[cert.status] ?? 'default'}>{cert.status}</Tag>
          </Descriptions.Item>
        </Descriptions>
      </Card>

      <Card title="Mechanical Properties" style={{ marginBottom: 24 }}>
        <Descriptions bordered column={{ xs: 1, sm: 2, lg: 3 }}>
          <Descriptions.Item label="Test Pressure">{cert.test_pressure ?? '-'}</Descriptions.Item>
          <Descriptions.Item label="Yield Strength">{cert.yield_strength ?? '-'}</Descriptions.Item>
          <Descriptions.Item label="Tensile Strength">{cert.tensile_strength ?? '-'}</Descriptions.Item>
          <Descriptions.Item label="Elongation (%)">{cert.elongation ?? '-'}</Descriptions.Item>
          <Descriptions.Item label="Hardness">{cert.hardness ?? '-'}</Descriptions.Item>
          <Descriptions.Item label="Inspection Standard">{cert.inspection_standard ?? '-'}</Descriptions.Item>
          <Descriptions.Item label="Inspector">{cert.inspector ?? '-'}</Descriptions.Item>
          <Descriptions.Item label="Cert Date">{cert.cert_date ?? '-'}</Descriptions.Item>
        </Descriptions>
      </Card>

      <Card title="Notes" style={{ marginBottom: 24 }}>
        <p>{cert.notes || '-'}</p>
      </Card>

      <Card
        title="Attachments"
        extra={
          <Upload
            beforeUpload={handleUpload}
            showUploadList={false}
            accept=".pdf,.jpg,.png,.doc,.docx"
          >
            <Button icon={<UploadOutlined />}>Upload</Button>
          </Upload>
        }
      >
        <Table
          columns={attachmentColumns}
          dataSource={attachments}
          rowKey="id"
          pagination={false}
          locale={{ emptyText: 'No attachments' }}
        />
      </Card>
    </div>
  );
}
