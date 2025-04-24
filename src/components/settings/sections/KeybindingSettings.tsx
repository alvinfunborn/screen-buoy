import React from 'react';
import { Form, Space, Typography, Collapse, Select } from 'antd';

const { Title } = Typography;
const { Panel } = Collapse;

export const KeybindingSettings: React.FC = () => {
  const keyOptions = [
    { label: 'Ctrl', value: 'ctrl' },
    { label: 'Shift', value: 'shift' },
    { label: 'Alt', value: 'alt' },
    { label: 'Space', value: 'space' },
    { label: 'Enter', value: 'enter' },
    { label: 'Escape', value: 'escape' },
    { label: 'Tab', value: 'tab' },
    { label: 'ArrowUp', value: 'arrowup' },
    { label: 'ArrowDown', value: 'arrowdown' },
    { label: 'ArrowLeft', value: 'arrowleft' },
    { label: 'ArrowRight', value: 'arrowright' },
  ];

  return (
    <Space direction="vertical" style={{ width: '100%' }}>
      <Title level={3}>Keybinding Settings</Title>

      <Form.Item
        label="Main Hotkey"
        name={['keybinding', 'hotkey_buoy']}
      >
        <Select
          mode="tags"
          style={{ width: '100%' }}
          options={keyOptions}
          placeholder="Select keys"
        />
      </Form.Item>

      <Collapse>
        <Panel header="Global Shortcuts" key="global">
          <Form.Item
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
        </Panel>

        <Panel header="At Hint Shortcuts" key="at_hint">
          <Form.Item
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
        </Panel>
      </Collapse>
    </Space>
  );
};
