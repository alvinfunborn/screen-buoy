import React from 'react';
import { Form, Switch, Space, Typography } from 'antd';

const { Title } = Typography;

export const SystemSettings: React.FC = () => {
  return (
    <Space direction="vertical" style={{ width: '100%' }}>
      <Title level={3}>System Settings</Title>
      
      <Form.Item
        label="Start in System Tray"
        name={['system', 'start_in_tray']}
        valuePropName="checked"
      >
        <Switch />
      </Form.Item>

      <Form.Item
        label="Show Tray Icon"
        name={['system', 'show_tray_icon']}
        valuePropName="checked"
      >
        <Switch />
      </Form.Item>

      <Form.Item
        label="Start at Login"
        name={['system', 'start_at_login']}
        valuePropName="checked"
      >
        <Switch />
      </Form.Item>
    </Space>
  );
}; 