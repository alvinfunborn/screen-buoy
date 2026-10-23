/** Match a supported language, preserving Chinese script preferences. */
export function matchLanguage(locale, languages) {
  const normalized = locale.toLowerCase().replaceAll('_', '-').split('.')[0];
  const base = normalized.split('-')[0];
  let code = base;
  if (base === 'zh') {
    const parts = normalized.split('-');
    // An explicit script wins over region (e.g. zh-Hans-TW).
    code = parts.includes('hant') || (!parts.includes('hans') && parts.some(part => ['tw', 'hk', 'mo'].includes(part))) ? 'zh-TW' : 'zh';
  }
  return languages.find(language => language.code === code)?.code;
}

export function resolveLanguage(preference, locales = [], languages) {
  if (languages.some(language => language.code === preference)) return preference;
  for (const locale of locales) {
    const match = matchLanguage(locale, languages);
    if (match) return match;
  }
  return 'en';
}

/** Storage may be disabled. Selection must still work for this visit. */
export function readPreference(read, languages) {
  try {
    const value = read();
    return languages.some(language => language.code === value) ? value : 'auto';
  } catch { return 'auto'; }
}

export function savePreference(write, preference) {
  try { write(preference); } catch { /* Private/restricted browser storage. */ }
}
