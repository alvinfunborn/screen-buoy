import React from 'react';
import { Form, Space, Typography, Spin, Input, Button, InputNumber } from 'antd';
import type { Config } from '../../../types/config';
import { MinusCircleOutlined, PlusOutlined } from '@ant-design/icons';

const { Title } = Typography;

interface HintSettingsProps {
  loading?: boolean;
}

export const HintSettings: React.FC<HintSettingsProps> = ({ loading }) => {
  const form = Form.useFormInstance<Config>(); // Get form instance from context

  if (loading) {
    return <Spin />;
  }

  // Get value directly from form state instead of using useWatch
  const hintTypes = form.getFieldValue(['hint', 'types']) || {};
  console.log("Render (getFieldValue):", hintTypes);
  const hintTypeNames = Object.keys(hintTypes);
  console.log("Render Keys (getFieldValue):", hintTypeNames);

  const handleStyleChange = (typeName: string, event: React.ChangeEvent<HTMLTextAreaElement>) => {
    try {
      const value = JSON.parse(event.target.value);
      form.setFieldValue(['hint', 'types', typeName, 'style'], value);
    } catch (err) {
      // Potentially provide user feedback about invalid JSON
      console.error('Invalid JSON format for', typeName, err);
    }
  };

  const handleZIndexChange = (typeName: string, value: number | null) => {
    form.setFieldValue(['hint', 'types', typeName, 'z_index'], value ?? 0); // Use 0 or some default if null
  };

  // Note: Add/Remove functionality for Record is complex with Form, omitted for simplicity
  // We will only allow editing existing types defined in config.toml initially

  return (
    <Space direction="vertical" style={{ width: '100%' }}>
      <Title level={3}>Hint Settings</Title>

      {/* Hint Charsets Section (Existing) */}
      <Form.Item label="Hint Charsets">
        <Form.List name={['hint', 'charsets']}>
          {(fields, { add, remove }) => (
            <Space direction="vertical" style={{ width: '100%' }}>
              {fields.map((field, index) => {
                // Extract key from field, spread the rest
                const { key, ...restField } = field;
                return (
                  <Space key={key} align="baseline" style={{ width: '100%'}}>
                    <Form.Item
                      key={key} // Pass key directly
                      {...restField} // Spread the rest of the props
                      validateTrigger={['onChange', 'onBlur']}
                      rules={[{ required: true, message: 'Please enter the charset' }]}
                      style={{ flex: 1 }}
                      getValueProps={(value: string[]) => ({
                        value: value?.join(', ')
                      })}
                      // Normalize to always store as string[]
                      normalize={(value: string) => value ? value.split(',').map(s => s.trim()) : []}
                    >
                      <Input placeholder="Enter the charset, separated by commas" style={{ width: '400px' }} />
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

      {/* Hint Charset Extra Section (Existing) */}
      <Form.Item
        label="Hint Charset Extra"
        name={['hint', 'charset_extra']}
        getValueProps={(value: string[]) => ({
          value: value?.join(', ')
        })}
        // Normalize to always store as string[]
        normalize={(value: string) => value ? value.split(',').map(s => s.trim()) : []}
      >
        <Input placeholder="Enter the extra charset, separated by commas" />
      </Form.Item>

      <Title level={4}>Hint Types</Title>
      {/* Dynamic Hint Types Section */}
      {hintTypeNames.map((typeName) => (
        <Space key={typeName} direction="vertical" style={{ border: '1px solid #d9d9d9', padding: '16px', borderRadius: '8px', marginBottom: '16px', width: '100%' }}>
          <Title level={5} style={{ marginTop: 0 }}>Type: {typeName}</Title>
          <Form.Item
            label="Style (JSON)"
            // Use name array for binding and validation
            name={['hint', 'types', typeName, 'style']}
            // Rule to validate JSON (optional but recommended)
            rules={[{
              validator: async (_, value) => {
                if (!value) return Promise.resolve();
                try {
                  JSON.parse(JSON.stringify(value)); // Check if it's valid object/parsable
                  return Promise.resolve();
                } catch (e) {
                  return Promise.reject(new Error('Invalid JSON format'));
                }
              }
            }]}
            // Use getValueProps to display the stringified version
            getValueProps={(value) => ({
              value: value ? JSON.stringify(value, null, 2) : ''
            })}
            style={{ width: '100%' }}
          >
            <Input.TextArea
              rows={6}
              placeholder={`Enter the style JSON for ${typeName}`}
              onChange={(e) => handleStyleChange(typeName, e)}
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
              onChange={(value) => {
                // Ensure value is number or null before passing
                const numericValue = typeof value === 'string' ? parseFloat(value) : value;
                handleZIndexChange(typeName, numericValue);
              }}
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