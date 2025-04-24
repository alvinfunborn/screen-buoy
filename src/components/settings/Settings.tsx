import React, { useEffect, useState } from 'react';
import { Tabs, Form, message, Spin } from 'antd';
import { invoke } from '@tauri-apps/api/tauri';
import type { Config } from '../../types/config';
import { HintSettings } from './sections/HintSettings';
import { KeyboardSettings } from './sections/KeyboardSettings';
import { MouseSettings } from './sections/MouseSettings';
import { KeybindingSettings } from './sections/KeybindingSettings';
import { SystemSettings } from './sections/SystemSettings';
import { UiAutomationSettings } from './sections/UiAutomationSettings';
import { debounce } from 'lodash';

const Settings: React.FC = () => {
  const [form] = Form.useForm<Config>();
  const [loading, setLoading] = useState(true);
  const [initialConfig, setInitialConfig] = useState<Config | null>(null);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      setLoading(true);
      const config = await invoke<Config>('get_config_for_frontend');
      console.log("Config loaded from backend:", JSON.stringify(config, null, 2));
      setInitialConfig(config);
    } catch (err) {
      message.error('加载配置失败');
      console.error('加载配置失败:', err);
    } finally {
      setLoading(false);
    }
  };

  const debouncedSave = debounce(async (values: Config) => {
    try {
      await invoke('save_config_for_frontend', { config: values });
      message.success('保存成功');
    } catch (err) {
      message.error('保存失败');
      console.error('保存配置失败:', err);
    }
  }, 500);

  const handleValuesChange = (_: any, allValues: Config) => {
    debouncedSave(allValues);
  };

  if (loading || !initialConfig) {
    return <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100vh' }}><Spin size="large" /></div>;
  }

  console.log("Rendering Form with initialConfig:", JSON.stringify(initialConfig, null, 2));
  return (
    <Form
      form={form}
      layout="vertical"
      initialValues={initialConfig}
      onValuesChange={handleValuesChange}
      style={{ padding: '24px' }}
    >
      <Tabs
        defaultActiveKey="keybinding"
        items={[
          {
            key: 'keybinding',
            label: 'Keybinding',
            children: <KeybindingSettings />,
          },
          {
            key: 'mouse',
            label: 'Mouse',
            children: <MouseSettings onValuesChange={handleValuesChange} />,
          },
          {
            key: 'hint',
            label: 'Hint',
            children: <HintSettings />,
          },
          {
            key: 'ui_automation',
            label: 'UI Automation',
            children: <UiAutomationSettings />,
          },
          {
            key: 'keyboard',
            label: 'Keyboard',
            children: <KeyboardSettings onValuesChange={handleValuesChange} />,
          },
          {
            key: 'system',
            label: 'System',
            children: <SystemSettings />,
          },
        ]}
      />
    </Form>
  );
};

export default Settings; 