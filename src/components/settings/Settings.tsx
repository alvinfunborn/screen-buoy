import React, { useEffect, useState } from 'react';
import { Tabs, Form, message } from 'antd';
import { invoke } from '@tauri-apps/api/tauri';
import type { Config } from '../../types/config';
import { HintSettings } from './sections/HintSettings';
import { KeyboardSettings } from './sections/KeyboardSettings';
import { MouseSettings } from './sections/MouseSettings';
import { KeybindingSettings } from './sections/KeybindingSettings';
import { SystemSettings } from './sections/SystemSettings';
import { UiAutomationSettings } from './sections/UiAutomationSettings';

const Settings: React.FC = () => {
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      setLoading(true);
      const config = await invoke<Config>('get_config_for_frontend');
      console.log(config);
      form.setFieldsValue(config);
    } catch (err) {
      message.error('加载配置失败');
      console.error('加载配置失败:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleValuesChange = async (_: any, allValues: Config) => {
    try {
      await invoke('save_config_for_frontend', { config: allValues });
      message.success('保存成功');
    } catch (err) {
      message.error('保存失败');
      console.error('保存配置失败:', err);
    }
  };

  const items = [
    {
      key: 'keybinding',
      label: 'Keybinding',
      children: <KeybindingSettings loading={loading} />,
    },
    {
      key: 'mouse',
      label: 'Mouse',
      children: <MouseSettings loading={loading} />,
    },
    {
      key: 'hint',
      label: 'Hint',
      children: <HintSettings loading={loading} />,
    },
    {
      key: 'ui_automation',
      label: 'Ui Automation',
      children: <UiAutomationSettings loading={loading} />,
    },
    {
      key: 'keyboard',
      label: 'Keyboard',
      children: <KeyboardSettings loading={loading} />,
    },
    {
      key: 'system',
      label: 'System',
      children: <SystemSettings loading={loading} />,
    },
  ];

  return (
    <Form
      form={form}
      layout="vertical"
      onValuesChange={handleValuesChange}
      disabled={loading}
    >
      <Tabs items={items} />
    </Form>
  );
};

export default Settings; 