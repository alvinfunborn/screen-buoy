import React, { useEffect, useState } from 'react';
import { Tabs, Form, message, Spin, Select, Space } from 'antd';
import { invoke } from '@tauri-apps/api/core';
import type { Config } from '../../types/config';
import { HintSettings } from './sections/HintSettings';
import { KeyboardSettings } from './sections/KeyboardSettings';
import { MouseSettings } from './sections/MouseSettings';
import { KeybindingSettings } from './sections/KeybindingSettings';
import { SystemSettings } from './sections/SystemSettings';
import { UiAutomationSettings } from './sections/UiAutomationSettings';
import { debounce } from 'lodash';
import { useTranslation } from 'react-i18next';
import i18n from '../../i18n';
import Title from 'antd/es/typography/Title';
import Text from 'antd/es/typography/Text';


const Settings: React.FC = () => {
  const { t, i18n: i18nInstance } = useTranslation();
  const [form] = Form.useForm<Config>();
  const [loading, setLoading] = useState(true);
  const [initialConfig, setInitialConfig] = useState<Config | null>(null);
  const [availableKeysState, setAvailableKeysState] = useState<Record<string, number> | undefined>(undefined);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      setLoading(true);
      const config = await invoke<Config>('get_config_for_frontend');
      console.log("Config loaded from backend:", JSON.stringify(config, null, 2));
      setInitialConfig(config);
      setAvailableKeysState(config.keyboard?.available_key);
    } catch (err) {
      message.error('Failed to load config');
      console.error('Failed to load config:', err);
    } finally {
      setLoading(false);
    }
  };

  const debouncedSave = debounce(async (values: Config) => {
    try {
      console.log("[Settings.tsx] Saving config:", JSON.stringify(values, null, 2));
      await invoke('save_config_for_frontend', { config: values });
    } catch (err) {
      message.error('Failed to save config');
      console.error('Failed to save config:', err);
    }
  }, 500);

  const handleValuesChange = (changedValues: any, _allValues: Config) => {
    console.log("[Settings.tsx] handleValuesChange triggered. Changed:", JSON.stringify(changedValues));

    // 从 form 实例获取最新的完整值，而不是依赖回调参数
    const currentAllValues = form.getFieldsValue(true);
    console.log("[Settings.tsx] currentAllValues from form:", JSON.stringify(currentAllValues, null, 2));

    if (changedValues.keyboard && changedValues.keyboard.available_key) {
      console.log("[Settings.tsx] keyboard.available_key changed, updating state:", changedValues.keyboard.available_key);
      setAvailableKeysState(changedValues.keyboard.available_key);
    } else if (changedValues.keyboard && currentAllValues.keyboard?.available_key !== availableKeysState) {
      console.log("[Settings.tsx] keyboard object changed, updating state:", currentAllValues.keyboard?.available_key);
      setAvailableKeysState(currentAllValues.keyboard?.available_key);
    }

    debouncedSave(currentAllValues); // 使用从 form 获取的最新值
  };

  const handleLanguageChange = (lng: string) => {
    i18nInstance.changeLanguage(lng);
  };

  if (loading || !initialConfig) {
    return <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100vh' }}><Spin size="large" /></div>;
  }

  console.log("Rendering Form with initialConfig:", JSON.stringify(initialConfig, null, 2));
  return (
    <>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginTop: 24, marginBottom: 16 }}>
        <div>
          <Title level={3} style={{ margin: 0, fontWeight: 'bold', marginBottom: 8 }}>{t('settings.title')}</Title>
          <Text type="secondary" style={{ display: 'block' }}>{t('settings.restart')}</Text>
        </div>
        <Space>
          <Select
            value={i18nInstance.language}
            style={{ width: 100 }}
            onChange={handleLanguageChange}
            options={[
              { value: 'en', label: t('settings.language.en') },
              { value: 'zh', label: t('settings.language.zh') },
            ]}
          />
        </Space>
      </div>
      <Form
        form={form}
        layout="vertical"
        initialValues={initialConfig}
        onValuesChange={handleValuesChange}
        style={{ padding: '0 8px' }}
      >
        <Tabs
          defaultActiveKey="keybinding"
          items={[
            {
              key: 'keybinding',
              label: t('settings.tab.keybinding'),
              children: <KeybindingSettings availableKeysData={availableKeysState} />,
            },
            {
              key: 'mouse',
              label: t('settings.tab.mouse'),
              children: <MouseSettings onValuesChange={handleValuesChange} availableKeysData={availableKeysState} />,
            },
            {
              key: 'hint',
              label: t('settings.tab.hint'),
              children: <HintSettings onValuesChange={handleValuesChange} />,
            },
            {
              key: 'ui_automation',
              label: t('settings.tab.uiAutomation'),
              children: <UiAutomationSettings />,
            },
            {
              key: 'keyboard',
              label: t('settings.tab.keyboard'),
              children: <KeyboardSettings onValuesChange={handleValuesChange} />,
            },
            {
              key: 'system',
              label: t('settings.tab.system'),
              children: <SystemSettings />,
            },
          ]}
        />
      </Form>
    </>
  );
};

export default Settings; 