import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { createInstance } from 'i18next';
import { matchLanguage, resolveLanguage, readPreference, savePreference } from '../website/language.js';

const read = path => JSON.parse(readFileSync(new URL(path, import.meta.url), 'utf8'));
const languages = read('../website/languages.json');
const flatten = (object, prefix = '') => Object.fromEntries(Object.entries(object).flatMap(([key, value]) =>
  typeof value === 'object' ? Object.entries(flatten(value, `${prefix}${key}.`)) : [[`${prefix}${key}`, value]]));
const placeholders = value => [...value.matchAll(/\{\{[^}]+\}\}|\{name\}/g)].map(match => match[0]).sort();
const tags = value => [...value.matchAll(/<[^>]+>/g)].map(match => match[0]);

test('language detection respects scripts, regions and ordered preferences', () => {
  for (const [locale, expected] of Object.entries({ 'zh-Hant-HK': 'zh-TW', 'zh-Hans-TW': 'zh',
    'ZH_tw': 'zh-TW', 'zh-CN': 'zh', 'en_GB.UTF-8': 'en', 'pt-BR': 'pt', 'pt-PT': 'pt',
    'ar-SA': 'ar', 'hi-IN': 'hi', 'ja-JP': 'ja', 'xx-XX': undefined })) {
    assert.equal(matchLanguage(locale, languages), expected, locale);
  }
  assert.equal(resolveLanguage('auto', ['xx-XX', 'fr-FR', 'en-US'], languages), 'fr');
  assert.equal(resolveLanguage('auto', [], languages), 'en');
  assert.equal(resolveLanguage('auto', ['xx-XX'], languages), 'en');
  for (const { code } of languages) assert.equal(resolveLanguage(code, ['en-US'], languages), code);
});

test('restricted storage and outdated preferences preserve usable selection', () => {
  assert.equal(readPreference(() => { throw Error('blocked'); }, languages), 'auto');
  assert.equal(readPreference(() => 'unsupported', languages), 'auto');
  assert.equal(readPreference(() => 'zh-TW', languages), 'zh-TW');
  assert.doesNotThrow(() => savePreference(() => { throw Error('blocked'); }, 'ar'));
});

for (const folder of ['src/locales', 'website/locales']) {
  const base = flatten(read(`../${folder}/en.json`));
  test(`${folder}: every selectable language has a complete, safe catalog`, async () => {
    assert.deepEqual(readdirSync(new URL(`../${folder}/`, import.meta.url)).filter(name => name.endsWith('.json')).sort(),
      languages.map(({ code }) => `${code}.json`).sort());
    for (const { code } of languages) {
      const catalog = read(`../${folder}/${code}.json`);
      const flat = flatten(catalog);
      assert.deepEqual(Object.keys(flat).sort(), Object.keys(base).sort(), code);
      if (code !== 'en') {
        const translated = Object.keys(base).filter(key => flat[key] !== base[key]).length;
        assert.ok(translated > Object.keys(base).length * 0.9, `${code}: catalog must contain translations, not an English copy`);
      }
      for (const [key, value] of Object.entries(flat)) {
        const label = `${code}: ${key}`;
        assert.equal(typeof value, 'string', label);
        assert.ok(value.trim(), label);
        assert.ok(!value.includes('```') && !value.includes('**'), `${label}: unexpected Markdown formatting`);
        assert.ok(!/ZXQ\d|<\|?\/?(?:end_of_turn|start_of_turn)/.test(value), `${label}: translation artifact`);
        assert.deepEqual(placeholders(value), placeholders(base[key]), label);
        if (base[key].includes('Screen Buoy')) assert.ok(value.includes('Screen Buoy'), `${label}: product name`);
        if (folder === 'src/locales') assert.ok(!/<[^>]+>/.test(value), `${label}: unexpected HTML`);
        // Translations must not change markup, link targets, code or shortcut keys.
        if (folder === 'website/locales') {
          assert.deepEqual(tags(value), tags(base[key]), label);
          for (const literal of base[key].match(/<(kbd|code)\b[^>]*>.*?<\/\1>/g) ?? []) assert.ok(value.includes(literal), label);
        }
      }
      if (folder === 'src/locales') {
        const instance = createInstance();
        await instance.init({ lng: code, resources: { [code]: { translation: catalog } }, fallbackLng: false });
        for (const key of ['mouse.addStep', 'settings.removeItem', 'updates.available', 'hint.placeholderTypeStyle']) {
          const value = instance.t(key, { label: 'TEST', name: 'TEST', version: 'TEST', typeName: 'TEST' });
          assert.ok(value.includes('TEST') && !value.includes('{{'), `${code}: ${key}`);
        }
      }
    }
  });
}

test('every website translation binding exists in the catalog', () => {
  const html = readFileSync(new URL('../website/index.html', import.meta.url), 'utf8');
  const base = read('../website/locales/en.json');
  for (const [, key] of html.matchAll(/data-i18n(?:-(?:aria-label|alt|content|href))?="([^"]+)"/g)) assert.ok(key in base, key);
  assert.equal(new Set(languages.map(item => item.code)).size, languages.length);
  assert.ok(languages.every(item => item.tray.length === 3 && ['ltr', 'rtl'].includes(item.direction)));
});

test('app language preference, loaders and component locales cover the registry', () => {
  const types = readFileSync(new URL('../src/types/config.ts', import.meta.url), 'utf8');
  const choices = types.match(/export type LanguagePreference = ([^;]+);/)[1].match(/'([^']+)'/g).map(s => s.slice(1, -1));
  assert.deepEqual(choices.sort(), ['auto', ...languages.map(item => item.code)].sort());
  const loaders = readFileSync(new URL('../src/i18n.ts', import.meta.url), 'utf8');
  const app = readFileSync(new URL('../src/App.tsx', import.meta.url), 'utf8');
  for (const { code, antd } of languages) {
    if (code !== 'en') assert.ok(loaders.includes(`import('./locales/${code}.json')`), code);
    assert.ok(app.includes(`'antd/locale/${antd}'`), code);
  }
});
