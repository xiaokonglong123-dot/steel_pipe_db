/**
 * Reusable confirm dialog — wraps Ant Design's Modal.confirm
 *
 * Comes with confirm/cancel buttons and a danger mode (turns the button red).
 * Great for stuff like deletes where you really want the user to double-check.
 */
import { Modal } from 'antd';
import { ExclamationCircleOutlined } from '@ant-design/icons';

interface ConfirmModalOptions {
  title: string;
  content: string;
  onOk: () => void;
  onCancel?: () => void;
  okText?: string;
  cancelText?: string;
  danger?: boolean;
}

export function showConfirm({ title, content, onOk, onCancel, okText = '确认', cancelText = '取消', danger = false }: ConfirmModalOptions) {
  Modal.confirm({
    title,
    icon: <ExclamationCircleOutlined />,
    content,
    okText,
    cancelText,
    okButtonProps: danger ? { danger: true } : undefined,
    onOk,
    onCancel,
  });
}
