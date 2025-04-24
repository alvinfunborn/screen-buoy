import React from 'react';
import { Form, Space, Typography, Button, Input, List, Spin } from 'antd';
import { MinusCircleOutlined, PlusOutlined } from '@ant-design/icons';
import type { Config, LeftRightConfig } from '../../../types/config';

const { Title } = Typography;

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
  const hint_key = "HintKey";
  const hint_right_key = "HintRightKey";
  const hint_left_key = "HintLeftKey";

  // 获取完整的表单值
  const values = Form.useWatch([], form);
  console.log(values);
  const availableKeys = values?.keyboard?.available_key || {};
  console.log(availableKeys);
  const keyEntries = Object.entries(availableKeys).sort((a, b) => a[1] - b[1])
    .map(([key, value]: [string, number]): KeyEntry => ({
    key,
    virtual_key: value
  }));
  console.log(keyEntries);
  const updateKeyVk = (entries: KeyEntry[]) => {
    const newValue = entries.reduce((acc, { key, virtual_key }) => ({
      ...acc,
      [key]: virtual_key
    }), {});
    form.setFieldValue(['keyboard', 'available_key'], newValue);
  };

  const keyLeftRight = values?.keyboard?.map_left_right || {};
  const keyLeftRightEntries = Object.entries(keyLeftRight).map(([key, value]: [string, LeftRightConfig]): LeftRightEntry => ({
    key,
    left: value?.left || '',
    right: value?.right || ''
  }));
  console.log(keyLeftRightEntries);
  const updateLeftRight = (entries: LeftRightEntry[]) => {
    const newValue = entries.reduce((acc, { key, left, right }) => ({
      ...acc,
      [key]: { left: left || null, right: right || null }
    }), {});
    form.setFieldValue(['keyboard', 'map_left_right'], newValue);
  };

  return (
    <Space direction="vertical" style={{ width: '100%' }}>
      <Title level={3}>Keyboard Settings</Title>
      
      <Form.Item
        label="Available Keys"
        required
      >
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
                    value={item.virtual_key === null ? '' : item.virtual_key}
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
            
          }} 
          block 
          icon={<PlusOutlined />}
        >
          Add Key
        </Button>
      </Form.Item>

      <Form.Item
        label="Propagation Modifier"
        name={['keyboard', 'propagation_modifier']}
      >
        <Form.List name={['keyboard', 'propagation_modifier']}>
          {(fields, { add, remove }) => (
            <Space direction="vertical" style={{ width: '100%' }}>
              <List
                header={<div>Propagation Modifier</div>}
                bordered
                dataSource={fields}
                renderItem={(field) => (
                  <List.Item key={field.key}>
                    <Space>
                      <Form.Item
                        name={field.name}
                        rules={[{ required: true }]}
                      >
                        <Input />
                      </Form.Item>
                      <MinusCircleOutlined onClick={() => remove(field.name)} />
                    </Space>
                  </List.Item>
                )}
              />
              <Button type="dashed" onClick={() => add('')} block icon={<PlusOutlined />}>
                Add Modifier
              </Button>
            </Space>
          )}
        </Form.List>
      </Form.Item>

      <Form.Item
        label="Left Right Mapping"
        required
      >
        <List
          header={<div>Left Right Mapping</div>}
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
                          ? { ...entry, left: e.target.value }
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
                          ? { ...entry, right: e.target.value }
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
          }} 
          block 
          icon={<PlusOutlined />}
        >
          Add Mapping
        </Button>
      </Form.Item>
    </Space>
  );
}; 