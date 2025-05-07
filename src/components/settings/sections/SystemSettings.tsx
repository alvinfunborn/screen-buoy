import React from 'react';
import { Form, Switch, Space, Typography, Spin, Button, Select } from 'antd';
import { relaunch, exit } from '@tauri-apps/plugin-process';
import { Config } from '@/types/config';
import '../../../styles/global.css';
import { useTranslation } from 'react-i18next';

const { Title } = Typography;

interface SystemSettingsProps {
  onValuesChange?: (changedValues: any, allValues: Config) => void;
}

export const SystemSettings: React.FC<SystemSettingsProps> = ({ onValuesChange }) => {
  const { t } = useTranslation();

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
        label={t('system.loggingLevel')}
        name={['system', 'logging_level']}
        style={{ width: '200px' }}
        tooltip={t('system.loggingLevelTooltip')}
        layout="horizontal"
      >
        <Select>
          <Select.Option value="debug">{t('system.debug')}</Select.Option>
          <Select.Option value="info">{t('system.info')}</Select.Option>
          <Select.Option value="warn">{t('system.warn')}</Select.Option>
          <Select.Option value="error">{t('system.error')}</Select.Option>
          <Select.Option value="none">{t('system.none')}</Select.Option>
        </Select>
      </Form.Item>

      <Form.Item
        label={t('system.debugMode')}
        name={['system', 'debug_mode']}
        valuePropName="checked"
        tooltip={t('system.debugModeTooltip')}
        layout="horizontal"
      >
        <Switch />
      </Form.Item>

      <Form.Item
        label={t('system.startInTray')}
        name={['system', 'start_in_tray']}
        valuePropName="checked"
        layout="horizontal"
      >
        <Switch />
      </Form.Item>

      <Form.Item
        label={t('system.showTrayIcon')}
        name={['system', 'show_tray_icon']}
        valuePropName="checked"
        layout="horizontal"
      >
        <Switch />
      </Form.Item>

      <Form.Item
        label={t('system.startAtLogin')}
        name={['system', 'start_at_login']}
        valuePropName="checked"
        layout="horizontal"
      >
        <Switch />
      </Form.Item>

      <Space>
        <Button type="primary" onClick={handleRestart}>{t('system.restart')}</Button>
        <Button danger onClick={handleExit}>{t('system.exit')}</Button>
      </Space>
    </Space>
  );
}; 