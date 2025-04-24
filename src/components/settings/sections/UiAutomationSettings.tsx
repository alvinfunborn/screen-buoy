import React from 'react';
import { Form, InputNumber, Space, Spin, Typography } from 'antd';
import type { Config } from '../../../types/config';

const { Title } = Typography;

interface UiAutomationSettingsProps {
  loading?: boolean;
  onValuesChange?: (changedValues: any, allValues: Config) => void;
}

export const UiAutomationSettings: React.FC<UiAutomationSettingsProps> = ({ loading, onValuesChange }) => {

  return (
    <Space direction="vertical" style={{ width: '100%' }}>
      <Title level={3}>UI Automation Settings</Title>
      
      <Form.Item
        layout="horizontal"
        label="Collection Interval (ms)"
        name={['ui_automation', 'collect_interval']}
        tooltip="Time interval between UI element scans in milliseconds"
      >
        <InputNumber min={50} max={10000} step={50} />
      </Form.Item>
    </Space>
  );
};
