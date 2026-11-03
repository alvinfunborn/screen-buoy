import React, { useEffect, useMemo, useRef, useState } from 'react';
import { Alert, Button, Form, Result, Select, Spin, message } from 'antd';
import {
  AimOutlined, AppstoreOutlined, CheckCircleOutlined, ControlOutlined,
  ExclamationCircleOutlined, FontSizeOutlined, GlobalOutlined,
  LoadingOutlined, SettingOutlined, ThunderboltOutlined, ReloadOutlined, RightOutlined, SaveOutlined,
} from '@ant-design/icons';
import { getVersion } from '@tauri-apps/api/app';
import { invoke } from '@tauri-apps/api/core';
import { debounce, isEqual } from 'lodash';
import { relaunch } from '@tauri-apps/plugin-process';
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
import { SettingsSearch } from './SettingsSearch';

const appIconUrl = new URL('../../../src-tauri/icons/icon.png', import.meta.url).href;

type SaveState = 'idle' | 'saving' | 'saved' | 'error';

const Settings: React.FC = () => {
  const { t } = useTranslation();
  const [languagePreference, setLanguagePreference] = useState<LanguagePreference>('auto');
  const [form] = Form.useForm<Config>();
  const [messageApi, messageContext] = message.useMessage();
  const [activePage, setActivePage] = useState('keybinding');
  const [visitedPages, setVisitedPages] = useState(['keybinding']);
  const [searchTarget, setSearchTarget] = useState<{ page: string; section: string }>();
  const [appVersion, setAppVersion] = useState('');
  const contentRef = useRef<HTMLDivElement>(null);
  const [loading, setLoading] = useState(true);
  const [initialConfig, setInitialConfig] = useState<Config | null>(null);
  const [platform, setPlatform] = useState<{ platform: string; ax_roles: string[] }>({ platform: '', ax_roles: [] });
  const [availableKeys, setAvailableKeys] = useState<Record<string, number>>();
  const [saveState, setSaveState] = useState<SaveState>('idle');
  const [saveError, setSaveError] = useState<string | null>(null);
  const [restartRequired, setRestartRequired] = useState(false);
  const [restarting, setRestarting] = useState(false);
  const startupConfig = useRef<Config | null>(null);
  const revision = useRef(0);
  const saveQueue = useRef(Promise.resolve());

  const loadConfig = async () => {
    setLoading(true);
    try {
      const [config, metadata] = await Promise.all([
        invoke<Config>('get_config_for_frontend'),
        invoke<{ platform: string; ax_roles: string[] }>('get_settings_platform'),
      ]);
      setPlatform(metadata);
      setInitialConfig(config);
      startupConfig.current ??= config;
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
  useEffect(() => { void getVersion().then(setAppVersion).catch(() => {}); }, []);

  const navigate = (page: string, section?: string) => {
    setActivePage(page);
    setVisitedPages(current => current.includes(page) ? current : [...current, page]);
    setSearchTarget(section ? { page, section } : undefined);
    if (!section) contentRef.current?.scrollTo({ top: 0 });
  };

  useEffect(() => {
    if (!searchTarget) return;
    // Tabs reveal previously mounted content asynchronously. Focus only once visible.
    let frame: number;
    let attempts = 0;
    const reveal = () => {
      const section = document.getElementById(`${searchTarget.page}-${searchTarget.section}`);
      if (section?.getClientRects().length) {
        section.scrollIntoView({ block: 'start' });
        section.focus({ preventScroll: true });
      } else if (++attempts < 30) {
        frame = requestAnimationFrame(reveal);
      }
    };
    frame = requestAnimationFrame(reveal);
    return () => cancelAnimationFrame(frame);
  }, [searchTarget]);

  // Keep the existing 500 ms autosave stable across presentation re-renders.
  // Serialize writes so a slower earlier save cannot overwrite the latest edit.
  const debouncedSave = useMemo(() => debounce((values: Config, version: number) => {
    saveQueue.current = saveQueue.current.then(async () => {
      if (revision.current !== version || getHintPrefixConflicts(values.hint).length) return;
      try {
        await invoke('save_config_for_frontend', { config: values });
        // Language applies immediately; all other settings may be cached until restart.
        const withoutLanguage = (config: Config) => ({
          ...config, system: { ...config.system, language: undefined },
        });
        setRestartRequired(startupConfig.current !== null &&
          !isEqual(withoutLanguage(values), withoutLanguage(startupConfig.current)));
        if (revision.current === version) { setSaveState('saved'); setSaveError(null); }
      } catch (err) {
        if (revision.current === version) {
          setSaveState('error');
          const conflicts = getPrefixConflictsFromError(err);
          setSaveError(conflicts
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
      setSaveError(i18n.t('hint.charsetPrefixConflict', { characters: getHintPrefixConflicts(values.hint).join(', ') }));
      return;
    }
    setSaveError(null);
    setSaveState('saving');
    debouncedSave(values, version);
  };

  const restart = async () => {
    setRestarting(true);
    try { await relaunch(); }
    catch (error) { messageApi.error(i18n.t('settings.runtimeError', { error: String(error) })); }
    finally { setRestarting(false); }
  };

  const retrySave = () => {
    handleValuesChange({}, form.getFieldsValue(true));
    debouncedSave.flush();
  };

  const pages = [
    { key: 'keybinding', icon: <ThunderboltOutlined />, content: <KeybindingSettings isMac={platform.platform === 'macos'} availableKeysData={availableKeys} requestedSection={searchTarget?.page === 'keybinding' ? searchTarget : undefined} /> },
    { key: 'mouse', icon: <AimOutlined />, content: <MouseSettings onValuesChange={handleValuesChange} availableKeysData={availableKeys} /> },
    { key: 'hint', icon: <FontSizeOutlined />, content: <HintSettings onValuesChange={handleValuesChange} isMac={platform.platform === 'macos'} axRoles={platform.ax_roles} /> },
    { key: 'uiAutomation', icon: <AppstoreOutlined />, content: <UiAutomationSettings /> },
    { key: 'keyboard', icon: <ControlOutlined />, content: <KeyboardSettings onValuesChange={handleValuesChange} isMac={platform.platform === 'macos'} requestedSection={searchTarget?.page === 'keyboard' ? searchTarget : undefined} /> },
    { key: 'system', icon: <SettingOutlined />, content: <SystemSettings onRestart={restart} restarting={restarting} restartDisabled={saveState === 'saving' || saveState === 'error'} /> },
  ];

  return (
    <div className="settings-shell">
      {messageContext}
      <aside className="settings-sidebar">
        <div className="settings-brand">
          <img className="brand-mark" src={appIconUrl} alt="" aria-hidden="true" />
          <div><strong>ScreenBuoy</strong><span>{t('settings.title')}</span></div>
        </div>
        <nav className="settings-navigation" aria-label={t('settings.title')}>
          {[
            { title: 'interaction', pages: pages.slice(0, 3) },
            { title: 'preferences', pages: pages.slice(3) },
          ].map(group => <div className="settings-nav-group" key={group.title}>
            <p className="settings-nav-caption">{t(`settings.navigation.${group.title}`)}</p>
            {group.pages.map(page => <button type="button" key={page.key}
              className={`settings-nav-item${activePage === page.key ? ' is-active' : ''}`}
              aria-current={activePage === page.key ? 'page' : undefined}
              aria-controls={`settings-panel-${page.key}`} disabled={loading || !initialConfig}
              onClick={() => navigate(page.key)}>
              {page.icon}<span>{t(`settings.tab.${page.key}`)}</span>
              {activePage === page.key && <span className="nav-active-dot" aria-hidden="true" />}
            </button>)}
          </div>)}
        </nav>
        <div className="settings-sidebar-bottom">
          <label className="sidebar-language-label" htmlFor="settings-language"><GlobalOutlined />{t('settings.language')}</label>
          <Select
            id="settings-language"
            className="language-select"
            aria-label={t('settings.language')}
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
          <div className="settings-version"><span>ScreenBuoy</span>{appVersion && <span>v{appVersion}</span>}</div>
        </div>
      </aside>
      <div className="settings-workspace">
        <header className="settings-header">
          <div className="settings-breadcrumb"><span>{t('settings.title')}</span><RightOutlined /><strong>{t(`settings.tab.${activePage}`)}</strong></div>
          <SettingsSearch disabled={loading || !initialConfig} isMac={platform.platform === 'macos'} onNavigate={navigate} />
        </header>
        <div className="settings-scroll-area" ref={contentRef}>
          <div className="settings-permissions"><MacPermissions /></div>
          {saveError && <Alert className="settings-save-alert" type="error" showIcon message={saveError}
            action={<Button size="small" onClick={retrySave}>{t('settings.retry')}</Button>} />}
          {loading ? <div className="settings-loading"><Spin size="large" /><p>{t('settings.loading')}</p></div>
            : !initialConfig ? <Result status="warning" title={t('settings.loadError')}
              extra={<Button type="primary" onClick={loadConfig}>{t('settings.retry')}</Button>} />
            : <Form form={form} layout="vertical" initialValues={initialConfig} onValuesChange={handleValuesChange} className="settings-form">
              <main>
                {pages.map(page => (
                  <section key={page.key} id={`settings-panel-${page.key}`} hidden={activePage !== page.key}
                    className={`settings-page settings-page-${page.key}`} aria-labelledby={`settings-title-${page.key}`}>
                    <div className="settings-page-heading">
                      <div><h1 id={`settings-title-${page.key}`}>{t(`settings.tab.${page.key}`)}</h1>
                      <p>{t(`settings.description.${page.key}`)}</p>
                      </div>
                      <span className="page-heading-icon" aria-hidden="true">{page.icon}</span>
                    </div>
                    {visitedPages.includes(page.key) && page.content}
                  </section>
                ))}
              </main>
            </Form>}
        </div>
        {initialConfig && <footer className={`settings-footer${restartRequired ? ' has-pending-changes' : ''}`}>
          <span className={`save-status save-status-${saveState}`} role="status" aria-live="polite">
            {saveState === 'saving' ? <LoadingOutlined /> : saveState === 'error' ? <ExclamationCircleOutlined /> : saveState === 'saved' ? <CheckCircleOutlined /> : <SaveOutlined />}
            <span>{t(`settings.save.${saveState}`)}</span>
            {saveState === 'error' && <Button type="link" size="small" onClick={retrySave}>{t('settings.retry')}</Button>}
          </span>
          <div className="settings-footer-action">
            <span>{t(restartRequired ? 'settings.restartRequired' : 'settings.restart')}</span>
            {restartRequired && <Button size="small" type="primary" icon={<ReloadOutlined />}
              disabled={saveState !== 'saved'} loading={restarting} onClick={restart}>{t('system.restart')}</Button>}
          </div>
        </footer>}
      </div>
    </div>
  );
};

export default Settings;
