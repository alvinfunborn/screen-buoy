import { useEffect, useRef, useState } from 'react';
import { AutoComplete, Empty, Input, type InputRef } from 'antd';
import { SearchOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';

// Search destinations are sections, so opening a result never filters or resets form data.
const destinations = [
  ['keybinding', 'activation', 'keybinding.activation.title', ['keybinding.mainHotkey', 'keybinding.mainHotkeyTooltip']],
  ['keybinding', 'global', 'keybinding.globalKeybindings', ['keybinding.moveToHint', 'keybinding.moveAndExit', 'keybinding.leftClickAndExit', 'keybinding.holdAtHint', 'keybinding.exit', 'settings.groups.translate']],
  ['keybinding', 'atHint', 'keybinding.atHintKeybindings', ['keybinding.hover', 'keybinding.leftClick', 'keybinding.doubleClick', 'keybinding.rightClick', 'keybinding.middleClick', 'settings.groups.drag', 'settings.groups.scroll', 'settings.groups.cursor']],
  ['mouse', 'movement', 'mouse.movementStep', ['mouse.movementStepTooltip']],
  ['mouse', 'scroll', 'mouse.scrollStep', ['mouse.scrollStepTooltip']],
  ['mouse', 'drag', 'mouse.dragStep', ['mouse.dragStepTooltip']],
  ['hint', 'characters', 'hint.charsets', ['hint.charsetExtra', 'hint.charsetsTooltip']],
  ['hint', 'grid', 'hint.grid', ['hint.rows', 'hint.columns', 'hint.hintType']],
  ['hint', 'appearance', 'hint.defaultStyle', ['hint.styleCss', 'hint.defaultStyleTooltip']],
  ['hint', 'types', 'hint.types', ['hint.axRoles', 'hint.zIndex', 'hint.elementControlTypes']],
  ['keyboard', 'modifiers', 'keyboard.propagationModifier', ['keyboard.selectModifiers']],
  ['keyboard', 'keys', 'keyboard.availableKeys', ['keyboard.macKeycode', 'keyboard.keyName']],
  ['keyboard', 'neighbors', 'keyboard.leftRightMapping', ['keyboard.leftRightMappingTooltip']],
  ['uiAutomation', 'detection', 'settings.groups.detection', ['uiAutomation.collectInterval', 'uiAutomation.cacheTtl']],
  ['system', 'updates', 'updates.title', ['updates.check', 'updates.currentVersion']],
  ['system', 'startup', 'settings.groups.startup', ['system.startAtLogin', 'system.startInTray', 'system.showTrayIcon']],
  ['system', 'diagnostics', 'settings.groups.diagnostics', ['system.loggingLevel', 'system.debugMode']],
  ['system', 'application', 'settings.groups.application', ['system.restart', 'system.exit']],
] as const;

export function SettingsSearch({ disabled, isMac, onNavigate }: {
  disabled: boolean;
  isMac: boolean;
  onNavigate: (page: string, section: string) => void;
}) {
  const { t, i18n } = useTranslation();
  const input = useRef<InputRef>(null);
  const [query, setQuery] = useState('');
  const [focused, setFocused] = useState(false);

  useEffect(() => {
    const shortcut = (event: KeyboardEvent) => {
      if (!disabled && (event.metaKey || event.ctrlKey) && !event.altKey && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        input.current?.focus();
      }
    };
    window.addEventListener('keydown', shortcut);
    return () => window.removeEventListener('keydown', shortcut);
  }, [disabled]);

  const words = query.trim().toLocaleLowerCase().split(/\s+/);
  const options = destinations.flatMap(([page, section, label, keywords]) => {
    const title = t(label);
    const category = t(`settings.tab.${page}`);
    const platformKeywords = isMac && section === 'activation'
      ? [t('keybinding.activation.singleKey'), t('keybinding.activation.leftKey', { key: 'Option' }), t('keybinding.activation.gameNote')] : [];
    const searchable = [title, category, ...keywords.map(key => t(key)), ...platformKeywords].join(' ').toLocaleLowerCase();
    return words.every(word => searchable.includes(word)) ? [{
      value: `${page}:${section}`,
      'aria-label': `${title}, ${category}`,
      label: <span className="settings-search-result"><span>{title}</span><small>{category}</small></span>,
    }] : [];
  });

  return (
    <AutoComplete className="settings-search" value={query} options={options} disabled={disabled}
      open={focused && query.trim().length > 0} onChange={setQuery} onFocus={() => setFocused(true)}
      onSearch={() => setFocused(true)}
      onBlur={() => setFocused(false)} popupMatchSelectWidth={360}
      placement={i18n.dir() === 'rtl' ? 'bottomLeft' : 'bottomRight'}
      getPopupContainer={trigger => trigger.parentElement!}
      // Navigation owns focus after selection; suppress Select's delayed mouse refocus.
      dropdownRender={menu => <div onMouseDown={event => event.stopPropagation()}>{menu}</div>}
      virtual={false}
      defaultActiveFirstOption
      {...(options.length === 0 ? { 'aria-activedescendant': undefined } : {})}
      notFoundContent={<div className="settings-search-empty" role="option" aria-disabled="true" aria-label={t('settings.search.empty')}>
        <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description={t('settings.search.empty')} />
      </div>}
      onSelect={value => {
        const [page, section] = value.split(':');
        setQuery('');
        setFocused(false);
        input.current?.blur();
        onNavigate(page, section);
      }}>
      <Input ref={input} prefix={<SearchOutlined />} placeholder={t('settings.search.placeholder')}
        aria-label={t('settings.search.placeholder')} allowClear
        suffix={<kbd className="search-shortcut" aria-hidden="true">{isMac ? '⌘' : 'Ctrl'} K</kbd>}
        onKeyDown={event => {
          if (event.key === 'Escape') { setQuery(''); input.current?.blur(); }
        }} />
    </AutoComplete>
  );
}
