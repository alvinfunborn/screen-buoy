import React from 'react';
import { Form, Switch, Space, Typography, Spin, Button, Select } from 'antd';
import { relaunch, exit } from '@tauri-apps/plugin-process';
import { Config } from '@/types/config';
import '../../../styles/global.css';

const { Title } = Typography;

interface SystemSettingsProps {
  onValuesChange?: (changedValues: any, allValues: Config) => void;
}

export const SystemSettings: React.FC<SystemSettingsProps> = ({ onValuesChange }) => {
  const handleRestart = async () => {
    try {
      await relaunch();
    } catch (error) {
      console.error('[handleRestart] Failed to restart:', error);
    }
  };

  const handleExit = async () => {
    try {
      await exit(0);
    } catch (error) {
      console.error('[handleExit] Failed to exit:', error);
    }
  };

  return (
    <Space direction="vertical" style={{ width: '100%' }}>
      <Form.Item
        label="Logging Level"
        name={['system', 'logging_level']}
        style={{ width: '200px' }}
        layout="horizontal"
      >
        <Select>
          <Select.Option value="debug">debug</Select.Option>
          <Select.Option value="info">info</Select.Option>
          <Select.Option value="warn">warn</Select.Option>
          <Select.Option value="error">error</Select.Option>
          <Select.Option value="none">none</Select.Option>
        </Select>
      </Form.Item>

      <Form.Item
        label="Debug Mode"
        name={['system', 'debug_mode']}
        valuePropName="checked"
        layout="horizontal"
      >
        <Switch />
      </Form.Item>

      <Form.Item
        label="Start in System Tray"
        name={['system', 'start_in_tray']}
        valuePropName="checked"
        layout="horizontal"
      >
        <Switch />
      </Form.Item>

      <Form.Item
        label="Show Tray Icon"
        name={['system', 'show_tray_icon']}
        valuePropName="checked"
        layout="horizontal"
      >
        <Switch />
      </Form.Item>

      <Form.Item
        label="Start at Login"
        name={['system', 'start_at_login']}
        valuePropName="checked"
        layout="horizontal"
      >
        <Switch />
      </Form.Item>

      <Space>
        <Button type="primary" onClick={handleRestart}>Restart</Button>
        <Button danger onClick={handleExit}>Exit</Button>
      </Space>
    </Space>
  );
}; 