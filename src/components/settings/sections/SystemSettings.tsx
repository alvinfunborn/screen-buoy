import React from 'react';
import { Button, Form, Select, Switch } from 'antd';
import { PoweroffOutlined, ReloadOutlined } from '@ant-design/icons';
import { exit, relaunch } from '@tauri-apps/plugin-process';
import { useTranslation } from 'react-i18next';
import { SettingSection } from '../SettingSection';
import { AppUpdates } from '../AppUpdates';

export const SystemSettings: React.FC = () => {
  const { t } = useTranslation();
  const toggle = (field: string, label: string, description: string) => (
    <Form.Item className="setting-row toggle-row" layout="horizontal"
      label={<span>{t(label)}<small>{t(description)}</small></span>}
      name={['system', field]} valuePropName="checked">
      <Switch />
    </Form.Item>
  );
  return (
    <div className="settings-stack">
      <AppUpdates />
      <SettingSection title={t('settings.groups.startup')}>
        {toggle('start_at_login', 'system.startAtLogin', 'settings.help.startAtLogin')}
        {toggle('start_in_tray', 'system.startInTray', 'settings.help.startInTray')}
        {toggle('show_tray_icon', 'system.showTrayIcon', 'settings.help.showTrayIcon')}
      </SettingSection>
      <SettingSection title={t('settings.groups.diagnostics')}>
        <Form.Item className="setting-row" layout="horizontal" label={t('system.loggingLevel')}
          name={['system', 'logging_level']} tooltip={t('system.loggingLevelTooltip')}>
          <Select options={['debug', 'info', 'warn', 'error', 'none'].map(value => ({ value, label: t(`system.${value}`) }))} />
        </Form.Item>
        {toggle('debug_mode', 'system.debugMode', 'system.debugModeTooltip')}
      </SettingSection>
      <SettingSection title={t('settings.groups.application')} description={t('settings.restart')}>
        <div className="application-actions">
          <Button icon={<ReloadOutlined />} onClick={() => relaunch().catch(error => console.error('[handleRestart] Failed to restart:', error))}>{t('system.restart')}</Button>
          <Button danger icon={<PoweroffOutlined />} onClick={() => exit(0).catch(error => console.error('[handleExit] Failed to exit:', error))}>{t('system.exit')}</Button>
        </div>
      </SettingSection>
    </div>
  );
};
