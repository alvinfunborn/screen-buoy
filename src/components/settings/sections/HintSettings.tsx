import React, { useState, useEffect } from 'react';
import { Form, Space, Typography, Spin, Input, Button, InputNumber } from 'antd';
import type { NamePath } from 'antd/es/form/interface';
import type { Config, HintType } from '../../../types/config';
import { MinusCircleOutlined, PlusOutlined } from '@ant-design/icons';

const { Title } = Typography;

interface HintSettingsProps {
  loading?: boolean;
  onValuesChange?: (changedValues: any, allValues: Config) => void;
}

export const HintSettings: React.FC<HintSettingsProps> = ({ loading, onValuesChange }) => {
  const form = Form.useFormInstance<Config>(); // Get form instance from context

  // Re-add the useState definitions
  const [rawStyleInputs, setRawStyleInputs] = useState<Record<string, string>>({});
  const [rawCharsetInputs, setRawCharsetInputs] = useState<Record<number, string>>({});
  const [rawExtraCharsetInput, setRawExtraCharsetInput] = useState<string>('');

  // Get value directly from form state instead of using useWatch
  const hintTypes = form.getFieldValue(['hint', 'types']) || {};
  console.log("Render (getFieldValue):", hintTypes);
  const hintTypeNames = Object.keys(hintTypes);
  console.log("Render Keys (getFieldValue):", hintTypeNames);

  // Note: Add/Remove functionality for Record is complex with Form, omitted for simplicity
  // We will only allow editing existing types defined in config.toml initially

  useEffect(() => {
    const initialHintConfig = form.getFieldValue('hint');
    if (initialHintConfig?.types) {
      // Assert the type of Object.entries result before calling reduce
      const initialRawStyles = (Object.entries(initialHintConfig.types) as [string, HintType][]).reduce((acc, [key, value]) => {
        const styleValue = value.style as any;
        acc[key] = styleValue ? JSON.stringify(styleValue, null, 2) : '';
        return acc;
      }, {} as Record<string, string>);
      setRawStyleInputs(initialRawStyles);
    }
    if (initialHintConfig?.charsets) {
      const initialRawCharsets = initialHintConfig.charsets.reduce((acc: Record<number, string>, value: string[], index: number) => {
        acc[index] = Array.isArray(value) ? value.join(', ') : '';
        return acc;
      }, {} as Record<number, string>);
      setRawCharsetInputs(initialRawCharsets);
    }
    // Initialize extra charset state
    if (initialHintConfig?.charset_extra) {
      setRawExtraCharsetInput(Array.isArray(initialHintConfig.charset_extra) ? initialHintConfig.charset_extra.join(', ') : '');
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [form]);

  // --- Event Handlers ---
  // Re-add the missing handler for local state update
  const handleRawStyleChange = (typeName: string, value: string) => {
    setRawStyleInputs(prev => ({ ...prev, [typeName]: value }));
  };

  const handleRawStyleBlur = (typeName: string) => {
    const rawValue = rawStyleInputs[typeName] ?? ''; // Default to empty string
    console.log(`[handleRawStyleBlur] typeName: ${typeName}, rawValue BEFORE parse attempt:`, JSON.stringify(rawValue)); // Log the raw value

    if (!rawValue.trim()) { // Handle empty or whitespace-only strings
        console.log("[handleRawStyleBlur] Raw value is empty, setting field to undefined.");
        form.setFieldValue(['hint', 'types', typeName, 'style'], undefined);
        form.validateFields([['hint', 'types', typeName, 'style']]); // Validate after clearing
        return;
    }

    try {
      const parsedValue = JSON.parse(rawValue);
      if (typeof parsedValue !== 'object' || Array.isArray(parsedValue) || parsedValue === null) {
          console.error("[handleRawStyleBlur] Parsed value is not a valid object:", parsedValue);
          throw new Error("Style must be a JSON object.");
      }
      console.log("[handleRawStyleBlur] Parse SUCCESS, setting field value:", parsedValue);
      form.setFieldValue(['hint', 'types', typeName, 'style'], parsedValue);
      // Re-validate on success to potentially clear previous errors
      form.validateFields([['hint', 'types', typeName, 'style']]);
    } catch (e) {
      console.error("[handleRawStyleBlur] Parse FAILED:", e);
      // Store the invalid raw string back to let validator catch it
      form.setFieldValue(['hint', 'types', typeName, 'style'], rawValue); // Store raw string on error
      // Trigger validation explicitly for immediate feedback
      form.validateFields([['hint', 'types', typeName, 'style']]);
    }
  };

  // Re-add the missing handler for charsets list items
  const handleRawCharsetBlur = (fieldKey: number, fieldName: number) => {
    const rawValue = rawCharsetInputs[fieldKey] ?? '';
    const newArrayValue = rawValue ? rawValue.split(',').map(s => s.trim()).filter(s => s) : [];
    const fieldPath: NamePath = ['hint', 'charsets', fieldName];
    form.setFieldValue(fieldPath, newArrayValue);

    if (onValuesChange) {
        const currentCharsets = form.getFieldValue(['hint', 'charsets']) || [];
        const allCharsets = [...currentCharsets];
        if (fieldName >= allCharsets.length) {
            allCharsets.length = fieldName + 1;
        }
        allCharsets[fieldName] = newArrayValue;

        const changedValues = { hint: { charsets: allCharsets } };
        const allValues = form.getFieldsValue(true);
        onValuesChange(changedValues, allValues);
        // Pass a mutable copy of the tuple
        form.validateFields([...fieldPath]);
    }
  };

  const handleRawCharsetChange = (fieldKey: number, value: string) => {
      setRawCharsetInputs(prev => ({ ...prev, [fieldKey]: value }));
  };

  // Add handler for extra charset blur
  const handleRawExtraCharsetBlur = () => {
    const rawValue = rawExtraCharsetInput ?? '';
    const newArrayValue = rawValue ? rawValue.split(',').map(s => s.trim()).filter(s => s) : [];
    const fieldPath: NamePath = ['hint', 'charset_extra'];
    form.setFieldValue(fieldPath, newArrayValue);

    if (onValuesChange) {
        const changedValues = { hint: { charset_extra: newArrayValue } };
        const allValues = form.getFieldsValue(true);
        onValuesChange(changedValues, allValues);
        // Pass a mutable copy of the tuple
        form.validateFields([...fieldPath]);
    }
  };

  return (
    <Space direction="vertical" style={{ width: '100%' }}>
      <Title level={3}>Hint Settings</Title>

      {/* Hint Charsets Section (Existing) */}
      <Form.Item label="Hint Charsets">
        <Form.List name={['hint', 'charsets']}>
          {(fields, { add, remove }) => (
            <Space direction="vertical" style={{ width: '100%' }}>
              {fields.map((field, index) => {
                const { key, name, ...restField } = field;
                // Initialize local state for new fields if missing
                if (rawCharsetInputs[key] === undefined) {
                    // Initialize with empty string based on potential form value if needed,
                    // but usually safe to start empty for a new field.
                    // Check form state just in case add() provides non-empty default
                    const formValue = form.getFieldValue(['hint', 'charsets', name]);
                    const initialRawValue = Array.isArray(formValue) ? formValue.join(', ') : '';
                    // Use effect or direct call? Direct call might cause issues.
                    // Let's just ensure value uses fallback for render.
                    // We need to update the state properly. Schedule update after render? No.
                    // Best approach: initialize directly in the state during this render cycle is tricky.
                    // Alternative: Use default value in the input and rely on onChange/onBlur to populate state.
                    // Let's stick to ensuring the input's value defaults correctly.
                    // We'll handle state init implicitly via onChange.
                }
                return (
                  <Space key={key} align="baseline" style={{ width: '100%'}}>
                    <Form.Item
                      key={key}
                      {...restField}
                      validateTrigger={['onChange', 'onBlur']}
                      rules={[{ required: true, message: 'Please enter the charset' }]}
                      style={{ flex: 1 }}
                    >
                      <Input
                        placeholder="Enter the charset, separated by commas"
                        // Ensure value defaults to empty string if state entry is missing
                        value={rawCharsetInputs[key] ?? ''}
                        onChange={(e) => handleRawCharsetChange(key, e.target.value)}
                        onBlur={() => handleRawCharsetBlur(key, name)}
                      />
                    </Form.Item>
                    <MinusCircleOutlined style={{ marginLeft: 8 }} onClick={() => remove(field.name)} />
                  </Space>
                );
              })}
              <Button type="dashed" onClick={() => add([])} block icon={<PlusOutlined />}>
                Add Charset
              </Button>
            </Space>
          )}
        </Form.List>
      </Form.Item>

      {/* Hint Charset Extra Section */}
      <Form.Item
        label="Hint Charset Extra"
      >
        <Input
          placeholder="Enter the extra charset, separated by commas"
          value={rawExtraCharsetInput}
          onChange={(e) => setRawExtraCharsetInput(e.target.value)}
          onBlur={handleRawExtraCharsetBlur}
        />
      </Form.Item>

      <Title level={4}>Hint Types</Title>
      {/* Dynamic Hint Types Section */}
      {hintTypeNames.map((typeName) => (
        <Space key={typeName} direction="vertical" style={{ border: '1px solid #d9d9d9', padding: '16px', borderRadius: '8px', marginBottom: '16px', width: '100%' }}>
          <Title level={5} style={{ marginTop: 0 }}>Type: {typeName}</Title>
          <Form.Item
            label="Style (JSON)"
            rules={[{
              validator: async (_, value) => {
                if (value === undefined || value === '') return Promise.resolve();
                if (typeof value === 'string') {
                  return Promise.reject(new Error('Invalid JSON format (parsing failed or not an object)'));
                }
                return Promise.resolve();
              }
            }]}
            style={{ width: '100%' }}
          >
            <Input.TextArea
              rows={6}
              placeholder={`Enter the style JSON for ${typeName}`}
              value={rawStyleInputs[typeName] ?? ''}
              onChange={(e) => handleRawStyleChange(typeName, e.target.value)}
              onBlur={() => handleRawStyleBlur(typeName)}
              style={{ width: '400px' }}
            />
          </Form.Item>
          <Form.Item
            label="Z-Index"
            name={['hint', 'types', typeName, 'z_index']}
            rules={[{ required: true, type: 'number', message: 'Please enter a z-index' }]}
            style={{ width: '100%' }}
          >
            <InputNumber
              placeholder="Enter Z-Index"
              style={{ width: '150px' }}
            />
          </Form.Item>
          {/* Add remove button if needed, requires more complex state management */}
          {/* <Button danger onClick={() => handleRemoveType(typeName)} icon={<MinusCircleOutlined />}>Remove {typeName}</Button> */}
        </Space>
      ))}
      {/* Add button if needed */}
      {/* <Button type="dashed" onClick={handleAddType} block icon={<PlusOutlined />}>Add Hint Type</Button> */}

    </Space>
  );
}; 