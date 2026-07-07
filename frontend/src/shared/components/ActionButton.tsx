import React from 'react';
import { Button, Tooltip, Popconfirm } from 'antd';
import type { ButtonProps } from 'antd';
import { useTranslation } from 'react-i18next';

export interface ActionButtonProps extends Omit<ButtonProps, 'onClick'> {
  tooltip?: string;
  confirmTitle?: string;
  confirmMessage?: string;
  onClick?: () => void | Promise<void>;
  danger?: boolean;
  text?: boolean;
}

export const ActionButton = React.memo(function ActionButton({
  tooltip,
  confirmTitle,
  confirmMessage,
  onClick,
  danger = false,
  text = false,
  children,
  ...props
}: ActionButtonProps) {
  const { t } = useTranslation('common');

  const button = (
    <Button
      danger={danger}
      type={text ? 'text' : 'default'}
      onClick={onClick}
      {...props}
    >
      {children}
    </Button>
  );

  const withTooltip = tooltip ? (
    <Tooltip title={tooltip}>{button}</Tooltip>
  ) : (
    button
  );

  if (confirmTitle) {
    return (
      <Popconfirm
        title={confirmTitle}
        description={confirmMessage}
        onConfirm={onClick}
        okText={t('confirm.ok', '确定')}
        cancelText={t('confirm.cancel', '取消')}
      >
        {withTooltip}
      </Popconfirm>
    );
  }

  return withTooltip;
});

export default ActionButton;
