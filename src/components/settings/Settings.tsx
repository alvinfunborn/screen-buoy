import React, { useEffect, useMemo, useRef, useState } from 'react';
import { Button, Form, Grid, Result, Select, Spin, Tabs, message } from 'antd';
import {
  AimOutlined, AppstoreOutlined, CheckCircleOutlined, ControlOutlined,
  ExclamationCircleOutlined, FontSizeOutlined, GlobalOutlined,
  LoadingOutlined, SettingOutlined, ThunderboltOutlined,
} from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';
import { debounce } from 'lodash';
import { useTranslation } from 'react-i18next';
import type { Config, LanguagePreference } from '../../types/config';
import { HintSettings } from './sections/HintSettings';
import { KeyboardSettings } from './sections/KeyboardSettings';
import { MouseSettings } from './sections/MouseSettings';
import { KeybindingSettings } from './sections/KeybindingSettings';
import { SystemSettings } from './sections/SystemSettings';
import { UiAutomationSettings } from './sections/UiAutomationSettings';
import { MacPermissions } from './MacPermissions';
import i18n, { applyLanguage } from '../../i18n';
import { languages } from '../../language.js';
import { getHintPrefixConflicts, getPrefixConflictsFromError } from './hint-validation.js';

const appIconUrl = new URL('../../../src-tauri/icons/icon.png', import.meta.url).href;

type SaveState = 'idle' | 'saving' | 'saved' | 'error';

const Settings: React.FC = () => {
  const { t } = useTranslation();
  const [languagePreference, setLanguagePreference] = useState<LanguagePreference>('auto');
  const [form] = Form.useForm<Config>();
  const screens = Grid.useBreakpoint();
  const [loading, setLoading] = useState(true);
  const [initialConfig, setInitialConfig] = useState<Config | null>(null);
  const [availableKeys, setAvailableKeys] = useState<Record<string, number>>();
  const [saveState, setSaveState] = useState<SaveState>('idle');
  const revision = useRef(0);
  const saveQueue = useRef(Promise.resolve());

  const loadConfig = async () => {
    setLoading(true);
    try {
      const config = await invoke<Config>('get_config_for_frontend');
      setInitialConfig(config);
      setLanguagePreference(config.system.language ?? 'auto');
      await applyLanguage(config.system.language ?? 'auto');
      setAvailableKeys(config.keyboard?.available_key);
    } catch (err) {
      console.error('Failed to load config:', err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { void loadConfig(); }, []);

  // Keep the existing 500 ms autosave stable across presentation re-renders.
  // Serialize writes so a slower earlier save cannot overwrite the latest edit.
  const debouncedSave = useMemo(() => debounce((values: Config, version: number) => {
    saveQueue.current = saveQueue.current.then(async () => {
      if (revision.current !== version || getHintPrefixConflicts(values.hint).length) return;
      try {
        await invoke('save_config_for_frontend', { config: values });
        if (revision.current === version) setSaveState('saved');
      } catch (err) {
        if (revision.current === version) {
          setSaveState('error');
          const conflicts = getPrefixConflictsFromError(err);
          message.error(conflicts
            ? i18n.t('hint.charsetPrefixConflict', { characters: conflicts.join(', ') })
            : i18n.t('settings.saveError'));
        }
        console.error('Failed to save config:', err);
      }
    });
  }, 500), []);

  useEffect(() => () => { debouncedSave.flush(); }, [debouncedSave]);

  const handleValuesChange = (changedValues: Partial<Config>, _allValues: Config) => {
    const values = form.getFieldsValue(true) as Config;
    if (changedValues.keyboard) setAvailableKeys(values.keyboard?.available_key);
    const version = ++revision.current;
    if (getHintPrefixConflicts(values.hint).length) {
      debouncedSave.cancel();
      setSaveState('error');
      return;
    }
    setSaveState('saving');
    debouncedSave(values, version);
  };

  const pages = [
    { key: 'keybinding', icon: <ThunderboltOutlined />, content: <KeybindingSettings availableKeysData={availableKeys} /> },
    { key: 'mouse', icon: <AimOutlined />, content: <MouseSettings onValuesChange={handleValuesChange} availableKeysData={availableKeys} /> },
    { key: 'hint', icon: <FontSizeOutlined />, content: <HintSettings onValuesChange={handleValuesChange} /> },
    { key: 'uiAutomation', icon: <AppstoreOutlined />, content: <UiAutomationSettings /> },
    { key: 'keyboard', icon: <ControlOutlined />, content: <KeyboardSettings onValuesChange={handleValuesChange} /> },
    { key: 'system', icon: <SettingOutlined />, content: <SystemSettings /> },
  ];

  return (
    <div className="settings-shell">
      <header className="settings-header">
        <div className="settings-brand">
          <img className="brand-mark" src={appIconUrl} alt="" aria-hidden="true" />
          <div><strong>Screen Buoy</strong><span>{t('settings.title')}</span></div>
        </div>
        <div className="settings-header-actions">
          {initialConfig && <span className={`save-status save-status-${saveState}`} role="status" aria-live="polite">
            {saveState === 'saving' ? <LoadingOutlined /> : saveState === 'error' ? <ExclamationCircleOutlined /> : <CheckCircleOutlined />}
            <span>{t(`settings.save.${saveState}`)}</span>
          </span>}
          <Select
            className="language-select"
            aria-label={t('settings.language')}
            prefix={<GlobalOutlined />}
            value={languagePreference}
            disabled={loading || !initialConfig}
            onChange={(lng: LanguagePreference) => {
              setLanguagePreference(lng);
              void applyLanguage(lng);
              form.setFieldValue(['system', 'language'], lng);
              handleValuesChange({}, form.getFieldsValue(true));
              debouncedSave.flush();
            }}
            showSearch
            optionFilterProp="label"
            options={[{ value: 'auto', label: t('settings.languageAuto') }, ...languages.map(({ code, label }) => ({ value: code, label }))]}
          />
        </div>
      </header>
      <div className="settings-permissions"><MacPermissions /></div>
      {loading ? <div className="settings-loading"><Spin size="large" /><p>{t('settings.loading')}</p></div>
        : !initialConfig ? <Result status="warning" title={t('settings.loadError')}
          extra={<Button type="primary" onClick={loadConfig}>{t('settings.retry')}</Button>} />
        : <Form form={form} layout="vertical" initialValues={initialConfig} onValuesChange={handleValuesChange} className="settings-form">
          <Tabs
            className="settings-tabs"
            defaultActiveKey="keybinding"
            tabPosition={screens.md ? (i18n.dir() === 'rtl' ? 'right' : 'left') : 'top'}
            items={pages.map(page => ({
              key: page.key,
              label: <span className="settings-nav-label">{page.icon}<span>{t(`settings.tab.${page.key}`)}</span></span>,
              children: <main className={`settings-page settings-page-${page.key}`}>
                <div className="settings-page-heading">
                  <h1>{t(`settings.tab.${page.key}`)}</h1>
                  <p>{t(`settings.description.${page.key}`)}</p>
                </div>
                {page.content}
              </main>,
            }))}
          />
          <footer className="settings-footer">
            <span><span className="footer-dot" />{t('settings.restart')}</span>
            <span>Screen Buoy</span>
          </footer>
        </Form>}
    </div>
  );
};

export default Settings;
