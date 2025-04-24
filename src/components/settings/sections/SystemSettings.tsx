import React from 'react';
import { Form, Switch, Space, Typography, Spin, Button } from 'antd';
import { invoke } from '@tauri-apps/api/tauri';
import { relaunch, exit } from '@tauri-apps/api/process';

const { Title } = Typography;

interface SystemSettingsProps {
  loading?: boolean;
}

export const SystemSettings: React.FC<SystemSettingsProps> = ({ loading }) => {
  if (loading) {
    return <Spin />;
  }

  const handleRestart = async () => {
    try {
      await relaunch();
    } catch (error) {
      console.error('重启失败:', error);
    }
  };

  const handleExit = async () => {
    try {
      await exit(0);
    } catch (error) {
      console.error('退出失败:', error);
    }
  };

  return (
    <Space direction="vertical" style={{ width: '100%' }}>
      <Title level={3}>System Settings</Title>
      
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