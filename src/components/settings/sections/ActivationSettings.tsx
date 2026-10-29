import { Form, Input, Select, Switch } from 'antd';
import { KeyOutlined, ThunderboltOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import type { KeybindingConfig } from '../../../types/config';
import './ActivationSettings.css';

const modifierKeys = [
  ['option', 'Option', '⌥'], ['command', 'Command', '⌘'],
  ['control', 'Control', '⌃'], ['shift', 'Shift', '⇧'],
] as const;

export function ActivationSettings({ isMac }: { isMac: boolean }) {
  const { t } = useTranslation();
  const form = Form.useFormInstance();
  const singleEnabled = Form.useWatch(['keybinding', 'single_key_enabled'], form) ?? false;
  const hotkeyEnabled = Form.useWatch(['keybinding', 'hotkey_enabled'], form) ?? true;
  const selectedKey: KeybindingConfig['single_key'] = Form.useWatch(['keybinding', 'single_key'], form) ?? 'left_option';
  const options = modifierKeys.flatMap(([name, key]) => ['left', 'right'].map(side => ({
    value: `${side}_${name}`, label: t(`keybinding.activation.${side}Key`, { key }),
  })));
  const symbol = modifierKeys.find(([name]) => selectedKey.endsWith(name))?.[2] ?? '⌥';
  const bothOff = !hotkeyEnabled && (!isMac || !singleEnabled);

  return (
    <section id="keybinding-activation" className="hint-activation" aria-labelledby="activation-title" tabIndex={-1}>
      <div className="activation-header">
        <span className="activation-icon" aria-hidden="true"><ThunderboltOutlined /></span>
        <div className="activation-heading">
          <h2 id="activation-title">{t('keybinding.activation.title')}</h2>
          <p>{t('keybinding.activation.description')}</p>
        </div>
      </div>
      <div className="activation-methods">
        {isMac && <div className="activation-tap">
          <div className="activation-method-row">
            <kbd className="activation-keycap" aria-hidden="true">{symbol}</kbd>
            <div className="activation-method-label">
              <label htmlFor="keybinding_single_key_enabled">{t('keybinding.activation.singleKey')}</label>
              <p id="single-key-description">{t('keybinding.activation.singleDescription')}</p>
            </div>
            <Form.Item name={['keybinding', 'single_key_enabled']} valuePropName="checked" className="activation-switch">
              <Switch id="keybinding_single_key_enabled" aria-label={t('keybinding.activation.singleKey')}
                aria-describedby="single-key-description single-key-rules activation-apply-note" />
            </Form.Item>
          </div>
          <Form.Item name={['keybinding', 'single_key']} className="activation-key-select" layout="horizontal"
            label={t('keybinding.activation.chooseKey')}>
            <Select options={options} disabled={!singleEnabled} aria-label={t('keybinding.activation.chooseKey')} />
          </Form.Item>
          <div className="activation-tap-detail">
            <p id="single-key-rules">{t('keybinding.activation.singleRules')}</p>
          </div>
        </div>}
        <div className="activation-combination">
          <div className="activation-method-row">
            <span className="activation-keycap activation-keycap-combination" aria-hidden="true"><KeyOutlined /></span>
            <div className="activation-method-label">
              <label htmlFor="keybinding_hotkey_enabled">{t('keybinding.activation.combination')}</label>
              <p id="activation-shortcut-description">{t('keybinding.activation.combinationDescription')}</p>
            </div>
            <Form.Item name={['keybinding', 'hotkey_enabled']} valuePropName="checked" className="activation-switch">
              <Switch id="keybinding_hotkey_enabled" aria-label={t('keybinding.activation.combination')}
                aria-describedby="activation-shortcut-description activation-apply-note" />
            </Form.Item>
          </div>
          <Form.Item name={['keybinding', 'hotkey_buoy']} className="hotkey-field" layout="horizontal"
            label={t('keybinding.activation.shortcutKeys')}>
            <Input id="keybinding_hotkey_buoy" aria-label={t('keybinding.activation.shortcutKeys')}
              disabled={!hotkeyEnabled} spellCheck={false} autoComplete="off" />
          </Form.Item>
        </div>
      </div>
      {isMac && <p className="activation-game-note">{t('keybinding.activation.gameNote')}</p>}
      <div className="activation-note" id="activation-apply-note">
        <p>{t('keybinding.activation.applyNote')}</p>
        {bothOff && <p className="activation-off-note" role="status">{t('keybinding.activation.bothOff')}</p>}
      </div>
    </section>
  );
}
