/**
 * FormField — Modern form field wrapper with label, help text, and validation.
 *
 * Provides consistent form field patterns:
 * - Label with optional tooltip
 * - Help text
 * - Error message
 * - Required indicator
 * - Compact/normal layout
 */
import { Form, Input, Select, DatePicker, InputNumber, Switch } from 'antd';
import type { FormItemProps } from 'antd';

const { TextArea } = Input;

export interface FormFieldProps extends Omit<FormItemProps, 'children'> {
  /** Field type */
  type?: 'text' | 'textarea' | 'number' | 'select' | 'date' | 'switch';
  /** Placeholder text */
  placeholder?: string;
  /** Options for select type */
  options?: Array<{ label: string; value: unknown }>;
  /** For textarea: number of rows */
  rows?: number;
  /** For number: min/max/step */
  min?: number;
  max?: number;
  step?: number;
  /** For select: allow clear */
  allowClear?: boolean;
  /** For select: show search */
  showSearch?: boolean;
  /** For select: filter function */
  filterOption?: (input: string, option: unknown) => boolean;
  /** Disable the field */
  disabled?: boolean;
  /** Readonly the field */
  readOnly?: boolean;
}

export function FormField({
  type = 'text',
  placeholder,
  options,
  rows = 3,
  min,
  max,
  step,
  allowClear = true,
  showSearch,
  filterOption,
  disabled,
  readOnly,
  ...props
}: FormFieldProps) {
  const renderField = () => {
    switch (type) {
      case 'textarea':
        return (
          <TextArea
            placeholder={placeholder}
            rows={rows}
            disabled={disabled}
            readOnly={readOnly}
          />
        );

      case 'number':
        return (
          <InputNumber
            placeholder={placeholder}
            min={min}
            max={max}
            step={step}
            disabled={disabled}
            readOnly={readOnly}
            style={{ width: '100%' }}
          />
        );

      case 'select':
        return (
          <Select
            placeholder={placeholder}
            options={options}
            allowClear={allowClear}
            showSearch={showSearch}
            filterOption={filterOption}
            disabled={disabled}
          />
        );

      case 'date':
        return (
          <DatePicker
            placeholder={placeholder}
            disabled={disabled}
            style={{ width: '100%' }}
          />
        );

      case 'switch':
        return (
          <Switch
            disabled={disabled}
            checkedChildren="是"
            unCheckedChildren="否"
          />
        );

      default:
        return (
          <Input
            placeholder={placeholder}
            disabled={disabled}
            readOnly={readOnly}
          />
        );
    }
  };

  return (
    <Form.Item {...props}>
      {renderField()}
    </Form.Item>
  );
}

export default FormField;
