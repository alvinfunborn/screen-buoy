import React from 'react';
import { Form, InputNumber, Space, Spin, Typography } from 'antd';
import type { Config } from '../../../types/config';
import '../../../styles/global.css';

const { Title } = Typography;

interface UiAutomationSettingsProps {
  onValuesChange?: (changedValues: any, allValues: Config) => void;
}

export const UiAutomationSettings: React.FC<UiAutomationSettingsProps> = ({ onValuesChange }) => {

  return (
    <Space direction="vertical" style={{ width: '100%' }}>
      <Form.Item
        layout="horizontal"
        label="Collection Interval (ms)"
        name={['ui_automation', 'collect_interval']}
        tooltip="Time interval between UI element scans in milliseconds"
      >
        <InputNumber min={50} max={10000} step={50} />
      </Form.Item>
      <Form.Item
        layout="horizontal"
        label="Cache TTL (ms)"
        name={['ui_automation', 'cache_ttl']}
        tooltip="Time to live for cached UI element data in milliseconds"
      >
        <InputNumber min={100} max={1000000} step={100} />
      </Form.Item>
    </Space>
  );
};
