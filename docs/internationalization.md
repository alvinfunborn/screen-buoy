# Internationalization

Screen Buoy and its website support 17 languages: English, Simplified Chinese,
Traditional Chinese, Japanese, Korean, Spanish, French, German, Portuguese,
Russian, Italian, Arabic, Hindi, Indonesian, Vietnamese, Thai and Turkish.

`website/languages.json` is the shared registry for selectable languages, native
names, HTML language tags, component locales, layout direction and tray labels.
The app bundles its translations; the website serves its own local JSON assets.
Neither product contacts a translation service.

## Language selection

- New and migrated app configurations default to `system.language = "auto"`.
  Native macOS/Windows preferred UI languages are checked in order, independently
  of the webview's browser locale. An unsupported list falls back to English.
- The website follows the browser's ordered language preferences. Its selection
  is saved under `screen-buoy.website.language` in local storage. Restricted
  storage still permits language changes for the current visit.
- Explicit selections override automatic detection and persist across restarts.
  Language changes apply immediately, including native tray labels, without
  restarting hint detection. Other configuration changes retain their existing
  restart behavior.
- `zh-Hant`, Taiwan, Hong Kong and Macao map to Traditional Chinese. Explicit
  `zh-Hans` takes precedence over region. Other regional variants match their
  supported base language (for example, `pt-BR` and `pt-PT` use Portuguese).
- Arabic mirrors the app and website. Shortcut keys, CSS and code remain LTR.
  Language handling adds no background timer or scan.

## Maintaining translations

App catalogs are in `src/locales/`; website catalogs are in `website/locales/`.
English defines the complete key set. Native language names and product names
remain recognizable in every locale. Translate user-facing text, including
accessibility labels, permission recovery instructions and update states.
Technical errors and release notes are external content and retain their source
language.

Preserve interpolation slots (`{{version}}`, `{{name}}`, `{name}`), HTML markup,
link destinations, shortcut keys and code. Website HTML comes only from reviewed,
checked-in catalogs; browser input is never inserted as HTML. New translations
may use machine assistance, but technical terminology and instructions require
contextual review; native-speaker improvements are welcome.

Run `npm run test:i18n` after changing any catalog or language matching rule.
It checks catalog coverage, placeholders, markup, shortcuts, interpolation,
website bindings and language detection. Native language/configuration tests
also run with `cargo test --locked --manifest-path src-tauri/Cargo.toml --lib`.
The normal app checks and website deployment both enforce catalog validation.

When adding a language, update the registry, app language preference type,
app catalog loader and Ant Design locale mapping; provide complete app and
website catalogs. Verify long labels on narrow screens and the interaction demo
before publishing.
