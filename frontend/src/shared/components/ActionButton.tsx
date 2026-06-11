/**
 * ActionButton — Modern button with loading, confirm, and tooltip support.
 *
 * Provides consistent action button patterns:
 * - Loading state
 * - Confirmation dialog
 * - Tooltip
 * - Icon support
 * - Danger variant
 */
import { Button, Tooltip, Popconfirm } from 'antd';
import type { ButtonProps } from 'antd';

export interface ActionButtonProps extends Omit<ButtonProps, 'onClick'> {
  /** Tooltip text */
  tooltip?: string;
  /** Confirm dialog title */
  confirmTitle?: string;
  /** Confirm dialog message */
  confirmMessage?: string;
  /** Click handler (async supported) */
  onClick?: () => void | Promise<void>;
  /** Danger style */
  danger?: boolean;
  /** Show as text button */
  text?: boolean;
}

export function ActionButton({
  tooltip,
  confirmTitle,
  confirmMessage,
  onClick,
  danger = false,
  text = false,
  children,
  ...props
}: ActionButtonProps) {
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

  // Wrap with tooltip if provided
  const withTooltip = tooltip ? (
    <Tooltip title={tooltip}>{button}</Tooltip>
  ) : (
    button
  );

  // Wrap with confirm if provided
  if (confirmTitle) {
    return (
      <Popconfirm
        title={confirmTitle}
        description={confirmMessage}
        onConfirm={onClick}
        okText="确定"
        cancelText="取消"
      >
        {withTooltip}
      </Popconfirm>
    );
  }

  return withTooltip;
}

export default ActionButton;
