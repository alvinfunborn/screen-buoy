import React from 'react';
import { Form, Space, Typography, Spin, Input, Select } from 'antd';
import type { Config } from '../../../types/config';
import { useKeyOptions } from '../../../hooks/useKeyOptions';
import '../../../styles/global.css';

const { Title, Text, Paragraph } = Typography;

interface KeybindingSettingsProps {
  onValuesChange?: (changedValues: any, allValues: Config) => void;
  availableKeysData?: Record<string, number>;
}

export const KeybindingSettings: React.FC<KeybindingSettingsProps> = ({ onValuesChange, availableKeysData }) => {
  const form = Form.useFormInstance<Config>();
  const keyOptions = useKeyOptions(availableKeysData);

  return (
    <Space direction="vertical" style={{ width: '100%' }}>
      <Form.Item layout="horizontal"
        className="config-section-title"
        label="Main Hotkey"
        name={['keybinding', 'hotkey_buoy']}
      >
        <Input />
      </Form.Item>

      {/* Global Shortcuts Section */}
      <Paragraph className="config-section-title">Global Keybindings</Paragraph>
      <Form.Item
        layout="horizontal"
        label="Move to Hint"
        name={['keybinding', 'global', 'move_to_hint']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder="Select keys"
        />
      </Form.Item>

      <Form.Item
        layout="horizontal"
        label="Move and Exit"
        name={['keybinding', 'global', 'move_to_hint_exit']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder="Select keys"
        />
      </Form.Item>

      <Form.Item
        layout="horizontal"
        label="Click and Exit"
        name={['keybinding', 'global', 'left_click_exit']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder="Select keys"
        />
      </Form.Item>

      <Form.Item
        layout="horizontal"
        label="Hold at Hint"
        name={['keybinding', 'global', 'hold_at_hint']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder="Select keys"
        />
      </Form.Item>

      <Form.Item
        layout="horizontal"
        label="Exit"
        name={['keybinding', 'global', 'exit']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder="Select keys"
        />
      </Form.Item>

      {/* At Hint Shortcuts Section */}
      <Paragraph className="config-section-title">At Hint Keybindings</Paragraph>
      <Form.Item
        layout="horizontal"
        label="Exit"
        name={['keybinding', 'at_hint', 'exit']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder="Select keys"
        />
      </Form.Item>

      <Form.Item
        layout="horizontal"
        label="Left Click"
        name={['keybinding', 'at_hint', 'left_click']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder="Select keys"
        />
      </Form.Item>

      <Form.Item
        layout="horizontal"
        label="Left Click and Exit"
        name={['keybinding', 'at_hint', 'left_click_exit']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder="Select keys"
        />
      </Form.Item>

      <Form.Item
        layout="horizontal"
        label="Double Click"
        name={['keybinding', 'at_hint', 'double_click']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder="Select keys"
        />
      </Form.Item>

      <Form.Item
        layout="horizontal"
        label="Double Click and Exit"
        name={['keybinding', 'at_hint', 'double_click_exit']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder="Select keys"
        />
      </Form.Item>

      <Form.Item
        layout="horizontal"
        label="Right Click"
        name={['keybinding', 'at_hint', 'right_click']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder="Select keys"
        />
      </Form.Item>

      <Form.Item
        layout="horizontal"
        label="Right Click and Exit"
        name={['keybinding', 'at_hint', 'right_click_exit']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder="Select keys"
        />
      </Form.Item>

      <Form.Item
        layout="horizontal"
        label="Middle Click"
        name={['keybinding', 'at_hint', 'middle_click']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder="Select keys"
        />
      </Form.Item>

      <Form.Item
        layout="horizontal"
        label="Middle Click and Exit"
        name={['keybinding', 'at_hint', 'middle_click_exit']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder="Select keys"
        />
      </Form.Item>
    </Space>
  );
};
