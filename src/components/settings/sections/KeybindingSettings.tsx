import React from 'react';
import { Form, Space, Typography, Spin, Input, Select } from 'antd';
import type { Config } from '../../../types/config';
import { useKeyOptions } from '../../../hooks/useKeyOptions';
import '../../../styles/global.css';
import { useTranslation } from 'react-i18next';

const { Title, Text, Paragraph } = Typography;

interface KeybindingSettingsProps {
  onValuesChange?: (changedValues: any, allValues: Config) => void;
  availableKeysData?: Record<string, number>;
}

export const KeybindingSettings: React.FC<KeybindingSettingsProps> = ({ onValuesChange, availableKeysData }) => {
  const { t } = useTranslation();
  const form = Form.useFormInstance<Config>();
  const keyOptions = useKeyOptions(availableKeysData);

  return (
    <Space direction="vertical" style={{ width: '100%' }}>
      <Form.Item layout="horizontal"
        className="config-section-title"
        label={t('keybinding.mainHotkey')}
        tooltip={t('keybinding.mainHotkeyTooltip')}
        name={['keybinding', 'hotkey_buoy']}
      >
        <Input />
      </Form.Item>

      {/* Global Shortcuts Section */}
      <Paragraph className="config-section-title">{t('keybinding.globalKeybindings')}</Paragraph>
      <Form.Item
        layout="horizontal"
        label={t('keybinding.moveToHint')}
        tooltip={t('keybinding.moveToHintTooltip')}
        name={['keybinding', 'global', 'move_to_hint']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder={t('keybinding.selectKeys')}
        />
      </Form.Item>

      <Form.Item
        layout="horizontal"
        label={t('keybinding.moveAndExit')}
        tooltip={t('keybinding.moveAndExitTooltip')}
        name={['keybinding', 'global', 'move_to_hint_exit']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder={t('keybinding.selectKeys')}
        />
      </Form.Item>

      <Form.Item
        layout="horizontal"
        label={t('keybinding.leftClickAndExit')}
        tooltip={t('keybinding.leftClickAndExitTooltip')}
        name={['keybinding', 'global', 'left_click_exit']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder={t('keybinding.selectKeys')}
        />
      </Form.Item>

      <Form.Item
        layout="horizontal"
        label={t('keybinding.holdAtHint')}
        tooltip={t('keybinding.holdAtHintTooltip')}
        name={['keybinding', 'global', 'hold_at_hint']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder={t('keybinding.selectKeys')}
        />
      </Form.Item>

      <Form.Item
        layout="horizontal"
        label={t('keybinding.exit')}
        tooltip={t('keybinding.exitTooltip')}
        name={['keybinding', 'global', 'exit']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder={t('keybinding.selectKeys')}
        />
      </Form.Item>

      {/* Move (Translate) Directions for Global */}
      <Form.Item
        layout="horizontal"
        label={t('keybinding.moveUp')}
        tooltip={t('keybinding.moveUpTooltip')}
        name={['keybinding', 'global', 'translate', 'up']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder={t('keybinding.selectKeys')} />
      </Form.Item>
      <Form.Item
        layout="horizontal"
        label={t('keybinding.moveDown')}
        tooltip={t('keybinding.moveDownTooltip')}
        name={['keybinding', 'global', 'translate', 'down']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder={t('keybinding.selectKeys')} />
      </Form.Item>
      <Form.Item
        layout="horizontal"
        label={t('keybinding.moveLeft')}
        tooltip={t('keybinding.moveLeftTooltip')}
        name={['keybinding', 'global', 'translate', 'left']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder={t('keybinding.selectKeys')} />
      </Form.Item>
      <Form.Item
        layout="horizontal"
        label={t('keybinding.moveRight')}
        tooltip={t('keybinding.moveRightTooltip')}
        name={['keybinding', 'global', 'translate', 'right']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder={t('keybinding.selectKeys')} />
      </Form.Item>

      {/* At Hint Shortcuts Section */}
      <Paragraph className="config-section-title">{t('keybinding.atHintKeybindings')}</Paragraph>
      <Form.Item
        layout="horizontal"
        label={t('keybinding.exit')}
        tooltip={t('keybinding.exitTooltip')}
        name={['keybinding', 'at_hint', 'exit']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder={t('keybinding.selectKeys')}
        />
      </Form.Item>

      <Form.Item
        layout="horizontal"
        label={t('keybinding.leftClick')}
        tooltip={t('keybinding.leftClickTooltip')}
        name={['keybinding', 'at_hint', 'left_click']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder={t('keybinding.selectKeys')}
        />
      </Form.Item>

      <Form.Item
        layout="horizontal"
        label={t('keybinding.leftClickAndExit')}
        tooltip={t('keybinding.leftClickAndExitTooltip')}
        name={['keybinding', 'at_hint', 'left_click_exit']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder={t('keybinding.selectKeys')}
        />
      </Form.Item>

      <Form.Item
        layout="horizontal"
        label={t('keybinding.doubleClick')}
        tooltip={t('keybinding.doubleClickTooltip')}
        name={['keybinding', 'at_hint', 'double_click']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder={t('keybinding.selectKeys')}
        />
      </Form.Item>

      <Form.Item
        layout="horizontal"
        label={t('keybinding.doubleClickAndExit')}
        tooltip={t('keybinding.doubleClickAndExitTooltip')}
        name={['keybinding', 'at_hint', 'double_click_exit']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder={t('keybinding.selectKeys')}
        />
      </Form.Item>

      <Form.Item
        layout="horizontal"
        label={t('keybinding.rightClick')}
        tooltip={t('keybinding.rightClickTooltip')}
        name={['keybinding', 'at_hint', 'right_click']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder={t('keybinding.selectKeys')}
        />
      </Form.Item>

      <Form.Item
        layout="horizontal"
        label={t('keybinding.rightClickAndExit')}
        tooltip={t('keybinding.rightClickAndExitTooltip')}
        name={['keybinding', 'at_hint', 'right_click_exit']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder={t('keybinding.selectKeys')}
        />
      </Form.Item>

      <Form.Item
        layout="horizontal"
        label={t('keybinding.middleClick')}
        tooltip={t('keybinding.middleClickTooltip')}
        name={['keybinding', 'at_hint', 'middle_click']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder={t('keybinding.selectKeys')}
        />
      </Form.Item>

      <Form.Item
        layout="horizontal"
        label={t('keybinding.middleClickAndExit')}
        tooltip={t('keybinding.middleClickAndExitTooltip')}
        name={['keybinding', 'at_hint', 'middle_click_exit']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder={t('keybinding.selectKeys')}
        />
      </Form.Item>

      {/* Move (Translate) Directions for At Hint */}
      <Form.Item
        layout="horizontal"
        label={t('keybinding.moveUp')}
        tooltip={t('keybinding.moveUpTooltip')}
        name={['keybinding', 'at_hint', 'translate', 'up']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder={t('keybinding.selectKeys')} />
      </Form.Item>
      <Form.Item
        layout="horizontal"
        label={t('keybinding.moveDown')}
        tooltip={t('keybinding.moveDownTooltip')}
        name={['keybinding', 'at_hint', 'translate', 'down']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder={t('keybinding.selectKeys')} />
      </Form.Item>
      <Form.Item
        layout="horizontal"
        label={t('keybinding.moveLeft')}
        tooltip={t('keybinding.moveLeftTooltip')}
        name={['keybinding', 'at_hint', 'translate', 'left']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder={t('keybinding.selectKeys')} />
      </Form.Item>
      <Form.Item
        layout="horizontal"
        label={t('keybinding.moveRight')}
        tooltip={t('keybinding.moveRightTooltip')}
        name={['keybinding', 'at_hint', 'translate', 'right']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder={t('keybinding.selectKeys')} />
      </Form.Item>
      {/* Drag Directions for At Hint */}
      <Form.Item
        layout="horizontal"
        label={t('keybinding.dragUp')}
        tooltip={t('keybinding.dragUpTooltip')}
        name={['keybinding', 'at_hint', 'drag', 'up']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder={t('keybinding.selectKeys')} />
      </Form.Item>
      <Form.Item
        layout="horizontal"
        label={t('keybinding.dragDown')}
        tooltip={t('keybinding.dragDownTooltip')}
        name={['keybinding', 'at_hint', 'drag', 'down']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder={t('keybinding.selectKeys')} />
      </Form.Item>
      <Form.Item
        layout="horizontal"
        label={t('keybinding.dragLeft')}
        tooltip={t('keybinding.dragLeftTooltip')}
        name={['keybinding', 'at_hint', 'drag', 'left']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder={t('keybinding.selectKeys')} />
      </Form.Item>
      <Form.Item
        layout="horizontal"
        label={t('keybinding.dragRight')}
        tooltip={t('keybinding.dragRightTooltip')}
        name={['keybinding', 'at_hint', 'drag', 'right']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder={t('keybinding.selectKeys')} />
      </Form.Item>
      {/* Scroll Directions for At Hint */}
      <Form.Item
        layout="horizontal"
        label={t('keybinding.scrollUp')}
        tooltip={t('keybinding.scrollUpTooltip')}
        name={['keybinding', 'at_hint', 'scroll', 'up']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder={t('keybinding.selectKeys')} />
      </Form.Item>
      <Form.Item
        layout="horizontal"
        label={t('keybinding.scrollDown')}
        tooltip={t('keybinding.scrollDownTooltip')}
        name={['keybinding', 'at_hint', 'scroll', 'down']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder={t('keybinding.selectKeys')} />
      </Form.Item>
      <Form.Item
        layout="horizontal"
        label={t('keybinding.scrollLeft')}
        tooltip={t('keybinding.scrollLeftTooltip')}
        name={['keybinding', 'at_hint', 'scroll', 'left']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder={t('keybinding.selectKeys')} />
      </Form.Item>
      <Form.Item
        layout="horizontal"
        label={t('keybinding.scrollRight')}
        tooltip={t('keybinding.scrollRightTooltip')}
        name={['keybinding', 'at_hint', 'scroll', 'right']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder={t('keybinding.selectKeys')} />
      </Form.Item>
    </Space>
  );
};
