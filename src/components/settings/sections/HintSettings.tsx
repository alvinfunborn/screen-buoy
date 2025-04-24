import React from 'react';
import { Form, Switch, InputNumber, Space, Typography, Spin } from 'antd';

const { Title } = Typography;

interface HintSettingsProps {
  loading?: boolean;
}

export const HintSettings: React.FC<HintSettingsProps> = ({ loading }) => {
  if (loading) {
    return <Spin />;
  }
  
  return (
    <Space direction="vertical" style={{ width: '100%' }}>
      <Title level={3}>Hint Settings</Title>
      
      <Form.Item
        label="Show Hints"
        name={['hints', 'enabled']}
        valuePropName="checked"
      >
        <Switch />
      </Form.Item>

      <Form.Item
        label="Hint Size"
        name={['hints', 'size']}
      >
        <InputNumber min={8} max={32} />
      </Form.Item>

      <Form.Item
        label="Hint Color"
        name={['hints', 'color']}
      >
        <input 
          type="color" 
          title="Choose hint color"
          aria-label="Choose hint color"
        />
      </Form.Item>

      <Form.Item
        label="Hint Opacity"
        name={['hints', 'opacity']}
      >
        <InputNumber min={0} max={1} step={0.1} />
      </Form.Item>

      <Form.Item
        label="Show Border"
        name={['hints', 'showBorder']}
        valuePropName="checked"
      >
        <Switch />
      </Form.Item>
    </Space>
  );
}; 