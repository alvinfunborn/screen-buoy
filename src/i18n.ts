import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { languages, resolveLanguage } from './language.js';
import type { Config, LanguagePreference } from './types/config';
import en from './locales/en.json';


const catalogLoaders: Record<string, () => Promise<{ default: object }>> = {
  'zh': () => import('./locales/zh.json'),
  'zh-TW': () => import('./locales/zh-TW.json'),
  'ja': () => import('./locales/ja.json'),
  'ko': () => import('./locales/ko.json'),
  'es': () => import('./locales/es.json'),
  'fr': () => import('./locales/fr.json'),
  'de': () => import('./locales/de.json'),
  'pt': () => import('./locales/pt.json'),
  'ru': () => import('./locales/ru.json'),
  'it': () => import('./locales/it.json'),
  'ar': () => import('./locales/ar.json'),
  'hi': () => import('./locales/hi.json'),
  'id': () => import('./locales/id.json'),
  'vi': () => import('./locales/vi.json'),
  'th': () => import('./locales/th.json'),
  'tr': () => import('./locales/tr.json'),
};
let languageRevision = 0;

let systemLanguages: string[] = [...navigator.languages];
let preference: LanguagePreference = 'auto';

void i18n.use(initReactI18next).init({
  resources: { en: { translation: en } },
  lng: resolveLanguage('auto', systemLanguages),
  supportedLngs: languages.map(language => language.code),
  load: 'currentOnly',
  fallbackLng: 'en',
  interpolation: { escapeValue: false },
});

function updateDocument() {
  const language = languages.find(item => item.code === i18n.resolvedLanguage) ?? languages[0];
  document.documentElement.lang = language.htmlLang;
  document.documentElement.dir = language.direction;
  document.title = i18n.t('settings.windowTitle');
  if (isTauri() && languageRevision > 0) {
    void getCurrentWindow().setTitle(document.title).catch(error => console.warn('Could not update window title:', error));
  }
}
i18n.on('languageChanged', updateDocument);
updateDocument();

export async function applyLanguage(next: LanguagePreference) {
  preference = next;
  const version = ++languageRevision;
  let code = resolveLanguage(preference, systemLanguages);
  if (!i18n.hasResourceBundle(code, 'translation') && catalogLoaders[code]) {
    try {
      const catalog = await catalogLoaders[code]();
      i18n.addResourceBundle(code, 'translation', catalog.default);
    } catch (error) {
      console.error('Could not load language catalog:', code, error);
      code = 'en';
    }
  }
  if (version === languageRevision) await i18n.changeLanguage(code);
}

// Read native preferences before rendering to avoid an English-first flash.
// macOS/Windows language comes from the OS, not the embedded webview locale.
export async function initializeLanguage() {
  if (isTauri()) {
    const [config, locales] = await Promise.allSettled([
      invoke<Config>('get_config_for_frontend'),
      invoke<string[]>('get_system_languages'),
    ]);
    if (locales.status === 'fulfilled') systemLanguages = locales.value;
    if (config.status === 'fulfilled') preference = config.value.system.language ?? 'auto';
  }
  await applyLanguage(preference);
}

window.addEventListener('languagechange', async () => {
  if (isTauri()) systemLanguages = await invoke<string[]>('get_system_languages').catch(() => systemLanguages);
  else systemLanguages = [...navigator.languages];
  if (preference === 'auto') await applyLanguage(preference);
});

export default i18n;
