import React, { useEffect, useState } from 'react';
import { Form, Select, Tabs } from 'antd';
import { useTranslation } from 'react-i18next';
import { useKeyOptions } from '../../../hooks/useKeyOptions';
import { SettingSection } from '../SettingSection';
import { ActivationSettings } from './ActivationSettings';

interface KeybindingSettingsProps {
  isMac: boolean;
  availableKeysData?: Record<string, number>;
  requestedSection?: { section: string };
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

export const KeybindingSettings: React.FC<KeybindingSettingsProps> = ({ isMac, availableKeysData, requestedSection }) => {
  const { t } = useTranslation();
  const [scope, setScope] = useState('global');
  useEffect(() => {
    if (requestedSection?.section === 'global' || requestedSection?.section === 'atHint') setScope(requestedSection.section);
  }, [requestedSection]);
  const keyOptions = useKeyOptions(availableKeysData);
  const field = (path: string[], label: string, tooltip = `${label}Tooltip`) => (
    <Form.Item key={path.join('.')} className="setting-row shortcut-row" layout="horizontal"
      label={t(`keybinding.${label}`)} tooltip={t(`keybinding.${tooltip}`)} name={['keybinding', ...path]}>
      <Select mode="tags" options={keyOptions} placeholder={t('keybinding.selectKeys')} popupMatchSelectWidth={false} />
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
      <ActivationSettings isMac={isMac} />
      <Tabs className="binding-tabs" activeKey={scope} onChange={setScope} items={[
        { key: 'global', label: t('settings.bindingScope.global'), forceRender: true, children:
      <SettingSection id="keybinding-global" title={t('keybinding.globalKeybindings')} description={t('settings.groups.global')}>
        {globalActions.map(([name, label]) => field(['global', name], label))}
        <h4 className="setting-subheading">{t('settings.groups.translate')}</h4>
        {directionFields('global', 'translate', 'move')}
      </SettingSection> },
        { key: 'atHint', label: t('settings.bindingScope.atHint'), forceRender: true, children:
      <SettingSection id="keybinding-atHint" title={t('keybinding.atHintKeybindings')} description={t('settings.groups.atHint')}>
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
      </SettingSection> },
      ]} />
    </div>
  );
};
