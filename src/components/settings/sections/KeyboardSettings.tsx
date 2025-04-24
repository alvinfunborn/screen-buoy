import React from 'react';
import { Form, Space, Typography, Button, Input, List, Spin, Collapse, Select } from 'antd';
import { MinusCircleOutlined, PlusOutlined } from '@ant-design/icons';
import type { Config, LeftRightConfig } from '../../../types/config';

const { Title } = Typography;
const { Panel } = Collapse;

interface KeyboardSettingsProps {
  loading?: boolean;
}

interface KeyEntry {
  key: string;
  virtual_key: number;
}

interface LeftRightEntry {
  key: string;
  left: string | null;
  right: string | null;
}

export const KeyboardSettings: React.FC<KeyboardSettingsProps> = ({ loading }) => {
  if (loading) {
    return <Spin />;
  }

  const form = Form.useFormInstance<Config>();
  // 直接从表单实例获取值
  const formValues = form.getFieldsValue(true) as Config;

  const hint_key = "HintKey";
  const hint_right_key = "HintRightKey";
  const hint_left_key = "HintLeftKey";
  
  const availableKeys = formValues?.keyboard?.available_key || {};
  const keyEntries = Object.entries(availableKeys)
    .sort((a, b) => (a[1] as number) - (b[1] as number))
    .map(([key, value]) => ({
      key,
      virtual_key: value as number
    }));

  const updateKeyVk = (entries: KeyEntry[]) => {
    const newValue = entries.reduce((acc, { key, virtual_key }) => ({
      ...acc,
      [key]: virtual_key
    }), {});
    const currentValues = form.getFieldsValue(true) as Config;
    form.setFieldsValue({
      keyboard: {
        ...currentValues.keyboard,
        available_key: newValue
      }
    });
  };

  const keyLeftRight = formValues?.keyboard?.map_left_right || {};
  const keyLeftRightEntries = Object.entries(keyLeftRight)
    .map(([key, value]) => ({
      key,
      left: (value as LeftRightConfig)?.left || null,
      right: (value as LeftRightConfig)?.right || null
    }));

  const updateLeftRight = (entries: LeftRightEntry[]) => {
    const newValue = entries.reduce((acc, { key, left, right }) => ({
      ...acc,
      [key]: { left, right }
    }), {});
    const currentValues = form.getFieldsValue(true) as Config;
    form.setFieldsValue({
      keyboard: {
        ...currentValues.keyboard,
        map_left_right: newValue
      }
    });
  };

  return (
    <Space direction="vertical" style={{ width: '100%' }}>
      <Title level={3}>Keyboard Settings</Title>

      <Form.Item
        layout="horizontal"
        label="Propagation Modifier"
        name={['keyboard', 'propagation_modifier']}
      >
        <Select
          mode="multiple"
          placeholder="Select modifiers"
          style={{ width: '100%' }}
          options={keyEntries.map(item => ({
            label: item.key,
            value: item.key
          }))}
        />
      </Form.Item>

      <Collapse>
        <Panel header="Available Keys" key="available_keys">
          <Form.Item required={true}>
        <List
          bordered
          dataSource={[hint_key, hint_right_key, hint_left_key]}
          renderItem={(key) => (
            <List.Item key={key}>
              <Space>
                <Form.Item label="Key">
                  <Input value={key} disabled />
                </Form.Item>
                <Form.Item label="Virtual Key Value">
                  <Input type="number" disabled />
                </Form.Item>
              </Space>
            </List.Item>
          )}
        />
        <List
          bordered
          dataSource={keyEntries.filter(item => 
            ![hint_key, hint_right_key, hint_left_key].includes(item.key)
          )}
        renderItem={(item) => (
          <List.Item key={item.key}>
            <Space>
              <Form.Item label="Key">
                <Input value={item.key} disabled />
              </Form.Item>
              <Form.Item label="Virtual Key Value">
                <Input 
                  type="number" 
                  value={item.virtual_key}
                  onChange={(e) => {
                    const newEntries = keyEntries.map(entry => 
                      entry.key === item.key 
                        ? { ...entry, virtual_key: Number(e.target.value) }
                        : entry
                    );
                    updateKeyVk(newEntries);
                  }}
                />
              </Form.Item>
              <MinusCircleOutlined onClick={() => {
                const newEntries = keyEntries.filter(entry => entry.key !== item.key);
                updateKeyVk(newEntries);
              }} />
            </Space>
          </List.Item>
        )}
      />
        <Button 
          type="dashed" 
          onClick={() => {
            const newKey = `Key${keyEntries.length + 1}`;
            updateKeyVk([...keyEntries, { key: newKey, virtual_key: 0 }]);
          }} 
          block 
          icon={<PlusOutlined />}
        >
          Add Key
        </Button>
          </Form.Item>
        </Panel>
      </Collapse>

      <Collapse>
        <Panel header="Left Right Mapping" key="left_right_mapping">
          <Form.Item
      >
        <List
          bordered
          dataSource={keyLeftRightEntries}
          renderItem={(item) => (
            <List.Item key={item.key}>
              <Space>
                <Form.Item label="Key Name">
                  <Input value={item.key} disabled />
                </Form.Item>
                <Form.Item label="Left Key">
                  <Input 
                    value={item.left || ''}
                    onChange={(e) => {
                      const newEntries = keyLeftRightEntries.map(entry => 
                        entry.key === item.key 
                          ? { ...entry, left: e.target.value || null }
                          : entry
                      );
                      updateLeftRight(newEntries);
                    }}
                  />
                </Form.Item>
                <Form.Item label="Right Key">
                  <Input 
                    value={item.right || ''}
                    onChange={(e) => {
                      const newEntries = keyLeftRightEntries.map(entry => 
                        entry.key === item.key 
                          ? { ...entry, right: e.target.value || null }
                          : entry
                      );
                      updateLeftRight(newEntries);
                    }}
                  />
                </Form.Item>
                <MinusCircleOutlined onClick={() => 
                  updateLeftRight(keyLeftRightEntries.filter(entry => entry.key !== item.key))
                } />
              </Space>
            </List.Item>
          )}
        />
        <Button 
          type="dashed" 
          onClick={() => {
            const newKey = `Key${keyLeftRightEntries.length + 1}`;
            updateLeftRight([...keyLeftRightEntries, { key: newKey, left: null, right: null }]);
          }}
          block 
          icon={<PlusOutlined />}
        >
          Add Mapping
        </Button>
          </Form.Item>
        </Panel>
      </Collapse>
    </Space>
  );
}; 