import React from 'react';
import { Form, InputNumber, Space, Spin, Typography } from 'antd';

const { Title } = Typography;

interface UiAutomationSettingsProps {
  loading?: boolean;
}

export const UiAutomationSettings: React.FC<UiAutomationSettingsProps> = ({ loading }) => {
  if (loading) {
    return <Spin />;
  }

  return (
    <Space direction="vertical" style={{ width: '100%' }}>
      <Title level={3}>UI Automation Settings</Title>
      
      <Form.Item
        label="Collection Interval (ms)"
        name={['ui_automation', 'collect_interval']}
        tooltip="Time interval between UI element scans in milliseconds"
      >
        <InputNumber min={100} max={1000} step={50} />
      </Form.Item>
    </Space>
  );
};
