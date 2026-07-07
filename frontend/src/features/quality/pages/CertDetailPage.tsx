// 质检证书详情页 — 使用 PageLayout 共享组件
import { Button, Descriptions, Tag, Card, Table, Upload, Popconfirm, message } from 'antd';
import { EditOutlined, UploadOutlined, LinkOutlined } from '@ant-design/icons';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { PageLayout } from '@/shared/components/PageLayout';
import { useCert, useAttachments, useCreateAttachment, useDeleteAttachment } from '../hooks/useQuality';
import type { PipeAttachment } from '../types';

const RESULT_COLORS: Record<string, string> = {
  pass: 'green',
  fail: 'red',
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
    createAttachmentMutation.mutate({
      pipe_type: cert!.pipe_type,
      pipe_id: cert!.pipe_id,
      file_name: file.name,
      file_path: `uploads/${file.name}`,
      file_size: file.size,
      content_type: file.type,
    }, {
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
      render: (name: string, record: PipeAttachment) => (
        <a href={record.file_path} target="_blank" rel="noopener noreferrer">
          <LinkOutlined /> {name}
        </a>
      ),
    },
    {
      title: t('quality.content_type'),
      dataIndex: 'content_type',
      key: 'content_type',
      render: (val: string | null) => val ?? '-',
    },
    {
      title: t('quality.created_at'),
      dataIndex: 'created_at',
      key: 'created_at',
    },
    {
      title: t('common.actions'),
      key: 'actions',
      render: (_: unknown, record: PipeAttachment) => (
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
    <PageLayout
      title={`${t('quality.certificate')} — ${cert.cert_number}`}
      onBack={() => navigate('/quality/certs')}
      extra={
        <Button
          type="primary"
          icon={<EditOutlined />}
          onClick={() => navigate(`/quality/certs/${cert.id}/edit`)}
        >
          {t('common.edit')}
        </Button>
      }
    >
      <Card title={t('quality.basic_info')} style={{ marginBottom: 24 }}>
        <Descriptions bordered column={{ xs: 1, sm: 2, lg: 3 }}>
          <Descriptions.Item label={t('quality.cert_number')}>{cert.cert_number}</Descriptions.Item>
          <Descriptions.Item label={t('quality.pipe_type')}>{cert.pipe_type}</Descriptions.Item>
          <Descriptions.Item label={t('quality.pipe_id')}>{cert.pipe_id}</Descriptions.Item>
          <Descriptions.Item label={t('quality.result')}>
            <Tag color={RESULT_COLORS[cert.result] ?? 'default'}>{cert.result}</Tag>
          </Descriptions.Item>
          <Descriptions.Item label={t('quality.inspector')}>{cert.inspector ?? '-'}</Descriptions.Item>
          <Descriptions.Item label={t('quality.inspection_body')}>{cert.inspection_body ?? '-'}</Descriptions.Item>
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
    </PageLayout>
  );
}
