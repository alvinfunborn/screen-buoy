import React from 'react';
import { Form, Space, Typography, Button, Input, List, Spin } from 'antd';
import { MinusCircleOutlined, PlusOutlined } from '@ant-design/icons';
import type { Config, LeftRightConfig } from '../../../types/config';

const { Title } = Typography;

interface KeyboardSettingsProps {
  loading?: boolean;
}

export const KeyboardSettings: React.FC<KeyboardSettingsProps> = ({ loading }) => {
  if (loading) {
    return <Spin />;
  }

  const form = Form.useFormInstance<Config>();
  const hint_key = "HintKey";
  const hint_right_key = "HintRightKey";
  const hint_left_key = "HintLeftKey";

  return (
    <Space direction="vertical" style={{ width: '100%' }}>
      <Title level={3}>Keyboard Settings</Title>
      
      <Form.Item
        label="Available Keys"
        required
      >
        <List
          bordered
          dataSource={[ hint_key, hint_right_key, hint_left_key]}
          renderItem={(item) => (
            <List.Item>
              <Space>
                <Form.Item
                  label="Key"
                >
                  <Input value={item} disabled />
                </Form.Item>
                <Form.Item
                  label="Virtual Key Value"
                >
                  <Input type="number" disabled />
                </Form.Item>
              </Space>
            </List.Item>
          )}
        />
        
        <Form.List name={['keyboard', 'available_key']}>
          {(fields, { add, remove }) => (
            <Space direction="vertical" style={{ width: '100%' }}>
              <List
                bordered
                dataSource={fields}
                renderItem={(field) => (
                  <List.Item>
                    <Space>
                      <Form.Item
                        {...field}
                        label="Key"
                        name={[field.name, 0]}
                        rules={[{ required: true }]}
                      >
                        <Input />
                      </Form.Item>
                      <Form.Item
                        {...field}
                        label="Virtual Key Value"
                        name={[field.name, 1]}
                      >
                        <Input type="number" />
                      </Form.Item>
                      <MinusCircleOutlined onClick={() => remove(field.name)} />
                    </Space>
                  </List.Item>
                )}
              />
              <Button type="dashed" onClick={() => add({ key: '', virtual_key: undefined })} block icon={<PlusOutlined />}>
                Add Key
              </Button>
            </Space>
          )}
        </Form.List>
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
                  <List.Item>
                    <Space>
                      <Form.Item
                        {...field}
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
        <Form.List name={['keyboard', 'map_left_right']}>
          {(fields, { add, remove }) => (
            <Space direction="vertical" style={{ width: '100%' }}>
              <List
                header={<div>Left Right Mapping</div>}
                bordered
                dataSource={fields}
                renderItem={(field) => (
                  <List.Item>
                    <Space>
                      <Form.Item
                        {...field}
                        label="Key Name"
                        name={[field.name, 0]}
                        rules={[{ required: true }]}
                      >
                        <Input />
                      </Form.Item>
                      <Form.Item
                        {...field}
                        label="Left Key"
                        name={[field.name, 1, 'left']}
                      >
                        <Input />
                      </Form.Item>
                      <Form.Item
                        {...field}
                        label="Right Key"
                        name={[field.name, 1, 'right']}
                      >
                        <Input />
                      </Form.Item>
                      <MinusCircleOutlined onClick={() => remove(field.name)} />
                    </Space>
                  </List.Item>
                )}
              />
              <Button type="dashed" onClick={() => add(['', { left: '', right: '' }])} block icon={<PlusOutlined />}>
                Add Mapping
              </Button>
            </Space>
          )}
        </Form.List>
      </Form.Item>
    </Space>
  );
}; 