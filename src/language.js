import languages from '../website/languages.json';
import { resolveLanguage as resolve } from '../website/language.js';
export { languages };
export function resolveLanguage(preference, locales = []) {
  return resolve(preference, locales, languages);
}
