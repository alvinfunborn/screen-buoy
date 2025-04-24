import React from 'react';
import { Form, Space, Typography, Collapse, Select, Spin, Input } from 'antd';
import type { Config } from '../../../types/config';

const { Title } = Typography;
const { Panel } = Collapse;

interface KeybindingSettingsProps {
  loading?: boolean;
}

export const KeybindingSettings: React.FC<KeybindingSettingsProps> = ({ loading }) => {
  if (loading) {
    return <Spin />;
  }

  const form = Form.useFormInstance<Config>();
  const values = form.getFieldsValue();
  const keyOptions = values?.keyboard?.available_key ? 
    Object.entries(values.keyboard.available_key).sort((a, b) => a[1] - b[1]).map(([key, _]) => ({
      label: key,
      value: key
    })) : [];
  keyOptions.push({
    label: "HintKey",
    value: "HintKey"
  });
  keyOptions.push({
    label: "HintRightKey",
    value: "HintRightKey"
  });
  keyOptions.push({
    label: "HintLeftKey",
    value: "HintLeftKey"
  });

  return (
    <Space direction="vertical" style={{ width: '100%' }}>
      <Title level={3}>Keybinding Settings</Title>

      <Form.Item
        label="Main Hotkey"
        name={['keybinding', 'hotkey_buoy']}
      >
        <Input/>
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
