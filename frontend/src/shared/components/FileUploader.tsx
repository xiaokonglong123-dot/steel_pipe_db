/**
 * File uploader — wraps Ant Design Upload with file type/size validation
 *
 * Accepts .pdf/.jpg/.png by default, single file up to 10MB.
 * After upload completes, it sends back the file URL via the onSuccess callback.
 */
import { Upload, Button, message } from 'antd';
import { UploadOutlined } from '@ant-design/icons';
import type { UploadProps } from 'antd';
import apiClient from '@/api/client';

interface FileUploaderProps {
  accept?: string;
  maxCount?: number;
  maxSizeMB?: number;
  action?: string;
  onSuccess?: (url: string) => void;
}

export default function FileUploader({ accept = '.pdf,.jpg,.png', maxCount = 1, maxSizeMB = 10, action, onSuccess }: FileUploaderProps) {
  const props: UploadProps = {
    accept,
    maxCount,
    beforeUpload: (file) => {
      const isLt = file.size / 1024 / 1024 < maxSizeMB;
      if (!isLt) {
        message.error(`文件大小不能超过 ${maxSizeMB}MB`);
        return Upload.LIST_IGNORE;
      }
      return true;
    },
    customRequest: (options) => {
      const { file, onSuccess: onOk, onError, filename } = options;
      const formData = new FormData();
      formData.append(filename || 'file', file as File);
      apiClient
        .post(action || '/upload', formData, {
          headers: { 'Content-Type': 'multipart/form-data' },
        })
        .then((res) => {
          onOk?.(res.data);
        })
        .catch((err) => {
          onError?.(err);
        });
    },
    onChange: (info) => {
      if (info.file.status === 'done') {
        message.success(`${info.file.name} 上传成功`);
        onSuccess?.(info.file.response?.url || '');
      } else if (info.file.status === 'error') {
        message.error(`${info.file.name} 上传失败`);
      }
    },
  };

  return (
    <Upload {...props}>
      <Button icon={<UploadOutlined />}>上传文件</Button>
    </Upload>
  );
}
