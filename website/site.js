import { resolveLanguage, readPreference, savePreference } from './language.js';

const [languages, english] = await Promise.all([
  fetch(new URL('./languages.json', import.meta.url)).then(response => {
    if (!response.ok) throw new Error('Cannot load language registry');
    return response.json();
  }),
  fetch(new URL('./locales/en.json', import.meta.url)).then(response => {
    if (!response.ok) throw new Error('Cannot load English catalog');
    return response.json();
  }),
]);
const messages = new Map([['en', english]]);
let languageRevision = 0;


const demo = document.querySelector('#demo');
const toggle = document.querySelector('#demo-toggle');
const status = document.querySelector('#demo-status');
const cards = [...document.querySelectorAll('[data-hint]')];
const languagePicker = document.querySelector('#site-language');
const preferenceKey = 'screen-buoy.website.language';
let preference = readPreference(() => localStorage.getItem(preferenceKey), languages);
let language = 'en';
let active = false;
let selected = null;

function renderDemo() {
  const copy = messages.get(language);
  toggle.setAttribute('aria-pressed', String(active));
  // Translations are trusted local strings; the selected name is plain text.
  toggle.textContent = `${copy[active ? 'demo.hide' : 'demo.show']} ↵`;
  status.textContent = selected
    ? copy['demo.selected'].replace('{name}', selected.querySelector('.card-name').textContent)
    : copy[active ? 'demo.active' : 'demo.idle'];
}

async function renderLanguage() {
  const version = ++languageRevision;
  let code = resolveLanguage(preference, navigator.languages, languages);
  if (!messages.has(code)) {
    try {
      const response = await fetch(new URL(`./locales/${code}.json`, import.meta.url));
      if (!response.ok) throw new Error(`Cannot load ${code} catalog`);
      messages.set(code, await response.json());
    } catch (error) {
      console.error(error);
      code = 'en';
    }
  }
  if (version !== languageRevision) return;
  language = code;
  const info = languages.find(item => item.code === language);
  const copy = messages.get(language);
  document.documentElement.lang = info.htmlLang;
  document.documentElement.dir = info.direction;
  document.querySelectorAll('[data-i18n]').forEach(element => {
    // Only the checked-in catalog is inserted as HTML, never browser/user input.
    element.innerHTML = copy[element.dataset.i18n];
  });
  for (const attribute of ['aria-label', 'alt', 'content', 'href']) {
    document.querySelectorAll(`[data-i18n-${attribute}]`).forEach(element => {
      element.setAttribute(attribute, copy[element.getAttribute(`data-i18n-${attribute}`)]);
    });
  }
  languagePicker.value = preference;
  renderDemo();
}

// Language names use their own scripts in every locale.
languagePicker.replaceChildren(languagePicker.options[0], ...languages.map(language => {
  const option = document.createElement('option');
  option.value = language.code;
  option.textContent = language.label;
  return option;
}));
languagePicker.addEventListener('change', () => {
  preference = languagePicker.value;
  savePreference(value => localStorage.setItem(preferenceKey, value), preference);
  renderLanguage();
});
window.addEventListener('languagechange', () => {
  if (preference === 'auto') renderLanguage();
});
window.addEventListener('storage', event => {
  if (event.key === preferenceKey || event.key === null) {
    preference = readPreference(() => localStorage.getItem(preferenceKey), languages);
    renderLanguage();
  }
});
await renderLanguage();

function showHints(show) {
  active = show;
  selected = null;
  cards.forEach(card => {
    card.querySelector('.hint-tag').hidden = !show;
    card.classList.remove('selected');
  });
  renderDemo();
}

function selectCard(card) {
  showHints(false);
  selected = card;
  card.classList.add('selected');
  renderDemo();
}

toggle.addEventListener('click', () => {
  showHints(!active);
  if (active) demo.focus({ preventScroll: true });
});
cards.forEach(card => card.addEventListener('click', () => selectCard(card)));
document.querySelector('#try-demo').addEventListener('click', () => {
  showHints(true);
  demo.focus({ preventScroll: true });
});
// Only handle keys while the visitor is interacting with the demo.
demo.addEventListener('keydown', event => {
  if (event.altKey || event.metaKey || event.ctrlKey || event.repeat) return;
  if (event.key === 'Escape' && active) {
    event.preventDefault();
    showHints(false);
    toggle.focus({ preventScroll: true });
  } else if (active) {
    // Physical letter keys work with a Chinese IME enabled.
    const letter = /^Key[A-Z]$/.test(event.code) ? event.code.slice(3).toLowerCase() : event.key.toLowerCase();
    const card = cards.find(item => item.dataset.hint === letter);
    if (card) {
      event.preventDefault();
      selectCard(card);
    }
  }
});
