import React from 'react';
import { Form, Input, Select } from 'antd';
import { useTranslation } from 'react-i18next';
import { useKeyOptions } from '../../../hooks/useKeyOptions';
import { SettingSection } from '../SettingSection';

interface KeybindingSettingsProps {
  availableKeysData?: Record<string, number>;
}

type Binding = [field: string, label: string];
const globalActions: Binding[] = [
  ['move_to_hint', 'moveToHint'], ['move_to_hint_exit', 'moveAndExit'],
  ['left_click_exit', 'leftClickAndExit'], ['hold_at_hint', 'holdAtHint'], ['exit', 'exit'],
];
const hintActions: Binding[] = [
  ['hover', 'hover'],
  ['exit', 'exit'], ['left_click', 'leftClick'], ['left_click_exit', 'leftClickAndExit'],
  ['double_click', 'doubleClick'], ['double_click_exit', 'doubleClickAndExit'],
  ['right_click', 'rightClick'], ['right_click_exit', 'rightClickAndExit'],
  ['middle_click', 'middleClick'], ['middle_click_exit', 'middleClickAndExit'],
];
const directions = ['up', 'down', 'left', 'right'] as const;

export const KeybindingSettings: React.FC<KeybindingSettingsProps> = ({ availableKeysData }) => {
  const { t } = useTranslation();
  const keyOptions = useKeyOptions(availableKeysData);
  const field = (path: string[], label: string, tooltip = `${label}Tooltip`) => (
    <Form.Item key={path.join('.')} className="setting-row shortcut-row" layout="horizontal"
      label={t(`keybinding.${label}`)} tooltip={t(`keybinding.${tooltip}`)} name={['keybinding', ...path]}>
      <Select mode="tags" options={keyOptions} placeholder={t('keybinding.selectKeys')} />
    </Form.Item>
  );
  const directionFields = (scope: string, action: string, label: string) => (
    <div className="direction-fields">
      {directions.map(direction => field([scope, action, direction], `${label}${direction[0].toUpperCase()}${direction.slice(1)}`,
        scope === 'at_hint' && action === 'translate' ? 'cursorMoveTooltip' : undefined))}
    </div>
  );

  return (
    <div className="settings-stack">
      <SettingSection title={t('keybinding.mainHotkey')} description={t('settings.groups.activation')} className="activation-section">
        <Form.Item name={['keybinding', 'hotkey_buoy']} className="hotkey-field">
          <Input aria-label={t('keybinding.mainHotkey')} spellCheck={false} />
        </Form.Item>
        <p className="setting-note">{t('keybinding.mainHotkeyTooltip')}</p>
      </SettingSection>
      <SettingSection title={t('keybinding.globalKeybindings')} description={t('settings.groups.global')}>
        {globalActions.map(([name, label]) => field(['global', name], label))}
        <h4 className="setting-subheading">{t('settings.groups.translate')}</h4>
        {directionFields('global', 'translate', 'move')}
      </SettingSection>
      <SettingSection title={t('keybinding.atHintKeybindings')} description={t('settings.groups.atHint')}>
        {hintActions.map(([name, label]) => field(['at_hint', name], label))}
        <h4 className="setting-subheading">{t('settings.groups.cursor')}</h4>
        <p className="setting-note">{t('keybinding.cursorMoveTooltip')}</p>
        {directionFields('at_hint', 'translate', 'move')}
        <h4 className="setting-subheading">{t('settings.groups.drag')}</h4>
        <p className="setting-note">{t('settings.groups.dragDescription')}</p>
        {directionFields('at_hint', 'drag', 'drag')}
        <h4 className="setting-subheading">{t('settings.groups.scroll')}</h4>
        <p className="setting-note">{t('settings.groups.scrollDescription')}</p>
        {directionFields('at_hint', 'scroll', 'scroll')}
      </SettingSection>
    </div>
  );
};
