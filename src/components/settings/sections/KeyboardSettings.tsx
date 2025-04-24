import React from 'react';
import { Form, Switch, Space, Typography, Select, Input } from 'antd';

const { Title } = Typography;

export const KeyboardSettings: React.FC = () => {
  return (
    <Space direction="vertical" style={{ width: '100%' }}>
      <Title level={3}>Keyboard Settings</Title>
      
      <Form.Item
        label="Available Keys"
        name={['keyboard', 'availableKeys']}
      >
        <Input.TextArea rows={4} placeholder="Enter available keys, one per line" />
      </Form.Item>

      <Form.Item
        label="Enable Hotkeys"
        name={['keyboard', 'enabled']}
        valuePropName="checked"
      >
        <Switch />
      </Form.Item>

      <Form.Item
        label="Activation Key"
        name={['keyboard', 'activationKey']}
      >
        <Select
          options={[
            { label: 'Alt', value: 'alt' },
            { label: 'Ctrl', value: 'ctrl' },
            { label: 'Shift', value: 'shift' },
            { label: 'Win', value: 'win' }
          ]}
        />
      </Form.Item>

      <Form.Item
        label="Hint Keys"
        name={['keyboard', 'hintKeys']}
      >
        <Select
          options={[
            { label: 'Letters', value: 'letters' },
            { label: 'Numbers', value: 'numbers' },
            { label: 'Both', value: 'both' }
          ]}
        />
      </Form.Item>

      <Form.Item
        label="Clear on Escape"
        name={['keyboard', 'clearOnEscape']}
        valuePropName="checked"
      >
        <Switch />
      </Form.Item>
    </Space>
  );
}; 