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
import { debounce } from 'lodash';

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
      form.setFieldsValue(config);
    } catch (err) {
      message.error('加载配置失败');
      console.error('加载配置失败:', err);
    } finally {
      setLoading(false);
    }
  };

  // 使用 debounce 防止频繁保存
  const debouncedSave = debounce(async (values: Config) => {
    try {
      await invoke('save_config_for_frontend', { config: values });
      message.success('保存成功');
    } catch (err) {
      message.error('保存失败');
      console.error('保存配置失败:', err);
    }
  }, 500);

  const handleValuesChange = async (_: any, allValues: Config) => {
    debouncedSave(allValues);
  };

  return (
    <Form
      form={form}
      layout="vertical"
      onValuesChange={handleValuesChange}
      style={{ padding: '24px' }}
    >
      <Tabs
        defaultActiveKey="keybinding"
        items={[
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
            label: 'UI Automation',
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
        ]}
      />
    </Form>
  );
};

export default Settings; 