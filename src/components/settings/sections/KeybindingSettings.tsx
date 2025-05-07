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
        tooltip="The main hotkey to activate and show hints."
        name={['keybinding', 'hotkey_buoy']}
      >
        <Input />
      </Form.Item>

      {/* Global Shortcuts Section */}
      <Paragraph className="config-section-title">Global Keybindings</Paragraph>
      <Form.Item
        layout="horizontal"
        label="Move to Hint"
        tooltip="Move the mouse cursor to the selected hint."
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
        tooltip="Move the mouse cursor to the selected hint and exit hint mode."
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
        tooltip="Left click the selected hint and exit hint mode."
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
        tooltip="Enter Hold mode at the selected hint position."
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
        tooltip="Exit hint mode."
        name={['keybinding', 'global', 'exit']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder="Select keys"
        />
      </Form.Item>

      {/* Move (Translate) Directions for Global */}
      <Form.Item
        layout="horizontal"
        label="Move Up"
        tooltip="Move all hints up."
        name={['keybinding', 'global', 'translate', 'up']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder="Select keys" />
      </Form.Item>
      <Form.Item
        layout="horizontal"
        label="Move Down"
        tooltip="Move all hints down."
        name={['keybinding', 'global', 'translate', 'down']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder="Select keys" />
      </Form.Item>
      <Form.Item
        layout="horizontal"
        label="Move Left"
        tooltip="Move all hints left."
        name={['keybinding', 'global', 'translate', 'left']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder="Select keys" />
      </Form.Item>
      <Form.Item
        layout="horizontal"
        label="Move Right"
        tooltip="Move all hints right."
        name={['keybinding', 'global', 'translate', 'right']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder="Select keys" />
      </Form.Item>

      {/* At Hint Shortcuts Section */}
      <Paragraph className="config-section-title">At Hint Keybindings</Paragraph>
      <Form.Item
        layout="horizontal"
        label="Exit"
        tooltip="Exit hint mode."
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
        tooltip="Left click at the hint position."
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
        tooltip="Left click at the hint position and exit hint mode."
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
        tooltip="Double left click at the hint position."
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
        tooltip="Double left click at the hint position and exit hint mode."
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
        tooltip="Right click at the hint position."
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
        tooltip="Right click at the hint position and exit hint mode."
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
        tooltip="Middle click at the hint position."
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
        tooltip="Middle click at the hint position and exit hint mode."
        name={['keybinding', 'at_hint', 'middle_click_exit']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder="Select keys"
        />
      </Form.Item>

      {/* Move (Translate) Directions for At Hint */}
      <Form.Item
        layout="horizontal"
        label="Move Up"
        tooltip="Move all hints up."
        name={['keybinding', 'at_hint', 'translate', 'up']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder="Select keys" />
      </Form.Item>
      <Form.Item
        layout="horizontal"
        label="Move Down"
        tooltip="Move all hints down."
        name={['keybinding', 'at_hint', 'translate', 'down']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder="Select keys" />
      </Form.Item>
      <Form.Item
        layout="horizontal"
        label="Move Left"
        tooltip="Move all hints left."
        name={['keybinding', 'at_hint', 'translate', 'left']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder="Select keys" />
      </Form.Item>
      <Form.Item
        layout="horizontal"
        label="Move Right"
        tooltip="Move all hints right."
        name={['keybinding', 'at_hint', 'translate', 'right']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder="Select keys" />
      </Form.Item>
      {/* Drag Directions for At Hint */}
      <Form.Item
        layout="horizontal"
        label="Drag Up"
        tooltip="Drag up at the hint position."
        name={['keybinding', 'at_hint', 'drag', 'up']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder="Select keys" />
      </Form.Item>
      <Form.Item
        layout="horizontal"
        label="Drag Down"
        tooltip="Drag down at the hint position."
        name={['keybinding', 'at_hint', 'drag', 'down']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder="Select keys" />
      </Form.Item>
      <Form.Item
        layout="horizontal"
        label="Drag Left"
        tooltip="Drag left at the hint position."
        name={['keybinding', 'at_hint', 'drag', 'left']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder="Select keys" />
      </Form.Item>
      <Form.Item
        layout="horizontal"
        label="Drag Right"
        tooltip="Drag right at the hint position."
        name={['keybinding', 'at_hint', 'drag', 'right']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder="Select keys" />
      </Form.Item>
      {/* Scroll Directions for At Hint */}
      <Form.Item
        layout="horizontal"
        label="Scroll Up"
        tooltip="Scroll up at the hint position."
        name={['keybinding', 'at_hint', 'scroll', 'up']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder="Select keys" />
      </Form.Item>
      <Form.Item
        layout="horizontal"
        label="Scroll Down"
        tooltip="Scroll down at the hint position."
        name={['keybinding', 'at_hint', 'scroll', 'down']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder="Select keys" />
      </Form.Item>
      <Form.Item
        layout="horizontal"
        label="Scroll Left"
        tooltip="Scroll left at the hint position."
        name={['keybinding', 'at_hint', 'scroll', 'left']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder="Select keys" />
      </Form.Item>
      <Form.Item
        layout="horizontal"
        label="Scroll Right"
        tooltip="Scroll right at the hint position."
        name={['keybinding', 'at_hint', 'scroll', 'right']}
      >
        <Select mode="tags" style={{ width: '100%' }} options={keyOptions} placeholder="Select keys" />
      </Form.Item>
    </Space>
  );
};
